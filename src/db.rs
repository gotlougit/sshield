use rusqlite::{params, Connection, Result};
use russh_keys::{key::KeyPair, PublicKeyBase64};
use std::fmt::Display;

/// Row representation
struct RawKey {
    nickname: String,
    user: String,
    host: String,
    port: u16,
    encoded_key: Vec<u8>,
}

pub(crate) struct ProcessedKey {
    pub nickname: String,
    pub user: String,
    pub host: String,
    pub port: u16,
    pub keypair: KeyPair,
    pub pubkey: String,
    pub cipher: String,
}

impl RawKey {
    pub fn process(&self) -> ProcessedKey {
        let keypair = russh_keys::pkcs8::decode_pkcs8(&self.encoded_key, None).unwrap();
        let pubkey = keypair.clone_public_key().unwrap().public_key_base64();
        let cipher = keypair.name().to_string();
        ProcessedKey {
            nickname: self.nickname.clone(),
            user: self.user.clone(),
            host: self.host.clone(),
            port: self.port,
            keypair,
            pubkey,
            cipher,
        }
    }
}

impl Display for ProcessedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SSH key '{}' for {}@{}:{} using cipher '{}'",
            self.nickname, self.user, self.host, self.port, self.cipher
        )?;
        writeln!(f, "{} {} {}", self.cipher, self.pubkey, self.nickname)?;
        Ok(())
    }
}

/// Opens the database and creates the table if necessary
pub(crate) fn open_db() -> Result<Connection> {
    match Connection::open("./keys.db3") {
        Ok(conn) => {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS keys (
                    nickname VARCHAR PRIMARY KEY,
                    user VARCHAR,
                    host VARCHAR,
                    port INT,
                    encoded_key BLOB
                )",
                [],
            )?;
            Ok(conn)
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn insert_key<'a>(
    nick: &str,
    user: &str,
    host: &str,
    port: u16,
    encoded_key: Vec<u8>,
) -> bool {
    let db = open_db().unwrap();
    let mut prepstatement = db
        .prepare(
            "INSERT INTO keys (
        nickname, user, host, port, encoded_key
        ) VALUES (?1, ?2, ?3, ?4, (?5))",
        )
        .unwrap();
    match prepstatement.execute(params![nick, user, host, &port.to_string(), encoded_key]) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Get the required key from the database
pub(crate) fn get_key(nick: &str) -> Result<ProcessedKey> {
    let db = open_db()?;
    let mut prepstatement = db.prepare(
        "SELECT 
        nickname, user, host, port, encoded_key
        FROM keys WHERE nickname = ?1",
    )?;
    prepstatement.query_row([nick], |row| {
        let row = RawKey {
            nickname: row.get(0)?,
            user: row.get(1)?,
            host: row.get(2)?,
            port: row.get(3)?,
            encoded_key: row.get(4)?,
        };
        Ok(row.process())
    })
}

/// Get all the keys from the database
pub(crate) fn get_all_keys() -> Result<Vec<ProcessedKey>, rusqlite::Error> {
    let db = open_db()?;
    let mut prepstatement = db.prepare(
        "SELECT
        nickname, user, host, port, encoded_key
        FROM keys",
    )?;
    let rows: Result<Vec<ProcessedKey>, rusqlite::Error> = prepstatement
        .query_map([], |row| {
            let row = RawKey {
                nickname: row.get(0)?,
                user: row.get(1)?,
                host: row.get(2)?,
                port: row.get(3)?,
                encoded_key: row.get(4)?,
            };
            Ok(row.process())
        })?
        .collect();
    rows
}
