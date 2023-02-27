#!/usr/bin/env bash
# This script sets up the environment for the Ziggurat test suite.

set -e

# Rippled files
if [ -z $RIPPLED_BIN_PATH ]; then
    echo "Aborting. Export RIPPLED_BIN_PATH before running this script."
    exit
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
    echo "start_command = \"./$RIPPLED_BIN_NAME\"" >> $ZIGGURAT_RIPPLED_SETUP_CFG_FILE

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
    # Wait for node to gracefully shutdown before performing cleanup
    SHUTDOWN_GRACE_PERIOD_SEC=10s

    echo "--- Setting up initial node state, takes at least 5 minutes"
    echo
    echo "--- Spinning up a node instance, please be patient"
    cargo t setup::testnet::test::run_testnet -- --ignored &
    echo
    sleep $ACCOUNT_QUERY_DELAY_MIN
    echo "--- Querying account info"
    timeout 10s python3 tools/account_info.py | grep "ResponseStatus.SUCCESS"
    echo
    echo "--- Executing transfer script"
    python3 tools/transfer.py
    echo
    echo "--- Gracefully stopping the network"
    kill -2 $(pidof cargo)
    sleep $SHUTDOWN_GRACE_PERIOD_SEC
    echo "--- Performing cleanup"
    cp -a $ZIGGURAT_RIPPLED_TESTNET_DIR $ZIGGURAT_RIPPLED_STATEFUL_DIR
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