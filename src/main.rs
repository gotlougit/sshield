use std::process::exit;

use crate::socket::Client;
use clap::Parser;
use cli::{Args, Command};

mod cli;
mod db;
mod gui;
mod socket;

#[tokio::main]
async fn main() {
    let mgr = Client::init("").unwrap();
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
                    match mgr.gen_key(&name, &user, &host, port) {
                        true => println!("{}", mgr.show_key(&name).unwrap()),
                        false => println!("Error, key wasn't able to be created"),
                    };
                }
                Command::ShowKey { name } => match name {
                    Some(name) => {
                        mgr.show_key(&name).unwrap();
                    }
                    None => {
                        let keys = mgr.show_all_keys();
                        for key in keys.iter() {
                            println!("{key}");
                        }
                    }
                },
                Command::DeleteKey { name } => match mgr.delete_key(name.as_str()) {
                    true => println!("Key deleted"),
                    false => println!("Key could not be deleted, maybe it does not exist?"),
                },
                Command::Serve {} => {
                    tokio::spawn(async move {
                        tokio::signal::ctrl_c().await.unwrap();
                        println!("Closing socket...");
                        socket::close_socket().await;
                        exit(0);
                    });
                    println!("Starting server process...");
                    let task1 = socket::start_server();
                    let task2 = {
                        // In order to prevent any funny business with the scheduler,
                        // let's wait for a while and let the other task take over
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        mgr.add_all_keys()
                    };
                    tokio::join!(task1, task2);
                }
                Command::UpdateKey {
                    name,
                    user,
                    host,
                    port,
                    genkey,
                } => {
                    match mgr.update_key(&name, &user, &host, &port, &genkey) {
                        true => println!("Updated key successfully"),
                        false => eprintln!("Couldn't update key, does it exist?"),
                    };
                }
            };
        }
        None => {}
    };
}
