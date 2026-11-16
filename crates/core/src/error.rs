use thiserror::Error;

/// Erreurs du cœur SSH. Aucun message ne contient de secret (mot de passe, clé privée).
#[derive(Debug, Error)]
pub enum SshError {
    /// L'empreinte de la clé d'hôte ne correspond pas à celle attendue : connexion refusée.
    #[error("empreinte de l'hôte inattendue : attendue {expected}, reçue {actual}")]
    HostKeyMismatch { expected: String, actual: String },

    /// Le serveur a refusé l'authentification.
    #[error("authentification refusée par le serveur")]
    AuthRejected,

    /// Erreur du protocole ou du transport SSH.
    #[error("erreur SSH : {0}")]
    Ssh(#[from] russh::Error),
}
