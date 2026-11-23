//! Test manuel de l'étape 1b : ouvre un shell avec PTY, envoie une commande, affiche la sortie.
//!
//! Mêmes variables d'environnement que l'exemple `connect`.

use std::env;
use std::io::Write;

use sshrust_core::client::{ConnectParams, ShellEvent, SshSession};
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
    let mut shell = session.open_shell(120, 30).await?;
    shell.write(b"echo hello-sshrust; exit\n").await?;

    let mut stdout = std::io::stdout();
    while let Some(event) = shell.next_event().await {
        match event {
            ShellEvent::Output(bytes) => {
                stdout.write_all(&bytes)?;
                stdout.flush()?;
            }
            ShellEvent::ExitStatus(code) => println!("\n[code de sortie : {code}]"),
            ShellEvent::Closed => break,
        }
    }

    session.disconnect().await?;
    Ok(())
}
