use std::{
    io::ErrorKind,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::{Duration, Instant},
};

use tempfile::TempDir;
use tokio::net::TcpSocket;

use crate::{
    protocol::{
        codecs::message::Payload,
        proto::{
            tm_get_object_by_hash::ObjectType, TmGetObjectByHash, TmIndexedObject, TmTransactions,
        },
    },
    setup::node::{Node, NodeType},
    tools::{
        constants::{EXPECTED_RESULT_TIMEOUT, TEST_ACCOUNT},
        ips::IPS,
        metrics::{
            recorder::TestMetrics,
            tables::{duration_as_ms, RequestStats, RequestsTable},
        },
        rpc::{get_transaction_info, wait_for_account_data, wait_for_state},
        synth_node::SyntheticNode,
    },
};

const MAX_PEERS: usize = 100;
const METRIC_LATENCY: &str = "transaction_test_latency";
const CONNECTION_PORT: u16 = 31337;
// number of requests to send per peer
const REQUESTS: u16 = 150;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const TX_HASH_LEN: usize = 32;

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[allow(non_snake_case)]
async fn p003_t1_GET_TRANSACTION_latency() {
    // ZG-PERFORMANCE-003, Transaction getting latency
    //
    // The node behaves as expected under load from other peers.
    //
    // We test the overall performance of a node's get transaction latency.
    // There are possible several error messages during the test, all with the error kind:
    // thread 'tokio-runtime-worker' panicked at 'unable to connect to node: Kind(TimedOut)', src/tests/performance/get_trans.rs:152:54
    // The above error means that ziggurat was unable to connect to the node. This error is
    // thrown above 100 concurrent connections.
    //
    // Sample results:
    //
    // ┌─────────┬────────────┬────────────┬────────────┬────────────────┬────────────┬────────────┬────────────┬────────────┬────────────┬────────────────┬────────────┬──────────────┐
    // │  peers  │  requests  │  min (ms)  │  max (ms)  │  std dev (ms)  │  10% (ms)  │  50% (ms)  │  75% (ms)  │  90% (ms)  │  99% (ms)  │  completion %  │  time (s)  │  requests/s  │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │       1 │        150 │          0 │         49 │              5 │          0 │          0 │          0 │          0 │         49 │         100.00 │       0.32 │       471.98 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      10 │        150 │          0 │         56 │              4 │          0 │          0 │          0 │          0 │          0 │         100.00 │       2.19 │       684.12 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      20 │        150 │          0 │        149 │              5 │          0 │          0 │          0 │          0 │          2 │         100.00 │       3.76 │       796.98 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      50 │        150 │          0 │        134 │              3 │          0 │          1 │          1 │          2 │          6 │         100.00 │      10.42 │       719.48 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      75 │        150 │          0 │        266 │              4 │          2 │          2 │          2 │          3 │         10 │         100.00 │      13.85 │       812.22 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     100 │        150 │          0 │        133 │              4 │          1 │          2 │          2 │          3 │         11 │          68.00 │      19.47 │       523.90 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     125 │        150 │          0 │        753 │              8 │          3 │          3 │          3 │          3 │         11 │          68.00 │      34.34 │       371.28 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     150 │        150 │          0 │       4146 │            372 │          3 │          3 │          3 │          4 │       2014 │          58.00 │      38.47 │       339.23 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     200 │        150 │          0 │      13640 │            487 │          2 │          3 │          3 │          3 │         94 │          44.50 │      48.56 │       274.92 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     250 │        150 │          0 │      22103 │            373 │          3 │          3 │          3 │          4 │         81 │          34.00 │      58.25 │       218.87 │
    // └─────────┴────────────┴────────────┴────────────┴────────────────┴────────────┴────────────┴────────────┴────────────┴────────────┴────────────────┴────────────┴──────────────┘
    // *NOTE* run with `cargo test --release tests::performance::get_transaction -- --nocapture`
    // Before running test generate dummy devices with different ips using toos/ips.py

    let synth_counts = vec![1, 10, 20, 50, 75, 100, 125, 150, 200, 250];

    let mut table = RequestsTable::default();

    for synth_count in synth_counts {
        let target = TempDir::new().expect("Unable to create TempDir");
        let mut node = Node::builder()
            .max_peers(MAX_PEERS)
            .start(target.path(), NodeType::Stateful)
            .await
            .unwrap();
        let node_addr = node.addr();

        let mut synth_sockets = Vec::with_capacity(synth_count);
        let mut ips = IPS.to_vec();

        for _ in 0..synth_count {
            let socket = TcpSocket::new_v4().unwrap();

            // Make sure we can reuse the address and port
            socket.set_reuseaddr(true).unwrap();
            socket.set_reuseport(true).unwrap();

            // If there is address for our thread in the pool we can use it.
            // Otherwise we'll not set bound_addr and use local IP addr (127.0.0.1).
            let ip = if let Some(ip_addr) = ips.pop() {
                SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::from_str(ip_addr).unwrap()),
                    CONNECTION_PORT,
                )
            } else {
                "127.0.0.1:0".parse().unwrap()
            };

            socket.bind(ip).expect("unable to bind to socket");
            synth_sockets.push(socket);
        }

        // setup metrics recorder
        let test_metrics = TestMetrics::default();
        // clear metrics and register metrics
        metrics::register_histogram!(METRIC_LATENCY);

        // Wait for correct state and account data.
        wait_for_state(&node.rpc_url(), "proposing".into()).await;
        let account_data =
            wait_for_account_data(&node.rpc_url(), TEST_ACCOUNT, EXPECTED_RESULT_TIMEOUT)
                .await
                .expect("unable to get account data");

        // Get transaction info by rpc to put in cache.
        let tx = account_data.result.account_data.previous_transaction;
        let _ = get_transaction_info(&node.rpc_url(), tx.clone())
            .await
            .expect("unable to get transaction info");

        let mut tx_hash = [0u8; TX_HASH_LEN];
        hex::decode_to_slice(&tx, &mut tx_hash as &mut [u8])
            .expect("unable to decode transaction hash");

        let mut synth_handles = Vec::with_capacity(synth_count);
        let test_start = tokio::time::Instant::now();

        for socket in synth_sockets {
            synth_handles.push(tokio::spawn(simulate_peer(node_addr, socket, tx_hash)));
        }

        // wait for peers to complete
        for handle in synth_handles {
            let _ = handle.await;
        }

        let time_taken_secs = test_start.elapsed().as_secs_f64();

        let snapshot = test_metrics.take_snapshot();
        if let Some(latencies) = snapshot.construct_histogram(METRIC_LATENCY) {
            if latencies.entries() >= 1 {
                // add stats to table display
                table.add_row(RequestStats::new(
                    synth_count as u16,
                    REQUESTS,
                    latencies,
                    time_taken_secs,
                ));
            }
        }

        node.stop().unwrap();
    }

    // Display results table
    println!("\r\n{}", table);
}

