use crate::db::{self, ProcessedKey};
use anyhow::Result;
use async_trait::async_trait;
use rusqlite::Connection;
use russh_keys::{agent::client, agent::server, agent::server::MessageType, key::KeyPair, pkcs8};
use std::future::Future;
use std::sync::Arc;
use tokio::fs;
use tokio::net::{UnixListener, UnixStream};

const SOCKNAME: &str = "/tmp/ssh-agent-2";

#[derive(Clone)]
struct SecureAgent {}

#[async_trait]
impl server::Agent for SecureAgent {
    fn confirm(self, _pk: Arc<KeyPair>) -> Box<dyn Future<Output = (Self, bool)> + Unpin + Send> {
        Box::new(futures::future::ready((self, true)))
    }
    async fn confirm_request(&self, msg: MessageType) -> bool {
        true
    }
}

pub async fn start_server() {
    match UnixListener::bind(SOCKNAME) {
        Ok(listener) => {
            let wrapper = tokio_stream::wrappers::UnixListenerStream::new(listener);
            server::serve(wrapper, SecureAgent {}).await.unwrap();
        }
        Err(e) => {
            eprintln!("Error while starting agent server: {}", e);
        }
    }
}

pub struct Socket {
    conn: Connection,
    // FIXME: DO NOT AND I REPEAT DO NOT
    // HAVE THIS BE PLAINTEXT, EVEN IN MEMORY!
    // PROTECT THESE BITS AT ALL COSTS!
    pass: String,
}

impl Socket {
    pub fn init(pass: &str) -> Result<Self> {
        let conn = db::open_db()?;
        // Here we would ideally place some decryption mechanisms to handle
        // sensitive key data
        Ok(Self {
            conn,
            pass: pass.to_string(),
        })
    }

    pub async fn add_all_keys(&self) {
        // Add keys to server automatically
        // This is done by creating a dummy client that adds all the keys we have
        let keys = self.show_all_keys();
        match UnixStream::connect(SOCKNAME).await {
            Ok(stream) => {
                let mut dummyclient = client::AgentClient::connect(stream);
                for key in keys.iter() {
                    println!("adding key {}", key.nickname);
                    let pair = key.keypair.clone();
                    dummyclient.add_identity(&pair, &[]).await.unwrap();
                }
            }
            Err(_) => eprintln!("Couldn't connect to agent, is it running?"),
        }
    }

    // TODO: make this cryptographically secure
    fn auth_req(&self, proposed_pass: &str) -> bool {
        proposed_pass == self.pass
    }

    pub fn gen_key(&self, nick: &str, user: &str, host: &str, port: u16) -> bool {
        let key = KeyPair::generate_ed25519().unwrap();
        // store this encoded key in db
        let encoded_key = pkcs8::encode_pkcs8(&key);
        crate::db::insert_key(&self.conn, nick, user, host, port, encoded_key)
    }

    pub fn show_key(&self, nick: &str) -> Result<ProcessedKey> {
        match crate::db::get_key(&self.conn, nick) {
            Ok(res) => {
                return Ok(res);
            }
            Err(e) => {
                println!("That key doesn't exist, try creating it?");
                return Err(e.into());
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

    pub async fn close(&self) {
        fs::remove_file(SOCKNAME).await.unwrap();
    }
}
