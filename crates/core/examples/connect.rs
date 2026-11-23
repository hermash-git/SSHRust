//! Test manuel de l'étape 1a : connexion par mot de passe avec empreinte d'hôte épinglée.
//!
//! Variables d'environnement :
//! - `SSHRUST_FINGERPRINT` (ou premier argument) : empreinte attendue, ex. `SHA256:...`
//! - `SSHRUST_PASSWORD` (défaut `sshrust-test`, serveur de test local uniquement)
//! - `SSHRUST_HOST` (défaut 127.0.0.1), `SSHRUST_PORT` (défaut 2222), `SSHRUST_USER` (défaut test)

use std::env;

use sshrust_core::client::{ConnectParams, SshSession};
use zeroize::Zeroizing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = env::var("SSHRUST_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = env::var("SSHRUST_PORT")
        .unwrap_or_else(|_| "2222".into())
        .parse()?;
    let user = env::var("SSHRUST_USER").unwrap_or_else(|_| "test".into());
    let fingerprint = env::var("SSHRUST_FINGERPRINT")
        .ok()
        .or_else(|| env::args().nth(1))
        .ok_or("empreinte manquante : SSHRUST_FINGERPRINT ou premier argument (SHA256:...)")?;
    let password =
        Zeroizing::new(env::var("SSHRUST_PASSWORD").unwrap_or_else(|_| "sshrust-test".into()));

    let params = ConnectParams {
        host: &host,
        port,
        user: &user,
        expected_fingerprint: &fingerprint,
    };

    let session = SshSession::connect_with_password(&params, &password).await?;
    println!("Authentification réussie sur {host}:{port}");
    session.disconnect().await?;
    Ok(())
}
