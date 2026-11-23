# SSHRUST

Client SSH desktop en Rust, inspiré de Termius, avec interface native en `iced`, moteur terminal en Rust et logique métier découplée de l’UI.

## Objectif

Construire un client SSH moderne pour desktop, simple et fiable :

- interface native en Rust (`iced`),
- terminal robuste basé sur `alacritty_terminal`,
- accès SSH via `russh`,
- stockage des hôtes en TOML,
- coffre chiffré pour les secrets,
- support de connexion à un serveur SSH de test Docker avant de passer à l’étape suivante.

## Arborescence

```text
SSHRust/
├── Cargo.toml
├── claude.md
├── README.md
├── rustfmt.toml
├── crates/
│   ├── app/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── theme.rs
│   ├── core/
│   │   ├── Cargo.toml
│   │   ├── examples/
│   │   │   ├── connect.rs
│   │   │   └── shell_echo.rs
│   │   └── src/
│   │       ├── client.rs
│   │       ├── error.rs
│   │       └── lib.rs
│   ├── terminal/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   └── vault/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
└── target/
```

## Stack technique

- Interface : `iced` v0.14.0
- SSH : `russh`, `tokio`
- Terminal : `alacritty_terminal` + `vte::ansi::Processor`
- Authentification SSH / config : `ssh2-config`
- Secrets : `age` ou `argon2 + chacha20poly1305`, `zeroize`
- Stockage hôtes : `serde + toml`
- Chemins d’applications : `directories`
- Presse-papiers : `arboard`
- Logs / erreurs : `thiserror`, `anyhow`, `tracing`

## État d’avancement

- Étape 0 · Préparation : terminée.
- Étape en cours : 1 · Cœur SSH sans interface (non validée).
- Étape 1 codée en partie : connexion par mot de passe avec empreinte d’hôte épinglée, shell interactif avec PTY et redimensionnement dans `crates/core` ; testé avec succès contre le conteneur Docker (`docker/compose.yml`).
- Serveur SSH Docker local : connexion par mot de passe et par clé (avec passphrase) confirmées.
- `cargo build --workspace` : réussi.
- `cargo clippy --workspace --all-targets -- -D warnings` : réussi.
- `cargo fmt --all -- --check` : réussi.

Le serveur de test est censé écouter sur `127.0.0.1:2222` dans le conteneur `sshrust-test`. Ne l’utilise que pour les tests locaux.

## Validation de l’étape 0

Les critères de fin sont :

- `cargo build --workspace` passe,
- `cargo clippy --workspace --all-targets -- -D warnings` passe,
- un serveur SSH Docker local répond bien,
- un accès par mot de passe puis par clé fonctionne avec `ssh` standard,
- le profil visuel de base est prêt dans `crates/app/src/theme.rs`.

## Commandes utiles

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo run -p sshrust-app
# Tests manuels de l’étape 1 (empreinte en 1er argument ou via SSHRUST_FINGERPRINT, voir docker/compose.yml)
# Exemple : cargo run -p sshrust-core --example connect -- SHA256:...
cargo run -p sshrust-core --example connect
cargo run -p sshrust-core --example shell_echo
cargo run -p sshrust-core --example connect_key
cargo run -p sshrust-core --example ssh_cli -- <empreinte> [key]
```

## Plan de travail concret

## Prochaine tâche concrète

Étape 1 dans `crates/core`, sans dépendance à iced (points 1 et 2 codés ; point 3 en partie avec l’exemple `shell_echo`, sans CLI interactif ; point 4 à faire ; test réel réussi) :

1. Vérifier la version et l’API de `russh` appropriées avant d’écrire du code.
2. Définir les types d’erreurs et une API minimale de connexion par mot de passe.
3. Ajouter un petit client CLI de test et obtenir un shell interactif avec pseudo-terminal sur le conteneur local.
4. Ajouter ensuite l’authentification par clé Ed25519 avec passphrase, le redimensionnement et le keepalive.

Tester exclusivement contre le conteneur local. Les essais de connexion Rust et la validation de la clé d’hôte font partie de l’étape 1 ; ne pas accepter silencieusement une empreinte en production.

## Sécurité

- Ne jamais logger un mot de passe, une clé privée ou un contenu de coffre.
- Utiliser des types qui effacent les secrets (`zeroize` / `Zeroizing`).
- Ne pas écrire de cryptographie maison.
- Empreinte d’hôte : confirmation au premier contact, blocage si elle change.
- Rester sur un serveur Docker de test, jamais un vrai serveur de production.

## Risques à surveiller

- `iced` a une API qui évolue ; il faut rester sur la version fixée dans le workspace.
- Le moteur terminal est le point de risque technique principal ; on le fait en tête en priorité.
- Le SSH côté `russh` est sensible aux détails de PTY / keepalive / redimensionnement.
