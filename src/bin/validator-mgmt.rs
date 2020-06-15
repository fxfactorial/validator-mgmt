mod yaml_config {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct BLSKeyManagement {
        adjust_keys_off_median_every: i64,
        shard_0_keys: Vec<String>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct EmailSender {
        account_name: String,
        account_password: String,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Notifications {
        enabled: bool,
        email_sender: EmailSender,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Manage {
        mainnet_account_addr: String,
        collect_rewards_every: i64,
        rpc_endpoint: String,
        bls_key_management: BLSKeyManagement,
        notifications: Notifications,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let f = std::fs::File::open("example.yaml")?;
    // let d: String = serde_yaml::from_reader(f)?;

    // let input_config = std::fs::read_to_string("example.yaml")?;

    // println!("Read YAML string: {:?}", f);
    Ok(())
}
