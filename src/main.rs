use async_trait::async_trait;
use futures::Future;
use russh::server::{Auth, Session};
use russh::*;
use russh_keys::*;
use std::io::Read;
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
    let config = russh::client::Config::default();
    let config = Arc::new(config);
    let sh = Client {};

    let key = russh_keys::key::KeyPair::generate_ed25519().unwrap();
    let mut agent = russh_keys::agent::client::AgentClient::connect_env()
        .await
        .unwrap();
    agent.add_identity(&key, &[]).await.unwrap();
    let mut session = russh::client::connect(config, ("127.0.0.1", 22), sh)
        .await
        .unwrap();
    if session
        .authenticate_future(
            std::env::var("USER").unwrap_or("user".to_owned()),
            key.clone_public_key().unwrap(),
            agent,
        )
        .await
        .1
        .unwrap()
    {
        let mut channel = session.channel_open_session().await.unwrap();
        channel.data(&b"Hello, world!"[..]).await.unwrap();
        if let Some(msg) = channel.wait().await {
            println!("{:?}", msg)
        }
    }
}
