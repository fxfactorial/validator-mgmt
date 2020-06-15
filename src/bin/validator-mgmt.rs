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

fn run_program(path: String) -> Result<(), Box<dyn std::error::Error>> {
    let f = std::fs::File::open(path)?;
    let m: yaml_config::Manage = serde_yaml::from_reader(f)?;

    Ok(())
}

use clap::{App, Arg};

fn main() {
    let matches = App::new("harmony validator management")
        .version("0.0.1")
        .author("Edgar Aroutiounian <edgar.factorial@gmail.com>")
        .about("manage validator")
        .arg(
            Arg::with_name("yaml-config")
                .short('c')
                .long("yaml-config")
                .about("need path to yaml"),
        )
        .get_matches();
    match matches.value_of("yaml-config") {
        None => println!("didnt work out yo"),
        Some(p) => {
            println!("here is more");
            run_program("example.yaml".to_string());
            ()
        }
    }
}
