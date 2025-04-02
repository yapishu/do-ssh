use clap::{ArgAction, Parser, Subcommand};
use iroh::NodeId;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub sub: Sub,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Sub {
    /// Start a server
    Server {
        /// The path to the keyfile to use
        #[arg(short = 'i', long = "key", default_value = "key.priv")]
        key_file: PathBuf,
        /// Create a new keyfile
        #[arg(short = 'n', long = "no-create", action = ArgAction::SetFalse)]
        create: bool,
    },
    /// Connect as a client
    Client {
        /// The iroh NodeId to connect to
        #[arg(required = true)]
        node_id: NodeId,
        /// The port of the ssh server
        #[arg(default_value_t = 22)]
        port: u16,
    },
    /// Generate a new private key (which implies the node id)
    Gen {
        /// Path to the keyfile to generate
        #[arg(short = 'i', short = 'k', long = "key", default_value = "key.priv")]
        key_file: PathBuf,
        /// Allow replacing an existing file
        #[arg(short = 'o', long = "override")]
        r#override: bool,
    },
}
