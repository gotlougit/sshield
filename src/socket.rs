use crate::config::Prompt;
use crate::db::{self, ProcessedKey};
use crate::gui;
use anyhow::Result;
use async_trait::async_trait;
use chrono::prelude::*;
use chrono::Duration;
use rusqlite::Connection;
use russh_keys::load_secret_key;
use russh_keys::{agent::client, agent::server, agent::server::MessageType, key::KeyPair, pkcs8};
use ssh2_config::{ParseRule, SshConfig};
use std::sync::Arc;
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use tokio::macros::support::Future;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

const SOCKNAME: &str = "/run/user/1000/ssh-agent";

#[derive(Clone)]
struct SecureAgent {
    auth_timeout: Prompt,
    last_auth_time: Arc<Mutex<DateTime<Utc>>>,
}

#[async_trait]
impl server::Agent for SecureAgent {
    fn confirm(self, _pk: Arc<KeyPair>) -> Box<dyn Future<Output = (Self, bool)> + Unpin + Send> {
        Box::new(futures::future::ready((self, true)))
    }
    async fn confirm_request(&self, msg: MessageType) -> bool {
        match self.auth_timeout {
            Prompt::NoPrompt => true,
            Prompt::EveryNSeconds(diff) => {
                // Only prompt on requesting signing since that is most important
                let msgstr = match msg {
                    MessageType::Sign => "Allow request to sign data?",
                    _ => "",
                };
                if msgstr.is_empty() {
                    return true;
                }
                let current_time = Utc::now();
                let mut last_time = self.last_auth_time.lock().await;
                let timediff = current_time - *last_time;
                if timediff < Duration::seconds(diff) {
                    return true;
                }
                if gui::confirm_request(msgstr) {
                    *last_time = current_time;
                    return true;
                }
                false
            }
        }
    }
}

pub async fn start_server(auth_timeout: Prompt) {
    match UnixListener::bind(SOCKNAME) {
        Ok(listener) => {
            let wrapper = tokio_stream::wrappers::UnixListenerStream::new(listener);
            let last_auth_time = Arc::new(Mutex::new(Utc::now()));
            server::serve(
                wrapper,
                SecureAgent {
                    auth_timeout,
                    last_auth_time,
                },
            )
            .await
            .unwrap();
        }
        Err(e) => {
            eprintln!("Error while starting agent server: {}", e);
        }
    }
}

pub async fn close_socket() {
    fs::remove_file(SOCKNAME).await.unwrap();
}

pub struct Client {
    conn: Connection,
}

impl Client {
    pub fn init(pass: &str, db_path: &str) -> Result<Self> {
        let conn = db::open_db(pass, db_path)?;
        // Here we would ideally place some decryption mechanisms to handle
        // sensitive key data
        Ok(Self { conn })
    }

    pub async fn import_key_from_file(
        &self,
        pass: Option<String>,
        nick: &str,
        keypath: &str,
    ) -> bool {
        let mut confpath = dirs::home_dir().unwrap();
        confpath.push(".ssh/config");
        let mut configfile = File::open(confpath).await.unwrap();
        let mut reader: Vec<u8> = Vec::new();
        configfile.read_to_end(&mut reader).await.unwrap();
        let config = SshConfig::default()
            .parse(&mut reader.as_slice(), ParseRule::STRICT)
            .unwrap();
        let params = config.query(nick);
        let host = params.host_name.unwrap();
        let user = params.user.unwrap();
        let port = params.port.unwrap_or(22);
        let key = load_secret_key(keypath, pass.as_deref()).unwrap();
        let encoded_key = pkcs8::encode_pkcs8(&key);
        crate::db::insert_key(&self.conn, nick, &user, &host, port, encoded_key)
    }

    pub async fn add_key_to_running_agent(&self, key: &ProcessedKey) {
        match UnixStream::connect(SOCKNAME).await {
            Ok(stream) => {
                let mut dummyclient = client::AgentClient::connect(stream);
                println!("adding key {}", key.nickname);
                let pair = key.keypair.clone();
                dummyclient.add_identity(&pair, &[]).await.unwrap();
            }
            Err(_) => eprintln!("Couldn't connect to agent, is it running?"),
        }
    }

    pub async fn add_all_keys(&self) {
        // Add keys to server automatically
        // This is done by creating a dummy client that adds all the keys we have
        let keys = self.show_all_keys();
        for key in keys.iter() {
            self.add_key_to_running_agent(key).await;
        }
    }

    pub async fn gen_key(&self, nick: &str, user: &str, host: &str, port: u16) -> bool {
        let key = KeyPair::generate_ed25519().unwrap();
        // store this encoded key in db
        let encoded_key = pkcs8::encode_pkcs8(&key);
        let res = crate::db::insert_key(&self.conn, nick, user, host, port, encoded_key);
        if !res {
            return false;
        }
        let processedkey = self.show_key(nick).unwrap();
        // insert key into running agent, if any
        self.add_key_to_running_agent(&processedkey).await;
        true
    }

    pub fn show_key(&self, nick: &str) -> Result<ProcessedKey> {
        match crate::db::get_key(&self.conn, nick) {
            Ok(res) => Ok(res),
            Err(e) => {
                println!("That key doesn't exist, try creating it?");
                Err(e.into())
            }
        }
    }

    pub fn update_key(
        &self,
        nick: &str,
        user: &Option<String>,
        host: &Option<String>,
        port: &Option<u16>,
        genkey: &Option<bool>,
    ) -> bool {
        match crate::db::update_key(&self.conn, nick, user, host, port, genkey) {
            Ok(rows) => rows != 0,
            Err(_) => false,
        }
    }

    pub fn delete_key(&self, nick: &str) -> bool {
        match crate::db::del_key(&self.conn, nick) {
            Ok(rows) => rows != 0,
            Err(_) => false,
        }
    }

    pub fn show_all_keys(&self) -> Vec<ProcessedKey> {
        crate::db::get_all_keys(&self.conn).unwrap()
    }
}
