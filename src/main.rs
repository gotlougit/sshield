use crate::config::Config;
use crate::socket::Client;
use clap::Parser;
use cli::{Args, Command};
use std::process::exit;

mod cli;
mod config;
mod db;
mod gui;
mod socket;

/// Allow only local file access, and no socket access
fn restrict_net() {
    extrasafe::SafetyContext::new()
        .enable(extrasafe::builtins::Networking::nothing())
        .unwrap()
        .enable(
            extrasafe::builtins::SystemIO::nothing()
                .allow_close()
                .allow_open()
                .yes_really()
                .allow_read()
                .allow_write()
                .allow_unlink()
                .allow_ioctl()
                .allow_metadata(),
        )
        .unwrap()
        .apply_to_all_threads()
        .unwrap();
}

/// Allow only socket access and restricted file access
fn restrict_file() {
    extrasafe::SafetyContext::new()
        .enable(
            extrasafe::builtins::Networking::nothing()
                // This allows `connect()` syscalls
                // TODO: Wait for https://github.com/boustrophedon/extrasafe/pull/25
                // to be merged in order to use `allow_connect()`
                .allow_start_tcp_clients()
                .allow_running_unix_clients()
                .allow_running_unix_servers()
                .allow_start_unix_servers()
                .yes_really(),
        )
        .unwrap()
        // TODO: actually restrict file access
        .enable(extrasafe::builtins::SystemIO::everything())
        .unwrap()
        // Allows tokio's Ctrl+C handler to work
        .enable(extrasafe::builtins::danger_zone::Threads::nothing().allow_create())
        .unwrap()
        .apply_to_all_threads()
        .unwrap();
}

#[tokio::main]
async fn main() {
    let Config { db_path, prompt } = crate::config::get_all_vars();
    let args = Args::parse();
    if let Some(cmd) = args.command {
        let pass = crate::config::get_pass();
        match Client::init(pass.as_str(), &db_path) {
            Ok(mgr) => {
                match cmd {
                    Command::ChangePassword {} => {
                        std::mem::drop(mgr);
                        let newpass = crate::gui::get_new_db_pass();
                        match crate::db::change_db_pass(&db_path, &pass, &newpass) {
                            Ok(_) => println!("Password of database changed successfully!"),
                            Err(_) => eprintln!("Password was unable to be changed!"),
                        };
                    }
                    Command::GenKey {
                        name,
                        user,
                        host,
                        port,
                    } => {
                        match mgr.gen_key(&name, &user, &host, port).await {
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
                        let task1 = socket::start_server(prompt);
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
                    Command::ImportKey { name, path, pass } => {
                        match mgr.import_key_from_file(pass, &name, &path).await {
                            true => println!("Key imported successfully"),
                            false => eprintln!("Couldn't import key, check path or config?"),
                        }
                    }
                };
            }
            Err(_) => {
                crate::config::delete_pass_from_keyring();
                eprintln!("Check database password or location, unable to open database");
            }
        };
    };
}
