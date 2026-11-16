use std::sync::{Arc, Mutex};

use russh::client::{self, Handle, Handler};
use russh::keys::{HashAlg, ssh_key::PublicKey};
use russh::{Channel, ChannelMsg, Disconnect};
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

    /// Ouvre un shell interactif : canal de session, PTY `xterm-256color`, puis shell distant.
    pub async fn open_shell(&self, cols: u32, rows: u32) -> Result<ShellChannel, SshError> {
        let channel = self.handle.channel_open_session().await?;
        channel
            .request_pty(false, "xterm-256color", cols, rows, 0, 0, &[])
            .await?;
        channel.request_shell(false).await?;
        Ok(ShellChannel { channel })
    }

    /// Ferme proprement la session.
    pub async fn disconnect(&self) -> Result<(), SshError> {
        self.handle
            .disconnect(Disconnect::ByApplication, "", "en")
            .await?;
        Ok(())
    }
}

/// Évènement émis par un shell distant.
#[derive(Debug)]
pub enum ShellEvent {
    /// Octets de sortie du terminal distant (séquences d'échappement comprises).
    Output(Vec<u8>),
    /// Code de sortie du shell distant.
    ExitStatus(u32),
    /// Le canal est fermé par le serveur.
    Closed,
}

/// Canal d'un shell interactif avec PTY.
pub struct ShellChannel {
    channel: Channel<client::Msg>,
}

impl ShellChannel {
    /// Envoie des octets (frappes clavier, collage) au shell distant.
    pub async fn write(&self, bytes: &[u8]) -> Result<(), SshError> {
        self.channel.data(bytes).await?;
        Ok(())
    }

    /// Informe le serveur d'un redimensionnement du terminal.
    pub async fn resize(&self, cols: u32, rows: u32) -> Result<(), SshError> {
        self.channel.window_change(cols, rows, 0, 0).await?;
        Ok(())
    }

    /// Attend le prochain évènement ; `None` quand le canal est entièrement terminé.
    pub async fn next_event(&mut self) -> Option<ShellEvent> {
        loop {
            match self.channel.wait().await? {
                ChannelMsg::Data { data } | ChannelMsg::ExtendedData { data, .. } => {
                    return Some(ShellEvent::Output(data.to_vec()));
                }
                ChannelMsg::ExitStatus { exit_status } => {
                    return Some(ShellEvent::ExitStatus(exit_status));
                }
                ChannelMsg::Eof | ChannelMsg::Close => return Some(ShellEvent::Closed),
                _ => {}
            }
        }
    }
}
