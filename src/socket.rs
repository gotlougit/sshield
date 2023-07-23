use crate::db::{self, ProcessedKey};
use anyhow::Result;
use rusqlite::Connection;
use russh_keys::{key::KeyPair, pkcs8};
use std::{fs, os::unix::net::UnixListener};

const SOCKNAME: &str = "/tmp/ssh-agent-2";

pub struct Socket {
    listener: UnixListener,
    conn: Connection,
    // FIXME: DO NOT AND I REPEAT DO NOT
    // HAVE THIS BE PLAINTEXT, EVEN IN MEMORY!
    // PROTECT THESE BITS AT ALL COSTS!
    pass: String,
}

impl Socket {
    pub fn init(pass: &str) -> Result<Self> {
        let listener = UnixListener::bind(SOCKNAME)?;
        let conn = db::open_db()?;
        Ok(Self {
            listener,
            conn,
            pass: pass.to_string(),
        })
    }

    // TODO: make this cryptographically secure
    fn auth_req(&self, proposed_pass: &str) -> bool {
        proposed_pass == self.pass
    }

    pub fn gen_key(&self, nick: &str, user: &str, host: &str, port: u16) {
        let key = KeyPair::generate_ed25519().unwrap();
        // store this encoded key in db
        let encoded_key = pkcs8::encode_pkcs8(&key);
        crate::db::insert_key(&self.conn, nick, user, host, port, encoded_key);
        self.show_key(nick).unwrap();
    }

    pub fn show_key(&self, nick: &str) -> Result<ProcessedKey> {
        match crate::db::get_key(&self.conn, nick) {
            Ok(res) => {
                println!("{res}");
                return Ok(res);
            }
            Err(e) => {
                println!("That key doesn't exist, try creating it?");
                return Err(e.into());
            }
        }
    }

    pub fn show_all_keys(&self) -> Vec<ProcessedKey> {
        crate::db::get_all_keys(&self.conn).unwrap()
    }

    pub fn close(&self) {
        fs::remove_file(SOCKNAME).unwrap();
    }
}
