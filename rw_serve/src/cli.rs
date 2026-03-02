use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "rw_serve", about = "Static multi-app web server")]
pub struct Args {
    /// Directory containing app subdirectories (each becomes /<name>/)
    #[arg(short = 'd', long, default_value = "./dist")]
    pub apps_dir: PathBuf,

    /// Port to listen on (defaults: 8080 HTTP, 8443 HTTPS)
    #[arg(short = 'p', long)]
    pub port: Option<u16>,

    /// Enable HTTPS mode
    #[arg(long)]
    pub https: bool,

    /// Path to TLS certificate in PEM format
    #[arg(long, default_value = "./certs/cert.pem")]
    pub cert: PathBuf,

    /// Path to TLS private key in PEM format
    #[arg(long, default_value = "./certs/key.pem")]
    pub key: PathBuf,

    /// Emit logs as newline-delimited JSON instead of human-readable text
    #[arg(long)]
    pub log_json: bool,
}
