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
- Étape 1 · Cœur SSH sans interface : terminée et validée.
- Étape en cours : 2a · Test de faisabilité du terminal (`alacritty_terminal` dans iced).
- Étape 1 réalisée : connexion par mot de passe avec empreinte d’hôte épinglée, shell interactif avec PTY et redimensionnement dans `crates/core` ; testé avec succès contre le conteneur Docker (`docker/compose.yml`).
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

Étape 2a : test de faisabilité du terminal, sans toucher au cœur SSH. Afficher une grille `alacritty_terminal` dans iced (texte coloré, curseur), alimentée par des octets de test, et mesurer la fluidité avec un `cat` d’un gros fichier. Si le rendu est impraticable dans iced, s’arrêter et étudier egui.

1. Vérifier la version, la licence et l’API d’`alacritty_terminal` avant d’écrire du code.
2. Utiliser `Term` et `vte::ansi::Processor` en mode headless, sans boucle d’événements.
3. Dessiner la grille dans un canvas iced et alimenter le moteur avec des octets de test.
4. Mesurer la fluidité avec un gros fichier.

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
