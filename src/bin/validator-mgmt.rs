use clap::{App, Arg};

mod yaml_config {
    use std::collections::BTreeMap;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    pub struct BLSKeyManagement {
        pub adjust_keys_off_median_every: u64,
        pub shard_0_keys: Vec<String>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    pub struct Notifications {
        pub enable: bool,
        pub report_every: u64,
        // TODO these could be named types
        pub email_sender: BTreeMap<String, String>,
        pub email_receiver: BTreeMap<String, String>,
        pub mobile_phone: BTreeMap<String, String>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    pub struct Manage {
        pub mainnet_account_addr: String,
        pub collect_rewards_every: u64,
        pub rpc_endpoint: String,
        pub bls_key_management: BLSKeyManagement,
        pub notifications: Notifications,
    }
}

use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn run_program(path: String) -> Result<(), Box<dyn std::error::Error>> {
    let f = std::fs::File::open(path)?;
    let m: yaml_config::Manage = serde_yaml::from_reader(f)?;

    let wrapped = Arc::new(m);
    let (t1_config, t2_config) = (wrapped.clone(), wrapped.clone());

    thread::spawn(move || {
        let (every, endpoint, addr) = (
            t1_config.collect_rewards_every,
            t1_config.rpc_endpoint.as_str(),
            t1_config.mainnet_account_addr.as_str(),
        );
        let args = [
            "--node",
            endpoint,
            "staking",
            "collect-rewards",
            "--delegator-addr",
            addr,
            "--chain-id",
            "mainnet",
        ];

        loop {
            let output = std::process::Command::new("hmy")
                .args(&args)
                .output()
                .expect(format!("hmy command failed - very odd {:?}", args).as_str());
            println!(
                "here is a {}",
                match String::from_utf8(output.stdout) {
                    Ok(s) => s,
                    _ => {
                        println!("something broken");
                        return;
                    }
                }
            );
            thread::sleep(Duration::from_secs(every));
        }
    });

    thread::spawn(move || {
        let (adjust_every, _endpoint, _addr) = (
            t2_config.bls_key_management.adjust_keys_off_median_every,
            t2_config.rpc_endpoint.as_str(),
            t2_config.mainnet_account_addr.as_str(),
        );
        loop {
            thread::sleep(Duration::from_secs(adjust_every));
            println!("running the adjust bls key logic")
        }
    });

    // Here run the reporting status logic
    loop {
        thread::sleep(Duration::from_secs(wrapped.notifications.report_every * 60))
    }
}

use {
    hyper::{
        service::{make_service_fn, service_fn},
        Body, Client, Request, Response, Server, Uri,
    },
    std::net::SocketAddr,
};

async fn serve_req(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    // Always return successfully with a response containing a body with
    // a friendly greeting ;)
    println!("Got request at {:?}", _req.uri());
    let url_str = "http://www.google.com";
    let url = url_str.parse::<Uri>().expect("failed to parse URL");
    let res = Client::new().get(url).await?;
    Ok(res)
}

use async_std::{fs::File, io, prelude::*, task};
use std::io::{Error, ErrorKind};

async fn read_file(path: &str) -> io::Result<String> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}

async fn adjust_bls_keys(config: yaml_config::Manage) {
    loop {
        println!("adjust bls keys ");
        task::sleep(Duration::from_secs(
            config.bls_key_management.adjust_keys_off_median_every,
        ))
        .await
    }
}

async fn handle_reporting(config: yaml_config::Manage) {
    if !config.notifications.enable {
        return println!("email & sms notifications not enabled");
    }
    let bash_str = format!(
        "hmy --node={} blockchain validator elected | jq '{{\"elected\":.result}}'",
        config.rpc_endpoint
    );
    let every = config.notifications.report_every;

    loop {
        // task::sleep(Duration::from_secs(every)).await;

        let output = std::process::Command::new("bash")
            .args(&["-c", bash_str.as_str()])
            .output()
            .expect("why binary not working ");

        match (
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        ) {
            (_, problem) if problem != "" => eprintln!("some hmy issue {}", problem),
            (json_output, _) => println!("something good {:?}", json_output),
        }
        return;
    }
}

async fn collect_rewards(config: yaml_config::Manage) {
    let args = [
        "--node",
        config.rpc_endpoint.as_str(),
        "staking",
        "collect-rewards",
        "--delegator-addr",
        config.mainnet_account_addr.as_str(),
        "--chain-id",
        "mainnet",
    ];
    loop {
        task::sleep(Duration::from_secs(config.collect_rewards_every)).await;
        match std::process::Command::new("hmy").args(&args).output() {
            Ok(output) => {
                match (
                    String::from_utf8(output.stdout).unwrap(),
                    String::from_utf8(output.stderr).unwrap(),
                ) {
                    (_, problem) if problem != "" => eprintln!("some hmy issue {}", problem),
                    (json_output, _) => println!("something good {:?}", json_output),
                }
            }
            Err(oops) => {
                const WAIT_FOR: u64 = 60 * 5;
                eprintln!("issue {:?} with subprocess args {:?}", oops, args);
                task::sleep(Duration::from_secs(WAIT_FOR)).await;
            }
        }
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("harmony validator management")
        .version("0.0.1")
        .about("manage validator")
        .arg(
            Arg::with_name("file")
                .short('c')
                .takes_value(true)
                .long("yaml-config")
                .about("need path to yaml"),
        )
        .get_matches();

    let yaml_path = matches.value_of("file").ok_or("missing file")?;

    async_std::task::block_on(async {
        let config = read_file(yaml_path).await?;
        let m: yaml_config::Manage = serde_yaml::from_str(config.as_str())?;
        let (m2, m3) = (m.clone(), m.clone());
        task::spawn(adjust_bls_keys(m));
        task::spawn(collect_rewards(m2));
        task::spawn(handle_reporting(m3));
        async_std::future::pending().await
    })
}
