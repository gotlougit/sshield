use async_trait::async_trait;
use clap::Parser;
use cli::{Args, Command};
use db::ProcessedKey;
use rusqlite::Connection;
use russh::{client, ChannelId, Pty};
use russh_keys::{
    key::{KeyPair, PublicKey},
    pkcs8,
};
use std::sync::Arc;

mod cli;
mod db;

#[derive(Clone)]
struct Client {}

#[async_trait]
impl client::Handler for Client {
    type Error = anyhow::Error;

    // TODO: maybe check this against known_hosts
    async fn check_server_key(
        self,
        server_public_key: &PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        Ok((self, true))
    }

    async fn data(
        self,
        _channel: ChannelId,
        data: &[u8],
        session: client::Session,
    ) -> Result<(Self, client::Session), Self::Error> {
        let strrep = std::str::from_utf8(data).unwrap();
        println!("{strrep}");
        Ok((self, session))
    }
}

struct KeyMgr {
    db: Connection,
    client: Client,
}

impl KeyMgr {
    fn init() -> Self {
        KeyMgr {
            db: crate::db::open_db().unwrap(),
            client: Client {},
        }
    }

    async fn connect(&self, nick: &str) {
        let config = Arc::new(russh::client::Config::default());

        match self.show_key(nick) {
            Ok(structkey) => {
                let mut session = client::connect(
                    config,
                    (structkey.host, structkey.port),
                    self.client.clone(),
                )
                .await
                .unwrap();
                if session
                    .authenticate_publickey(structkey.user, Arc::new(structkey.keypair))
                    .await
                    .unwrap()
                {
                    let mut channel = session.channel_open_session().await.unwrap();
                    let mode = [
                        (Pty::ECHOCTL, 0),
                        (Pty::ICANON, 1),
                        (Pty::ISIG, 1),
                        (Pty::ISTRIP, 0),
                        (Pty::IXON, 0),
                        (Pty::IXOFF, 0),
                        (Pty::VFLUSH, 1),
                    ];
                    channel
                        .request_pty(false, "xterm-256color", 80, 24, 10, 10, &mode)
                        .await
                        .unwrap();
                    channel.request_shell(false).await.unwrap();
                    loop {
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap();
                        if input == "exit\n" {
                            channel.close().await.unwrap();
                            return;
                        }
                        channel.data(input.as_bytes()).await.unwrap();
                    }
                }
            }
            Err(_) => {}
        };
    }

    fn gen_key(&self, nick: &str, user: &str, host: &str, port: u16) {
        let key = KeyPair::generate_ed25519().unwrap();
        // store this encoded key in db
        let encoded_key = pkcs8::encode_pkcs8(&key);
        crate::db::insert_key(&self.db, nick, user, host, port, encoded_key);
        self.show_key(nick).unwrap();
    }

    fn show_key(&self, nick: &str) -> Result<ProcessedKey, rusqlite::Error> {
        match crate::db::get_key(&self.db, nick) {
            Ok(res) => {
                let msg_raw = format!("{res}");
                println!("{msg_raw}");
                return Ok(res);
            }
            Err(e) => {
                let msg = "That key doesn't exist, try creating it?";
                println!("{msg}");
                return Err(e);
            }
        }
    }

    fn show_all_keys(&self) {
        let keys = crate::db::get_all_keys(&self.db).unwrap();
        for key in keys.iter() {
            let msg = format!("{key}");
            println!("{msg}");
        }
    }
}
#[tokio::main]
async fn main() {
    let mgr = KeyMgr::init();
    let args = Args::parse();
    match args.command {
        Some(cmd) => {
            match cmd {
                Command::Connect { name } => {
                    mgr.connect(&name).await;
                }
                Command::GenKey {
                    name,
                    user,
                    host,
                    port,
                } => {
                    mgr.gen_key(&name, &user, &host, port);
                }
                Command::ShowKey { name } => match name {
                    Some(name) => {
                        mgr.show_key(&name).unwrap();
                    }
                    None => {
                        mgr.show_all_keys();
                    }
                },
                _ => {
                    println!("hello");
                }
            };
        }
        None => {}
    };
}
