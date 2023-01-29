# Ziggurat x XRPL

The Ziggurat implementation for XRPLF's `rippled` nodes.

## Getting started

1. Clone this repository.
2. Build [rippled](https://github.com/XRPLF/rippled) from source.
3. Create the `~/.ziggurat/ripple/setup` directories, and copy the `setup/validators.txt` file there.
   ```
   cp setup/validators.txt ~/.ziggurat/ripple/setup
   ```
4. In the same directory create a `config.toml` with the following contents:
   ```
   path = "<path to the directory where you built rippled>"
   start_command = "./rippled"
   ```
5. Create a package of IP addresses which are required for performance tests. From the root repository directory run, e.g.:
   Under Linux (to generate dummy devices with addresses):
   ```
   sudo python3 ./tools/ips.py --subnet 1.1.1.0/24 --file src/tools/ips.rs --dev_prefix test_zeth
   ```
   Under MacOS or Linux (to add whole subnet to loopback device - under Linux: lo, MacOS: lo0):
   ```
   sudo python3 ./tools/ips.py --subnet 1.1.1.0/24 --file src/tools/ips.rs --dev lo0
   ```
   Read ./tools/ips.py for more details.
6. Run tests with:
   ```
   cargo +stable t -- --test-threads=1
   ```

### Initial state
Specific tests require an initial node state to be set up.
Follow the steps below to save an initial state that can be loaded later in certain tests.

#### Preparation (needs to be done once)
1. Make sure you have python3 installed. You should be able to run `python3 --version`.
2. Install `xrpl` python lib: `pip3 install xrpl-py`.

##### Mac users
Make sure these two `127.0.0.x` (where `x != 1`) addresses are enabled:
```
    sudo ifconfig lo0 alias 127.0.0.2 up;
    sudo ifconfig lo0 alias 127.0.0.3 up;
```

#### Transferring XRP from the Genesis account to a new account and saving the state
1. In one terminal run test `cargo +stable t setup::testnet::test::run_testnet -- --ignored`.
   The test will start a local testnet and will keep it alive for 10 minutes. Ensure that you complete the
   following steps while above test is running.

2. Run `python3 tools/account_info.py` to monitor state of the accounts. 
   Wait until `ResponseStatus.SUCCESS` is reported for the genesis account. The response should include:
   ```
    "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    "Balance": "100000000000000000",
   ```
   This should happen within about a minute.
   Ignore error for the account `rNGknFCRBZguXcPqC63k6xTZnonSe6ZuWt` for the time being.
3. Run `python3 tools/transfer.py` to transfer xrp from genesis account to a new account.
4. Run `python3 tools/account_info.py` again to monitor accounts. The response for genesis account should include:
   ```
        "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "Balance": "99999994999999990",
   ```
   and the response for the new account should include:
   ```
        "Account": "rNGknFCRBZguXcPqC63k6xTZnonSe6ZuWt",
        "Balance": "5000000000",
   ```
5. Copy the node's files to directory referenced by constant `pub const STATEFUL_NODES_DIR`, currently:
   ```
   cp -a ~/.ziggurat/ripple/testnet/ ~/.ziggurat/ripple/stateful;
   ```
6. Now you can stop the test started in step 1.
7. Perform cleanup:
   ```
   rm ~/.ziggurat/ripple/stateful/*/rippled.cfg;  # config files will be created when nodes are started
   rm -rf ~/.ziggurat/ripple/testnet;             # not needed anymore
   ```
## Test status

Short overview of test cases and their current status. In case of failure, the behaviour observed is usually documented in the test case.
These results were obtained by running the test suite against [Ripple 1.9.3](https://github.com/XRPLF/rippled) (47dec467).

| Status |               |
|:------:|---------------|
|   ✓    | pass          |
|   ✖    | fail          |


### Conformance

|             Test Case             | Status | Additional Information |
|:---------------------------------:|:------:|:-----------------------|
| [001](SPEC.md#ZG-CONFORMANCE-001) |   ✓    |                        |
| [002](SPEC.md#ZG-CONFORMANCE-002) |   ✓    |                        |
| [003](SPEC.md#ZG-CONFORMANCE-003) |   ✓    |                        |
| [004](SPEC.md#ZG-CONFORMANCE-004) |   ✓    |                        |
| [005](SPEC.md#ZG-CONFORMANCE-005) |   ✓    |                        |
| [006](SPEC.md#ZG-CONFORMANCE-006) |   ✓    |                        |
| [007](SPEC.md#ZG-CONFORMANCE-007) |   ✓    |                        |
| [008](SPEC.md#ZG-CONFORMANCE-008) |   ✓    |                        |
| [009](SPEC.md#ZG-CONFORMANCE-009) |   ✓    |                        |
| [010](SPEC.md#ZG-CONFORMANCE-010) |   ✓    |                        |
| [011](SPEC.md#ZG-CONFORMANCE-011) |   ✓    |                        |
| [012](SPEC.md#ZG-CONFORMANCE-012) |   ✓    |                        |
| [013](SPEC.md#ZG-CONFORMANCE-013) |   ✓    |                        |
| [014](SPEC.md#ZG-CONFORMANCE-014) |   ✓    |                        |
| [015](SPEC.md#ZG-CONFORMANCE-015) |   ✓    |                        |
| [016](SPEC.md#ZG-CONFORMANCE-016) |   ✓    |                        |
| [017](SPEC.md#ZG-CONFORMANCE-017) |   ✓    |                        |
| [018](SPEC.md#ZG-CONFORMANCE-018) |   ✓    |                        |
| [019](SPEC.md#ZG-CONFORMANCE-019) |   ✓    |                        |
| [020](SPEC.md#ZG-CONFORMANCE-020) |   ✓    |                        |
| [021](SPEC.md#ZG-CONFORMANCE-021) |   ✓    |                        |
| [022](SPEC.md#ZG-CONFORMANCE-022) |   ✓    |                        |
| [023](SPEC.md#ZG-CONFORMANCE-023) |   ✓    |                        |
| [024](SPEC.md#ZG-CONFORMANCE-024) |   ✓    |                        |
| [025](SPEC.md#ZG-CONFORMANCE-025) |   ✓    |                        |
| [026](SPEC.md#ZG-CONFORMANCE-026) |   ✓    |                        |

### Performance

Tests are ignored by default. To explicitly run performance tests, run the command:
```
 cargo +stable test performance --features performance
```

|             Test Case             | Status | Additional Information |
|:---------------------------------:|:------:|:-----------------------|
| [001](SPEC.md#ZG-PERFORMANCE-001) |   ✓    |                        |
| [002](SPEC.md#ZG-PERFORMANCE-002) |   ✓    |                        |
| [003](SPEC.md#ZG-PERFORMANCE-003) |   ✓    |                        |

### Resistance

|            Test Case             | Status | Additional Information |
|:--------------------------------:|:------:|:-----------------------|
| [001](SPEC.md#ZG-RESISTANCE-001) |   ✓    |                        |
| [002](SPEC.md#ZG-RESISTANCE-002) |  ✓/✖   | ⚠ Fails in rare cases  |
| [003](SPEC.md#ZG-RESISTANCE-003) |   ✓    |                        |
| [004](SPEC.md#ZG-RESISTANCE-004) |   ✓    |                        |
