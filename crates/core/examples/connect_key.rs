//! Test manuel de l'étape 1c : connexion par clé privée avec passphrase.
//!
//! Arguments : `[empreinte] [passphrase]`
//! - empreinte : `SHA256:...` (ou variable `SSHRUST_FINGERPRINT`)
//! - passphrase : défaut `sshrust-key-pass` (clé de test jetable `docker/test_key`)
//!
//! Variables optionnelles : `SSHRUST_HOST`, `SSHRUST_PORT`, `SSHRUST_USER`, `SSHRUST_KEY`.

use std::env;
use std::path::PathBuf;

use sshrust_core::client::{ConnectParams, SshSession};
use zeroize::Zeroizing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = env::var("SSHRUST_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = env::var("SSHRUST_PORT")
        .unwrap_or_else(|_| "2222".into())
        .parse()?;
    let user = env::var("SSHRUST_USER").unwrap_or_else(|_| "test".into());
    let key_path =
        PathBuf::from(env::var("SSHRUST_KEY").unwrap_or_else(|_| "docker/test_key".into()));
    let fingerprint = env::var("SSHRUST_FINGERPRINT")
        .ok()
        .or_else(|| env::args().nth(1))
        .ok_or("empreinte manquante : SSHRUST_FINGERPRINT ou premier argument (SHA256:...)")?;
    let passphrase = Zeroizing::new(
        env::args()
            .nth(2)
            .unwrap_or_else(|| "sshrust-key-pass".into()),
    );

    let params = ConnectParams {
        host: &host,
        port,
        user: &user,
        expected_fingerprint: &fingerprint,
    };

    let session = SshSession::connect_with_key(&params, &key_path, Some(&passphrase)).await?;
    println!("Authentification par clé réussie sur {host}:{port}");
    session.disconnect().await?;
    Ok(())
}
