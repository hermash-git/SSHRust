//! Petit client SSH interactif (étape 1e) : relie le clavier et l'écran à un shell distant.
//!
//! Usage : `cargo run -p sshrust-core --example ssh_cli -- <empreinte> [key]`
//! - empreinte : `SHA256:...` (ou variable `SSHRUST_FINGERPRINT`)
//! - `key` en 2e argument : authentification par la clé de test `docker/test_key`
//!   (sinon mot de passe `sshrust-test`)
//!
//! Variables optionnelles : `SSHRUST_HOST`, `SSHRUST_PORT`, `SSHRUST_USER`, `SSHRUST_PASSWORD`.
//! Tape `exit` (ou Ctrl-D) pour fermer la session.

use std::env;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};
use sshrust_core::client::{ConnectParams, ShellEvent, SshSession};
use tokio::sync::mpsc;
use zeroize::Zeroizing;

/// Remet le terminal en mode normal quand il sort de portée, même en cas d'erreur.
struct RawModeGuard;

impl RawModeGuard {
    fn enable() -> std::io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

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
    let use_key = env::args().nth(2).as_deref() == Some("key");

    let params = ConnectParams {
        host: &host,
        port,
        user: &user,
        expected_fingerprint: &fingerprint,
    };

    let session = if use_key {
        let passphrase = Zeroizing::new("sshrust-key-pass".to_owned());
        SshSession::connect_with_key(&params, Path::new("docker/test_key"), Some(&passphrase))
            .await?
    } else {
        let password =
            Zeroizing::new(env::var("SSHRUST_PASSWORD").unwrap_or_else(|_| "sshrust-test".into()));
        SshSession::connect_with_password(&params, &password).await?
    };

    let exit_code = run_shell(&session).await?;
    session.disconnect().await?;
    println!("Connexion fermée (code de sortie : {exit_code})");
    Ok(())
}

/// Boucle interactive : clavier vers le serveur, sortie du serveur vers l'écran,
/// et suivi de la taille du terminal. Retourne le code de sortie du shell distant.
async fn run_shell(session: &SshSession) -> Result<u32, Box<dyn std::error::Error>> {
    let (cols, rows) = size()?;
    let mut shell = session.open_shell(cols.into(), rows.into()).await?;
    let _raw = RawModeGuard::enable()?;

    // Lecture bloquante de stdin dans un thread, relayée par un canal.
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut stdin = std::io::stdin();
        let mut buf = [0u8; 1024];
        while let Ok(n) = stdin.read(&mut buf) {
            if n == 0 || tx.send(buf[..n].to_vec()).is_err() {
                break;
            }
        }
    });

    let mut current_size = (cols, rows);
    let mut tick = tokio::time::interval(Duration::from_millis(500));
    let mut stdout = std::io::stdout();
    let mut exit_code = 0;

    loop {
        tokio::select! {
            Some(bytes) = rx.recv() => shell.write(&bytes).await?,
            event = shell.next_event() => match event {
                Some(ShellEvent::Output(bytes)) => {
                    stdout.write_all(&bytes)?;
                    stdout.flush()?;
                }
                Some(ShellEvent::ExitStatus(code)) => exit_code = code,
                Some(ShellEvent::Closed) | None => break,
            },
            _ = tick.tick() => match size() {
                Ok(now) if now != current_size => {
                    current_size = now;
                    shell.resize(now.0.into(), now.1.into()).await?;
                }
                _ => {}
            },
        }
    }
    Ok(exit_code)
}
