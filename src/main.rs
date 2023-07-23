use crate::socket::Socket;
use clap::Parser;
use cli::{Args, Command};

mod cli;
mod db;
mod socket;

fn main() {
    let mgr = Socket::init("").unwrap();
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
                    false => println!("Key could not be deleted"),
                },
                _ => {
                    println!("hello");
                }
            };
        }
        None => {}
    };
    mgr.close();
}
