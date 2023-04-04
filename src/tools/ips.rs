// If the IPS array is empty then it means that source IP addresses have not been
// generated and assigned to dummy devices. In such case tests should not bound to any
// specific address and use local pool instead.

/// Reference to a static array of IP addresses (represented as str).
/// If array is empty generate new addresses using eg.: for Linux:
/// sudo python3 ./tools/ips.py --subnet 1.1.1.0/24 --file src/tools/ips.rs --dev_prefix test_zeth
/// for MacOS:
/// sudo python3 ./tools/ips.py --subnet 1.1.1.0/24 --file src/tools/ips.rs --dev lo0
/// For more information read the documentation of the ips.py script.
use std::fs;

use serde::Deserialize;

const IPS_LIST_PATH: &str = "./tools/ips_list.json";

#[derive(Default, Clone, Deserialize, Debug)]
struct IpsList {
    pub nodes: Vec<String>,
}

/// Read in the node array generated by the ips.py
/// script. If file does not exist, return an empty
/// array.
fn load_ips_nodes(filepath: &str) -> Vec<String> {
    let result = fs::read_to_string(filepath);
    match result {
        Ok(jstring) => {
            let ips_list: IpsList = serde_json::from_str(&jstring).unwrap();
            ips_list.nodes
        }
        Err(_) => {
            panic!("Problem reading file: {filepath}.  Confirm that you have run the ips.py script, as described in the readme.");
        }
    }
}

/// Called by clients to obtain a list of
/// nodes generated by the ips.py script.
pub fn ips() -> Vec<String> {
    load_ips_nodes(IPS_LIST_PATH)
}
