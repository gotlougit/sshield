use async_trait::async_trait;
use clap::Parser;
use russh::{client, ChannelId};
use russh_keys::{key, load_secret_key};
use std::sync::Arc;

/// Rust reimplementation of SSH client with sandboxing
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Username
    #[arg(long)]
    user: String,
    /// Hostname
    #[arg(long)]
    host: String,
    /// SSH keyfile to use
    #[arg(long)]
    keyfile: String,
}

struct Client {}

#[async_trait]
impl client::Handler for Client {
    type Error = anyhow::Error;

    async fn check_server_key(
        self,
        server_public_key: &key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        println!("check_server_key: {:#?}", server_public_key);
        Ok((self, true))
    }

    async fn data(
        self,
        channel: ChannelId,
        data: &[u8],
        session: client::Session,
    ) -> Result<(Self, client::Session), Self::Error> {
        println!(
            "data on channel {:?}: {:?}",
            channel,
            std::str::from_utf8(data)
        );
        Ok((self, session))
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let user = args.user;
    let host = args.host;
    let keyfile = args.keyfile;
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
            let output = format!("{:#?}", msg);
            if output != "Success" {
                println!("{output}");
            }
        }
    }
}
