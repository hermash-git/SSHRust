//! Test manuel de l'étape 1a : connexion par mot de passe avec empreinte d'hôte épinglée.
//!
//! Variables d'environnement :
//! - `SSHRUST_FINGERPRINT` (obligatoire) : empreinte attendue, ex. `SHA256:...`
//! - `SSHRUST_PASSWORD` (obligatoire) : mot de passe (serveur de test uniquement)
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
        .map_err(|_| "SSHRUST_FINGERPRINT manquante (format SHA256:...)")?;
    let password =
        Zeroizing::new(env::var("SSHRUST_PASSWORD").map_err(|_| "SSHRUST_PASSWORD manquante")?);

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
