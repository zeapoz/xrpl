use std::{
    io::ErrorKind,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::{Duration, Instant},
};

use rand::{thread_rng, RngCore};
use tempfile::TempDir;
use tokio::net::TcpSocket;

use crate::{
    protocol::{
        codecs::message::Payload,
        proto::{tm_ping::PingType, TmPing},
    },
    setup::node::{Node, NodeType},
    tools::{
        config::TestConfig,
        ips::IPS,
        metrics::{
            recorder::TestMetrics,
            tables::{duration_as_ms, RequestStats, RequestsTable},
        },
        synth_node::SyntheticNode,
    },
};

const MAX_PEERS: usize = 100;
const PINGS: u16 = 1000;
const METRIC_LATENCY: &str = "ping_perf_latency";
const CONNECTION_PORT: u16 = 31337;

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[allow(non_snake_case)]
async fn p001_t1_PING_PONG_throughput() {
    // ZG-PERFORMANCE-001, Ping-Pong latency
    //
    // Testing the overall performance of a node's Ping-Pong latency. Two main parameters are
    // important for this test:
    // - the number of pings sent to the node by each synthetic peer
    // - the number of synthetic peers
    //
    // Note: This test does not assert any requirements, but requires manual inspection
    //       of the results table. This is because the results will rely on the machine
    //       running the test.
    //
    // rippled: Currently seems to perform quite well. There is one important condition - connections
    //          must be established with different source IPs. When connections come from single IP
    //          the performance drops significantly.
    //          There are possible several error messages during the test:
    //          `Result::unwrap()` on an `Err` value: Kind(InvalidData) - when connect to node failed
    //          `Result::unwrap()` on an `Err` value: Os { code: 32, kind: BrokenPipe, message: "Broken pipe" }' - communication with
    //                  already connected node suddenly failed.
    //          'Error receiving message: true' - when timeout occurs and reply was not received in 10s after sending request.
    //
    //          Conclusion: seems that lower completion percentage when there are more synthetic peers is caused by the fact that
    //          connections cannot be established and other ones are closed during the test. However, amount of nodes and ping count
    //          does not affect the latency and rippled responses have similar std time.
    //
    // Example test result (with percentile latencies) - 1000 pings per node with max_peers set to 100 - node was restarted each interation:
    // ┌─────────┬────────────┬────────────┬────────────┬────────────────┬────────────┬────────────┬────────────┬────────────┬────────────┬────────────────┬────────────┬──────────────┐
    // │  peers  │  requests  │  min (ms)  │  max (ms)  │  std dev (ms)  │  10% (ms)  │  50% (ms)  │  75% (ms)  │  90% (ms)  │  99% (ms)  │  completion %  │  time (s)  │  requests/s  │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │       1 │       1000 │          0 │         49 │              3 │          0 │          0 │          0 │          0 │          0 │         100.00 │       0.59 │      1698.56 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      10 │       1000 │          0 │         58 │              2 │          0 │          0 │          0 │          0 │          0 │         100.00 │       2.02 │      4961.28 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      15 │       1000 │          0 │         59 │              6 │          0 │          0 │          0 │          0 │         43 │         100.00 │       2.82 │      5328.07 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      20 │       1000 │          0 │         60 │              4 │          0 │          0 │          0 │          0 │          0 │         100.00 │       4.22 │      4743.56 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      30 │       1000 │          0 │         59 │              7 │          0 │          0 │          0 │          0 │         49 │         100.00 │       6.94 │      4319.85 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      50 │       1000 │          0 │        369 │              8 │          0 │          0 │          0 │          0 │         47 │         100.00 │      11.20 │      4463.01 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     100 │       1000 │          0 │       3130 │             48 │          0 │          0 │          0 │          1 │         44 │          71.01 │     137.67 │       515.81 │
    // └─────────┴────────────┴────────────┴────────────┴────────────────┴────────────┴────────────┴────────────┴────────────┴────────────┴────────────────┴────────────┴──────────────┘
    //
    // Example test result (with percentile latencies) - 1000 pings per node with max_peers set to 100 - node was NOT restarted each interation:
    // ┌─────────┬────────────┬────────────┬────────────┬────────────────┬────────────┬────────────┬────────────┬────────────┬────────────┬────────────────┬────────────┬──────────────┐
    // │  peers  │  requests  │  min (ms)  │  max (ms)  │  std dev (ms)  │  10% (ms)  │  50% (ms)  │  75% (ms)  │  90% (ms)  │  99% (ms)  │  completion %  │  time (s)  │  requests/s  │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │       1 │       1000 │          0 │         47 │              2 │          0 │          0 │          0 │          0 │          0 │         100.00 │       0.33 │      3009.46 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      10 │       1000 │          0 │         57 │              5 │          0 │          0 │          0 │          0 │          0 │          99.52 │      12.14 │       820.00 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      15 │       1000 │          0 │         59 │              3 │          0 │          0 │          0 │          0 │          0 │          97.33 │      12.00 │      1216.49 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      20 │       1000 │          0 │         84 │              4 │          0 │          0 │          0 │          0 │          0 │          87.69 │      13.55 │      1293.90 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      30 │       1000 │          0 │         67 │              4 │          0 │          0 │          0 │          0 │          0 │          86.68 │      14.89 │      1746.04 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      50 │       1000 │          0 │        302 │              5 │          0 │          0 │          0 │          0 │          1 │          95.32 │      19.10 │      2495.61 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     100 │       1000 │          0 │        177 │              8 │          0 │          0 │          0 │          1 │         48 │          66.00 │     143.66 │       459.43 │
    // └─────────┴────────────┴────────────┴────────────┴────────────────┴────────────┴────────────┴────────────┴────────────┴────────────┴────────────────┴────────────┴──────────────┘
    // *NOTE* run with `cargo test --release tests::performance::ping_pong -- --nocapture`
    // Before running test generate dummy devices with different ips using toos/ips.py

    let synth_counts = vec![1, 10, 15, 20, 30, 50, 100];

    let mut table = RequestsTable::default();

    let mut port_idx = 0;

    let target = TempDir::new().expect("Unable to create TempDir");
    let mut node = Node::builder()
        .max_peers(MAX_PEERS)
        .start(target.path(), NodeType::Stateless)
        .await
        .unwrap();
    let node_addr = node.addr();

    for synth_count in synth_counts {
        let mut synth_sockets = Vec::with_capacity(synth_count);

        #[allow(clippy::needless-range-loop)]
        for i in 0..synth_count {
            let socket = TcpSocket::new_v4().unwrap();

            // Make sure we can reuse the address and port
            socket.set_reuseaddr(true).unwrap();
            socket.set_reuseport(true).unwrap();

            // If there is address for our thread in the pool we can use it.
            // Otherwise we'll not set bound_addr and use local IP addr (127.0.0.1).
            if IPS.len() > i {
                let source_addr = SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::from_str(IPS[i]).unwrap()),
                    CONNECTION_PORT + port_idx,
                );
                port_idx += 1;
                socket.bind(source_addr).expect("unable to bind to socket");
            } else {
                socket
                    .bind("127.0.0.1:0".parse().unwrap())
                    .expect("unable to bind to socket");
            }
            synth_sockets.push(socket);
        }

        // setup metrics recorder
        let test_metrics = TestMetrics::default();
        // clear metrics and register metrics
        metrics::register_histogram!(METRIC_LATENCY);

        let mut synth_handles = Vec::with_capacity(synth_count);
        let test_start = tokio::time::Instant::now();
        for _ in 0..synth_count {
            let sock = synth_sockets.remove(0);
            synth_handles.push(tokio::spawn(simulate_peer(node_addr, sock)));
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
                    PINGS,
                    latencies,
                    time_taken_secs,
                ));
            }
        }
    }

    node.stop().unwrap();

    // Display results table
    println!("\r\n{}", table);
}

async fn simulate_peer(node_addr: SocketAddr, socket: TcpSocket) {
    let config = TestConfig::default();

    let mut synth_node = SyntheticNode::new(&config).await;

    // Establish peer connection
    synth_node.connect_from(node_addr, socket).await.unwrap();
    let mut seq;

    for _ in 0..PINGS {
        // Generate unique sequence for each ping
        seq = thread_rng().next_u32();

        let payload = Payload::TmPing(TmPing {
            r#type: PingType::PtPing as i32,
            seq: Some(seq),
            ping_time: None,
            net_time: None,
        });

        // Send Ping
        synth_node.unicast(node_addr, payload).unwrap();

        let now = Instant::now();
        let mut matched = false;

        // There is a need to read messages in a loop as we can read message that is not ping reply.
        while !matched {
            match synth_node
                .recv_message_timeout(Duration::from_secs(10))
                .await
            {
                Ok(m) => {
                    if matches!(
                        &m.1.payload,
                        Payload::TmPing(TmPing {
                        r#type: r_type,
                        seq: Some(s),
                        ..
                        }) if *s == seq && *r_type == PingType::PtPong as i32
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
