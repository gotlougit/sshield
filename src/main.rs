use clap::Parser;
use cli::{Args, Command};
use db::ProcessedKey;
use rusqlite::Connection;
use russh_keys::{key::KeyPair, pkcs8};

mod cli;
mod db;

struct KeyMgr {
    db: Connection,
}

impl KeyMgr {
    fn init() -> Self {
        KeyMgr {
            db: crate::db::open_db().unwrap(),
        }
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

fn main() {
    let mgr = KeyMgr::init();
    let args = Args::parse();
    match args.command {
        Some(cmd) => {
            match cmd {
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
