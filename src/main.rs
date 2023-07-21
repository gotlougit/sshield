use async_trait::async_trait;
use clap::{Parser, Subcommand};
use russh::{client, ChannelId};
use russh_keys::{
    encode_pkcs8_pem,
    key::{self, KeyPair},
    load_secret_key, PublicKeyBase64,
};
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
        #[arg(required = true)]
        user: String,
        /// Hostname
        #[arg(required = true)]
        host: String,
        /// Port (optional)
        #[arg(short, long, default_value_t = 22)]
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

async fn connect(nick: &str) {
    let config = Arc::new(russh::client::Config::default());
    let sh = Client {};

    let key = show_key(nick).unwrap();
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

fn gen_key(nick: &str, user: &str, host: &str, port: u16) {
    let key = russh_keys::key::KeyPair::generate_ed25519().unwrap();
    let cipher = key.name();
    let pubkey = key.clone_public_key().unwrap();
    // store this encoded key in db
    let encoded_key = russh_keys::pkcs8::encode_pkcs8(&key);
    crate::db::insert_key(nick, user, host, port, encoded_key, cipher);
    println!("Generated SSH key '{nick}' for {user}@{host}:{port}");
    println!("Public key:");
    println!("{} {} {}", cipher, pubkey.public_key_base64(), nick);
}

fn show_key(nick: &str) -> Result<KeyPair, rusqlite::Error> {
    match crate::db::get_key(nick) {
        Ok(res) => {
            println!("{res}");
            let rawkey = res.get_key();
            let key = russh_keys::pkcs8::decode_pkcs8(rawkey, None).unwrap();
            return Ok(key);
        }
        Err(e) => {
            eprintln!("That key doesn't exist, try creating it?");
            return Err(e);
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
                    gen_key(&name, &user, &host, port);
                }
                Command::ShowKey { name } => {
                    show_key(&name);
                }
                _ => {
                    println!("hello");
                }
            };
        }
        None => {}
    };
}
