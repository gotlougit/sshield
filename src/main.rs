use async_trait::async_trait;
use clap::{Parser, Subcommand};
use russh::{client, ChannelId};
use russh_keys::{key, load_secret_key};
use std::sync::Arc;

mod db;

#[derive(Subcommand, PartialEq)]
enum Command {
    /// Generate a key for a particular service
    GenKey {
        /// key nickname
        #[arg(required = true)]
        name: String,
        /// Username
        #[arg(long)]
        user: String,
        /// Hostname
        #[arg(long)]
        host: String,
        /// Port (optional)
        #[arg(long, default_value_t = 22)]
        port: u16,
    },
    /// Show a key for a particular service
    ShowKey {
        /// key nickname
        #[arg(required = true)]
        name: String,
    },
    /// Delete a key for a particular service
    DeleteKey {
        /// key nickname
        #[arg(required = true)]
        name: String,
    },
    /// Create a new key for a pre-existing service
    UpdateKey {
        /// key nickname
        #[arg(required = true)]
        name: String,
        /// Username
        #[arg(long, required = false)]
        user: Option<String>,
        /// Hostname
        #[arg(long, required = false)]
        host: Option<String>,
        /// Port (optional)
        #[arg(long, required = false)]
        port: Option<u16>,
    },
    /// Connect to a service using its nickname
    Connect {
        /// key nickname
        #[arg(required = true)]
        name: String,
    },
}

/// Rust reimplementation of SSH client with sandboxing
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

struct Client {}

#[async_trait]
impl client::Handler for Client {
    type Error = anyhow::Error;

    // TODO: maybe check this against known_hosts
    async fn check_server_key(
        self,
        server_public_key: &key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        println!("check_server_key: {:#?}", server_public_key);
        Ok((self, true))
    }

    async fn data(
        self,
        _channel: ChannelId,
        data: &[u8],
        session: client::Session,
    ) -> Result<(Self, client::Session), Self::Error> {
        println!("{}", std::str::from_utf8(data).unwrap());
        Ok((self, session))
    }
}

async fn connect(user: &str, host: &str, keyfile: &str) {
    let config = Arc::new(russh::client::Config::default());
    let sh = Client {};

    let key = load_secret_key(keyfile, None).unwrap();
    let mut session = client::connect(config, (host, 22), sh).await.unwrap();
    if session
        .authenticate_publickey(user, Arc::new(key))
        .await
        .unwrap()
    {
        let mut channel = session.channel_open_session().await.unwrap();
        channel.request_shell(true).await.unwrap();
        if let Some(msg) = channel.wait().await {
            println!("{:#?}", msg);
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.command {
        Some(cmd) => {
            match cmd {
                Command::Connect { name } => {
                    println!("{name}");
                }
                Command::GenKey {
                    name,
                    user,
                    host,
                    port,
                } => {
                    println!("{name}, {user}@{host}:{port}");
                }
                _ => {
                    println!("hello");
                }
            };
        }
        None => {}
    };
}
