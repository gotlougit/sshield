use rusqlite::{params, Connection, Result};
use std::fmt::Display;

/// Row representation
pub(crate) struct StructuredKey {
    nickname: String,
    user: String,
    host: String,
    port: u16,
    encoded_key: Vec<u8>,
    cipher: String,
}

impl StructuredKey {
    pub fn get_key(&self) -> &[u8] {
        &self.encoded_key
    }
}

impl Display for StructuredKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SSH key '{}' using cipher '{}'",
            self.nickname, self.cipher
        )?;
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
                    encoded_key BLOB,
                    cipher VARCHAR
                )",
                [],
            )
            .unwrap();
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
    cipher: &str,
) -> bool {
    let db = open_db().unwrap();
    let mut prepstatement = db
        .prepare(
            "INSERT INTO keys (
        nickname, user, host, port, encoded_key, cipher
        ) VALUES (?1, ?2, ?3, ?4, (?5), ?6)",
        )
        .unwrap();
    match prepstatement.execute(params![
        nick,
        user,
        host,
        &port.to_string(),
        encoded_key,
        cipher
    ]) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Get the required key from the database
pub(crate) fn get_key(nick: &str) -> Result<StructuredKey> {
    let db = open_db()?;
    let mut prepstatement = db.prepare(
        "SELECT 
        nickname, user, host, port, encoded_key, cipher
        FROM keys WHERE nickname = ?1",
    )?;
    prepstatement.query_row([nick], |row| {
        Ok(StructuredKey {
            nickname: row.get(0)?,
            user: row.get(1)?,
            host: row.get(2)?,
            port: row.get(3)?,
            encoded_key: row.get(4)?,
            cipher: row.get(5)?,
        })
    })
}
