use std::sync::{Arc, Mutex};

use russh::Disconnect;
use russh::client::{self, Handle, Handler};
use russh::keys::{HashAlg, ssh_key::PublicKey};
use zeroize::Zeroizing;

use crate::error::SshError;

/// Paramètres d'une connexion SSH.
pub struct ConnectParams<'a> {
    pub host: &'a str,
    pub port: u16,
    pub user: &'a str,
    /// Empreinte attendue de la clé d'hôte, au format `SHA256:...` (celui de `ssh-keygen -lf`).
    pub expected_fingerprint: &'a str,
}

/// Handler qui n'accepte la clé d'hôte que si son empreinte SHA256 est celle attendue.
/// Jamais d'acceptation silencieuse : toute autre clé est refusée.
struct PinnedKeyHandler {
    expected: String,
    /// Empreinte reçue en cas de refus, pour construire une erreur claire.
    rejected: Arc<Mutex<Option<String>>>,
}

impl Handler for PinnedKeyHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        let actual = server_public_key.fingerprint(HashAlg::Sha256).to_string();
        if actual == self.expected {
            return Ok(true);
        }
        if let Ok(mut slot) = self.rejected.lock() {
            *slot = Some(actual);
        }
        Ok(false)
    }
}

/// Session SSH authentifiée.
pub struct SshSession {
    handle: Handle<PinnedKeyHandler>,
}

impl SshSession {
    /// Se connecte, vérifie l'empreinte de l'hôte puis s'authentifie par mot de passe.
    pub async fn connect_with_password(
        params: &ConnectParams<'_>,
        password: &Zeroizing<String>,
    ) -> Result<Self, SshError> {
        let rejected = Arc::new(Mutex::new(None));
        let handler = PinnedKeyHandler {
            expected: params.expected_fingerprint.to_owned(),
            rejected: Arc::clone(&rejected),
        };
        let config = Arc::new(client::Config::default());

        let mut handle = match client::connect(config, (params.host, params.port), handler).await {
            Ok(handle) => handle,
            Err(err) => {
                let actual = rejected.lock().ok().and_then(|mut slot| slot.take());
                return Err(match actual {
                    Some(actual) => SshError::HostKeyMismatch {
                        expected: params.expected_fingerprint.to_owned(),
                        actual,
                    },
                    None => SshError::Ssh(err),
                });
            }
        };

        let result = handle
            .authenticate_password(params.user, password.as_str())
            .await?;
        if !result.success() {
            return Err(SshError::AuthRejected);
        }
        Ok(Self { handle })
    }

    /// Ferme proprement la session.
    pub async fn disconnect(&self) -> Result<(), SshError> {
        self.handle
            .disconnect(Disconnect::ByApplication, "", "en")
            .await?;
        Ok(())
    }
}
