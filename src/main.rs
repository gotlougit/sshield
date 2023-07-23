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
                    mgr.gen_key(&name, &user, &host, port);
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
                _ => {
                    println!("hello");
                }
            };
        }
        None => {}
    };
    mgr.close();
}
