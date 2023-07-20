use async_trait::async_trait;
use russh::{client, ChannelId};
use russh_keys::{key, load_secret_key};
use std::env;
use std::sync::Arc;

struct Client {}

#[async_trait]
impl client::Handler for Client {
    type Error = anyhow::Error;

    async fn check_server_key(
        self,
        server_public_key: &key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        println!("check_server_key: {:?}", server_public_key);
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
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("USAGE: ssh-rs <user> <hostname> <key>");
        return;
    }
    let user = args[1].clone();
    let host = args[2].clone();
    let keyfile = args[3].clone();
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
            println!("{:?}", msg)
        }
    }
}
