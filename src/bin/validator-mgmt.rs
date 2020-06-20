#![feature(proc_macro_hygiene)]

use clap::{App, Arg};
use serde::{Deserialize, Serialize};

mod yaml_config {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    pub struct BLSKeyManagement {
        pub adjust_keys_off_median_every: u64,
        pub shard_0_keys: Vec<String>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    #[serde(rename_all = "kebab-case")]
    pub struct EmailCredentials {
        pub account_name: String,
        pub account_password: String,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    pub struct Notifications {
        pub enable: bool,
        pub report_every: u64,
        // TODO these could be named types
        pub email_sender: EmailCredentials,
        pub email_receiver: String,
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

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
struct Validator {
    current_percent: String,
    name: String,
    rate: String,
    signed: u64,
    to_sign: u64,
}

use {
    hyper::{
        service::{make_service_fn, service_fn},
        Body, Client, Request, Response, Server, Uri,
    },
    std::net::SocketAddr,
};

async fn serve_req(
    _req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    // Always return successfully with a response containing a body with
    // a friendly greeting ;)
    println!("Got request at {:?}", _req.uri());
    let url_str = "http://www.google.com";
    let url = url_str.parse::<Uri>().expect("failed to parse URL");
    let res = Client::new().get(url).await?;
    Ok(res)
}

use async_std::{fs::File, io, prelude::*, task};
use std::time::Duration;

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

use duct::cmd;

const C: &'static str = r#"'{"current-percent":.result["current-epoch-performance"]["current-epoch-signing-percent"]["current-epoch-signing-percentage"], "name":.result.validator.name,"rate":.result.validator.rate, "signed":.result["current-epoch-performance"]["current-epoch-signing-percent"]["current-epoch-signed"],"to-sign":.result["current-epoch-performance"]["current-epoch-signing-percent"]["current-epoch-to-sign"]}'"#;

use lettre::{
    message::header, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};

fn send_email_report(
    all: Vec<Validator>,
    creds: Credentials,
    sender: &String,
    receiver: &String,
) {
    use maud::{html, DOCTYPE};
    let report = html! {
    (DOCTYPE)
            meta charset="utf-8";
    h1 { "signing report" }
    head {
            style {
        r#"
    .validator-row {
      font-size: 14px;
      background-color: aliceblue;
      border: solid;
      padding: 2px;
    }
    body {
      display:flex;
      flex-direction:column;
}"#
            }
    }
    body {
        @for val in all {
        p class="validator-row" { (format!("{:?}", val)) }
        }
    }
    };

    match Message::builder()
        .header(header::ContentType(
            "text/html; charset=utf8".parse().unwrap(),
        ))
        .header(header::ContentTransferEncoding::Binary)
        .from(sender.parse().unwrap())
        .to(receiver.parse().unwrap())
        .subject("Validator Signing Report")
        .body(report.into_string())
    {
        Err(reason) => return eprintln!("issue {:?}", reason),
        Ok(email) => {
            let mailer = SmtpTransport::relay("smtp.gmail.com")
                .unwrap()
                .credentials(creds)
                .build();
            match mailer.send(&email) {
                Ok(b) => println!("everything sent well  {:?}", b),
                Err(reason) => eprintln!("issue sending out email {}", reason),
            }
        }
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
    #[derive(Serialize, Deserialize)]
    struct Validators {
        elected: Vec<String>,
    }

    let yaml_config::EmailCredentials {
        account_name,
        account_password,
    } = config.notifications.email_sender;
    let creds = Credentials::new(account_name.clone(), account_password);
    let receipent = config.notifications.email_receiver;
    let endpoint = config.rpc_endpoint;

    loop {
        let output = std::process::Command::new("bash")
            .args(&["-c", bash_str.as_str()])
            .output()
            .expect("why binary not working ");

        match (
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        ) {
            (_, problem) if problem != "" => {
                eprintln!("some hmy issue {}", problem)
            }
            (json_output, _) => {
                let validators: Validators =
                    serde_json::from_str(&json_output).unwrap();
                let all: Vec<Validator> = validators.elected
                    .iter()
                    .map(|val| {
                        let bash_str = format!(
                            "hmy --node={} blockchain validator information {} | jq {}",
                            &endpoint, val, C,
                        );
                        let big_cmd = cmd!("bash", "-c", bash_str.as_str());
                        let stdout = big_cmd.read().unwrap();
                        serde_json::from_str(&stdout).unwrap()
                    })
                    .collect();

                send_email_report(all, creds.clone(), &account_name, &receipent)
            }
        }

        task::sleep(Duration::from_secs(every)).await;
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
                    (_, problem) if problem != "" => {
                        eprintln!("some hmy issue {}", problem)
                    }
                    (json_output, _) => {
                        println!("something good {:?}", json_output)
                    }
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
