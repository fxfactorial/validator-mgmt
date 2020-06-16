use clap::{App, Arg};
use std::io;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Widget};
use tui::Terminal;

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
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    match terminal.draw(|mut f| {
        let size = f.size();
        let block = Block::default().title("Block").borders(Borders::ALL);
        f.render_widget(block, size);
    }) {
        Err(err) => Err(Box::new(err)),
        _ => Ok(()),
    }
}

fn main() {
    let matches = App::new("harmony validator management")
        .version("0.0.1")
        .author("Edgar Aroutiounian <edgar.factorial@gmail.com>")
        .about("manage validator")
        .arg(
            Arg::with_name("file")
                .short('c')
                .takes_value(true)
                .long("yaml-config")
                .about("need path to yaml"),
        )
        .get_matches();

    match matches.value_of("file") {
        None => println!("didnt work out yo"),
        Some(p) => match run_program(p.to_string()) {
            Err(b) => println!("something wrong {:?}", b),
            _ => (),
        },
    }
}