async fn simulate_peer(node_addr: SocketAddr, socket: TcpSocket, tx_hash: [u8; TX_HASH_LEN]) {
    let mut synth_node = SyntheticNode::new(&Default::default()).await;

    // Establish peer connection
    synth_node
        .connect_from(node_addr, socket)
        .await
        .expect("unable to connect to node");

    let mut seq: u32 = 1;

    for _ in 0..REQUESTS {
        let payload = Payload::TmGetObjectByHash(TmGetObjectByHash {
            r#type: ObjectType::OtTransactions as i32,
            query: true,
            seq: Some(seq),
            ledger_hash: None,
            fat: None,
            objects: vec![TmIndexedObject {
                hash: Some(tx_hash.into()),
                node_id: None,
                index: None,
                data: None,
                ledger_seq: None,
            }],
        });

        seq += 1;

        // Query transaction via peer protocol.
        if synth_node.is_connected(node_addr) {
            synth_node
                .unicast(node_addr, payload)
                .expect("unable to send message");
        } else {
            synth_node.shut_down().await;
            return;
        }

        let now = Instant::now();

        let mut matched = false;
        while !matched {
            match synth_node.recv_message_timeout(REQUEST_TIMEOUT).await {
                Ok(m) => {
                    if matches!(
                        &m.1.payload,
                        Payload::TmTransactions(TmTransactions {transactions})
                        if transactions.len() == 1
                    ) {
                        metrics::histogram!(METRIC_LATENCY, duration_as_ms(now.elapsed()));
                        matched = true;
                    }
                }
                Err(e) => match e.kind() {
                    ErrorKind::TimedOut => {
                        break;
                    }
                    _ => {
                        panic!("Unexpected error: {:?}", e);
                    }
                },
            }
        }
    }

    synth_node.shut_down().await
}
