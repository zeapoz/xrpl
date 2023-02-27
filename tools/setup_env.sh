#!/usr/bin/env bash
# This script sets up the environment for the Ziggurat test suite.

# Rippled files
if [ -z $RIPPLED_BIN_PATH ]; then
    echo "Aborting. Export RIPPLED_BIN_PATH before running this script."
    exit 1
fi
RIPPLED_BIN_NAME="rippled"

# Ziggurat config files
ZIGGURAT_RIPPLED_DIR="$HOME/.ziggurat/ripple"
ZIGGURAT_RIPPLED_SETUP_DIR="$ZIGGURAT_RIPPLED_DIR/setup"
ZIGGURAT_RIPPLED_SETUP_CFG_FILE="$ZIGGURAT_RIPPLED_SETUP_DIR/config.toml"
ZIGGURAT_RIPPLED_TESTNET_DIR="$ZIGGURAT_RIPPLED_DIR/testnet"
ZIGGURAT_RIPPLED_STATEFUL_DIR="$ZIGGURAT_RIPPLED_SETUP_DIR/stateful"

setup_config_file() {
    echo "--- Setting up configuration file"
    echo "Creating $ZIGGURAT_RIPPLED_SETUP_CFG_FILE with contents:"
    mkdir -p $ZIGGURAT_RIPPLED_SETUP_DIR
    echo
    echo "# Rippled installation path" > $ZIGGURAT_RIPPLED_SETUP_CFG_FILE
    echo "path = \"$RIPPLED_BIN_PATH\"" >> $ZIGGURAT_RIPPLED_SETUP_CFG_FILE
    echo "# Start command with possible arguments" >> $ZIGGURAT_RIPPLED_SETUP_CFG_FILE
    echo "start_command = \"./$RIPPLED_BIN_NAME --silent\"" >> $ZIGGURAT_RIPPLED_SETUP_CFG_FILE

    # Print file contents so the user can check whether the path is correct
    cat $ZIGGURAT_RIPPLED_SETUP_CFG_FILE
    echo
}

setup_ip_addresses() {
    echo "--- Creating a package of IP addresses required for performance tests"
    sudo python3 ./tools/ips.py --subnet 1.1.1.0/24 --file src/tools/ips.rs --dev_prefix test_zeth
    sudo python3 ./tools/ips.py --subnet 1.1.1.0/24 --file src/tools/ips.rs --dev lo
    echo
}

setup_initial_node_state() {
    # Query only after a long delay to account for compilation times and network preparation work
    ACCOUNT_QUERY_DELAY_MIN=5m

    echo "--- Setting up initial node state, takes at least 5 minutes"
    echo
    echo "Spinning up a node instance, please be patient"
    cargo t setup::testnet::test::run_testnet -- --ignored &
    echo
    sleep $ACCOUNT_QUERY_DELAY_MIN
    echo "--- Querying account info"
    # Run account query until it responds with "ResponseStatus.SUCCESS" or MAX_ATTEMPTS is reached
    ACCOUNT_QUERY_TIMEOUT_SEC=5s
    MAX_ATTEMPTS=5
    NUM_ATTEMPTS=0
    until [ $NUM_ATTEMPTS -gt $(($MAX_ATTEMPTS-1)) ] || timeout $ACCOUNT_QUERY_TIMEOUT_SEC python3 tools/account_info.py | grep "ResopnseStatus.SUCCESS"; do
        ((NUM_ATTEMPTS++))
        echo "Query failed, number of attempts made: $NUM_ATTEMPTS"
        sleep $ACCOUNT_QUERY_TIMEOUT_SEC
    done
    echo "Established connection with genesis account"
    echo
    echo "--- Executing transfer script"
    python3 tools/transfer.py
    # Copy the node's files to directory referenced by constant pub const STATEFUL_NODES_DIR
    cp -a $ZIGGURAT_RIPPLED_TESTNET_DIR $ZIGGURAT_RIPPLED_STATEFUL_DIR
    echo
    echo "--- Gracefully stopping the network"
    kill -2 $(pidof cargo)
    echo "--- Performing cleanup"
    # Remove unneeded and temporary files
    rm $ZIGGURAT_RIPPLED_STATEFUL_DIR/*/rippled.cfg
    rm -rf $ZIGGURAT_RIPPLED_TESTNET_DIR
    echo
}

# Verify the repo location
if [ "$(git rev-parse --is-inside-work-tree 2>/dev/null)" != "true" ]; then
    echo "Aborting. Use this script only from the ziggurat/xrpl repo."
    exit 1
fi
REPO_ROOT=`git rev-parse --show-toplevel`
if [ "`basename $REPO_ROOT`" != "xrpl" ]; then
    # Wrong root directory, check for rename compared to origin url.
    ORIGIN_URL=$(git config --local remote.origin.url|sed -n 's#.*/\([^.]*\)\.git#\1#p')
    if [ "$ORIGIN_URL" != "xrpl" ]; then
        echo "Aborting. Use this script only from the ziggurat/xrpl repo."
        exit 1
    fi
fi

# Setup the main ziggurat directory in the home directory
rm -rf $ZIGGURAT_RIPPLED_DIR

# Change dir to ensure script paths are always correct
pushd . &> /dev/null
cd $REPO_ROOT;

setup_config_file
cp setup/validators.txt $ZIGGURAT_RIPPLED_SETUP_DIR
setup_ip_addresses
setup_initial_node_state
echo "--- Setup successful"

popd &> /dev/null