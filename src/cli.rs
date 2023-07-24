use clap::{Parser, Subcommand};

#[derive(Subcommand, PartialEq)]
pub enum Command {
    /// Generate a key for a particular service
    GenKey {
        /// key nickname
        #[arg(required = true)]
        name: String,
        /// Username
        #[arg(required = true)]
        user: String,
        /// Hostname
        #[arg(required = true)]
        host: String,
        /// Port (optional)
        #[arg(short, long, default_value_t = 22)]
        port: u16,
    },
    /// Show a key for a particular service,
    /// if no name is given, return all keys
    ShowKey {
        /// key nickname
        #[arg(required = false)]
        name: Option<String>,
    },
    /// Delete a key for a particular service
    DeleteKey {
        /// key nickname
        #[arg(required = true)]
        name: String,
    },
    /// Create a new key for a pre-existing service
    UpdateKey {
        /// key nickname
        #[arg(required = true)]
        name: String,
        /// Username
        #[arg(long, required = false)]
        user: Option<String>,
        /// Hostname
        #[arg(long, required = false)]
        host: Option<String>,
        /// Port (optional)
        #[arg(long, required = false)]
        port: Option<u16>,
        /// Generate new key (if needed)
        #[arg(long, required = false)]
        genkey: Option<bool>,
    },
    /// Start listener process and handle remote requests for keys
    Serve {},
    /// Add all keys into server
    AddKeysToServer {},
}

/// Rust reimplementation of SSH client with sandboxing
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}
