mod yaml_config {
    use std::collections::BTreeMap;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct BLSKeyManagement {
        pub adjust_keys_off_median_every: i64,
        pub shard_0_keys: Vec<String>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Notifications {
        pub enable: bool,
        // TODO these could be named types
        pub email_sender: BTreeMap<String, String>,
        pub email_receiver: BTreeMap<String, String>,
        pub mobile_phone: BTreeMap<String, String>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Manage {
        pub mainnet_account_addr: String,
        pub collect_rewards_every: i64,
        pub rpc_endpoint: String,
        pub bls_key_management: BLSKeyManagement,
        pub notifications: Notifications,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = std::fs::File::open("example.yaml")?;
    let m: yaml_config::Manage = serde_yaml::from_reader(f)?;

    println!("Read YAML string: {:?}", m);
    Ok(())
}
