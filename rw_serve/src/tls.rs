use std::path::Path;

use anyhow::{Context, Result};
use tracing::warn;

pub fn ensure_certs(cert_path: &Path, key_path: &Path) -> Result<()> {
    if cert_path.exists() && key_path.exists() {
        return Ok(());
    }

    warn!("Certificate or key not found — generating self-signed certificate");
    warn!("Self-signed certificates are NOT suitable for production use");

    let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    let certified = rcgen::generate_simple_self_signed(subject_alt_names)
        .context("Failed to generate self-signed certificate")?;

    if let Some(parent) = cert_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create cert directory")?;
    }
    if let Some(parent) = key_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create key directory")?;
    }

    std::fs::write(cert_path, certified.cert.pem()).context("Failed to write certificate")?;
    std::fs::write(key_path, certified.signing_key.serialize_pem())
        .context("Failed to write private key")?;

    Ok(())
}
