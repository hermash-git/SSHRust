# SSHRUST — Instructions pour l'assistant de code

## Comment tu dois travailler

Règles de collaboration (obligatoires) :

- Réponds en français, de façon concise et concrète.
- Travaille UNE étape de la roadmap à la fois. Ne passe pas à la suivante sans que je dise « étape validée ».
- Avant de modifier un fichier, demande-moi son contenu actuel (ou lis-le), puis propose le plus petit changement possible.
- Ne réécris jamais un fichier entier si un petit diff suffit. Ne supprime rien sans me le dire. Préserve mes sauvegardes et ne touche pas à ce que je n'ai pas demandé.
- Explique en 2 à 3 phrases le POURQUOI de chaque choix non évident.
- Si tu hésites, si une API t'est inconnue ou si tu n'es pas sûr d'une version, dis-le au lieu d'inventer. Les API d'iced, de russh et de keyring ont changé récemment : vérifie la doc de la version installée (`cargo doc`, docs.rs) avant d'écrire du code.
- Avant de proposer une crate, vérifie sa version (`cargo add`, docs.rs), sa licence et si elle est maintenue.
- N'ajoute pas de dépendance sans me l'annoncer et justifier en une phrase.
- Chaque étape se termine par des tests ou une vérification manuelle que je peux lancer. Donne-moi la commande exacte.

## État actuel (à mettre à jour par moi)

- Étape terminée : 0 · Préparation
- Étape en cours : 1 · Cœur SSH sans interface (codée en partie, non validée)
- Ce qui marche : workspace Cargo et crates app/core/terminal/vault présents ; iced fixé à 0.14.0 ; palette dans `crates/app/src/theme.rs` ; README ajouté ; `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` et `cargo fmt --all -- --check` passent ; serveur SSH Docker local (`docker/compose.yml`, utilisateur `test`) validé par test réel en connexion par mot de passe avec empreinte épinglée et en shell avec PTY ; la clé de test (`docker/test_key`, non versionnée) est configurée sur le conteneur
- Ce qui reste à faire : étape 1 — déjà codés dans `crates/core` : connexion par mot de passe avec empreinte d’hôte épinglée, shell interactif avec PTY et redimensionnement (exemples `connect` et `shell_echo`, testés avec succès contre le conteneur Docker) ; déjà faits aussi : authentification par clé avec passphrase (exemple `connect_key`, testée) et keepalive configuré (30 s, 3 échecs ; comportement lors d’une coupure non testé) ; reste : valider le petit CLI interactif `ssh_cli` (écrit, à tester dans un vrai terminal : mode raw, redimensionnement, fermeture propre)
- Règle de suivi : mettre à jour `README.md` et cet état à chaque étape validée ; ne pas passer à l’étape suivante sans confirmation « étape validée »

## Le projet

Client SSH desktop « SSHRUST », inspiré de Termius : simple, moderne, intuitif, thème sombre puis clair. 100 % Rust, interface comprise.

Interdits : Tauri, webview, JavaScript, xterm.js, bibliothèques C pour le SSH (pas de libssh2, pas de crate `ssh2`).

## Pile technique (décidée)

- Interface : **iced** (version fixée dans Cargo.toml, ne pas la changer sans me demander)
- Terminal : **alacritty_terminal** en mode « headless » (`Term` + `vte::ansi::Processor`, sans boucle d'événements ni rendu), avec un widget d'affichage écrit par moi en iced (canvas). `iced_term` existe mais lance un shell local et son API est instable : s'en inspirer, ne pas en dépendre aveuglément.
- SSH : **russh** (+ **russh-sftp** pour le SFTP), **tokio**
- Config SSH : **ssh2-config** pour lire `~/.ssh/config`
- Coffre : **age** (avec scrypt) OU **argon2 + chacha20poly1305** (à trancher à l'étape 5), plus **zeroize**
- Trousseau système (optionnel) : écosystème **keyring** (keyring-core + crates par système). Lire la doc actuelle, l'API a été réorganisée.
- Stockage des hôtes : **serde + toml** (rusqlite seulement si le besoin apparaît)
- Chemins de configuration : **directories**
- Presse-papiers : **arboard**
- Erreurs et logs : **thiserror**, **anyhow** (application uniquement), **tracing**
- Installeurs : **cargo-packager** (MSI/NSIS, DMG/.app, deb/AppImage)

## Organisation du workspace Cargo

- `crates/core` : SSH, sessions, known_hosts, SFTP. AUCUNE dépendance à iced. Testable en ligne de commande.
- `crates/vault` : coffre chiffré, secrets, verrouillage
- `crates/terminal` : moteur de terminal (alacritty_terminal) + widget iced
- `crates/app` : application iced (écrans, navigation, thème)

Règle : le code métier ne dépend jamais de l'interface ; l'interface appelle le cœur via des messages.

## Commandes utiles

```
cargo build
cargo run -p app
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

## Design (déjà terminé dans FigCraft)

Fichier : `D:\Documents\ArtCraft\Figcraft\SSHRUST-design.figcraft`, 14 écrans de 1440x900. Je te décrirai ou te montrerai l'écran concerné quand on y sera.

- 00 Design system
- 01 Espace de travail (barre latérale, onglets, deux terminaux côte à côte, panneau de ressources)
- 02 Palette de commandes (Ctrl K)
- 03 Hôtes (cartes groupées)
- 04 Nouvel hôte (formulaire)
- 05 SFTP
- 06 Tunnels
- 07 Snippets
- 08 Trousseau
- 09 Sécurité et journal
- 10 Équipe et synchronisation
- 11 Réglages
- 12 Verrouillage
- 13 Alerte d'empreinte

Navigation : barre latérale à gauche (Terminal, Hôtes, SFTP, Tunnels, Snippets, Trousseau, Sécurité et journal, Équipe et synchro, Réglages).

### Jetons de design

À centraliser dans `crates/app/src/theme.rs`. Jamais de couleur écrite « en dur » ailleurs.

| Jeton | Valeur |
|---|---|
| Fond | `#0B0F17` |
| Surface | `#131A28` |
| Surface 2 | `#1A2336` |
| Bordure | `#222D42` |
| Bordure forte | `#2A3750` |
| Barre latérale | `#0E1420` |
| Terminal | `#070A10` |
| Navigation modale | `#0F1522` |
| Texte | `#E8ECF4` |
| Texte atténué | `#8F9BB3` |
| Texte discret | `#5D6B85` |
| Primaire (action) | `#2DD4BF` |
| Primaire teinte | `#12332F` |
| Sur primaire | `#06201D` |
| Accent | `#38BDF8` |
| Succès | `#34D399` |
| Alerte | `#FBBF24` |
| Erreur | `#F87171` |
| Violet | `#A78BFA` |
| Accent teinte | `#12333F` |
| Erreur teinte | `#3A1A20` |
| Violet teinte | `#271F45` |
| Alerte teinte | `#3A2F12` |

- Tailles de texte : 11, 12, 13, 14, 15, 16, 18, 20, 24, 28 (et 40 pour un chiffre clé).
- Polices : Inter pour l'interface, une police mono embarquée pour le terminal.
- Rayons : 8 (lignes), 10 (boutons, champs), 14 (cartes), 18 (modales).
- Espacement : grille de 8.
- Le thème clair viendra plus tard : `theme.rs` doit exposer une structure (pas des constantes globales éparpillées) pour pouvoir échanger la palette.

## Règles de sécurité (non négociables)

- Ne jamais journaliser (tracing, println, debug) un mot de passe, une passphrase, une clé privée ou un contenu de coffre.
- Les secrets sont dans des types qui s'effacent (`zeroize` / `Zeroizing`), jamais clonés inutilement, jamais laissés dans un `String` qui traîne.
- Ne pas écrire ses propres primitives cryptographiques. Utiliser uniquement des crates reconnues.
- Empreintes d'hôtes : première connexion = demander confirmation ; empreinte modifiée = connexion BLOQUÉE par défaut. Jamais d'acceptation silencieuse (pas de `NoCheckServerKey` hors tests).
- Un fichier de coffre ouvert dans un éditeur de texte doit être illisible.
- Tests uniquement sur mon serveur Docker, jamais sur un vrai serveur.

## Licences

Je peux lire des projets existants (tern en GPUI sous GPL-3.0, Portal en egui) pour comprendre leur architecture, mais ne copie pas leur code sans vérifier la licence et me prévenir.

---

## Roadmap v1 (une étape à la fois)

Taille : S = quelques jours, M = 1 à 2 semaines, L = 2 semaines ou plus.

### Étape 0 · Préparation (S)

- Créer le workspace (core, vault, terminal, app), configurer rustfmt et clippy, écrire un README.
- Monter un serveur SSH de test dans Docker (par exemple `linuxserver/openssh-server`) avec un utilisateur de test, accessible par mot de passe puis par clé.
- Créer `theme.rs` avec les jetons ci-dessus.
- Fixer la version d'iced.

**Terminé quand :** `cargo build` et `cargo clippy` passent, et `ssh` classique se connecte au conteneur.

### Étape 1 · Cœur SSH sans interface (M)

- `crates/core` : connexion par mot de passe, puis par clé Ed25519 avec passphrase, shell avec pseudo-terminal, redimensionnement, keepalive. Types d'erreurs clairs (thiserror).
- Un petit binaire de test en ligne de commande qui ouvre un shell interactif.

**Terminé quand :** le binaire me donne un shell interactif sur le conteneur et se ferme proprement.

### Étape 2 · Terminal en Rust, le plus gros risque (L)

- **2a. Test de faisabilité (2 jours).** Afficher une grille alacritty_terminal dans iced (texte coloré, curseur), alimentée par des octets de test. Mesurer la fluidité avec un `cat` d'un gros fichier. Si le rendu est impraticable dans iced, STOP et me le dire : on étudie egui.
- **2b.** Brancher le flux d'un canal SSH réel sur le moteur, envoyer la saisie clavier (flèches, Ctrl, Alt, touches de fonction) et le redimensionnement.
- **2c.** Sélection à la souris, copier-coller (arboard), historique de défilement. Liens cliquables et recherche : plus tard.

**Terminé quand :** vim, htop et tmux fonctionnent correctement dans un terminal affiché par iced.

Conseils : limiter l'étape à environ 3 semaines ; ne redessiner que les lignes modifiées ; regrouper les octets reçus par petits lots.

### Étape 3 · Coque de l'application (M)

- Écran 01 : barre latérale, onglets (plusieurs sessions), un terminal par onglet.
- Composants réutilisables, écrits une seule fois : pastille de statut, toggle, carte, bouton primaire et secondaire, champ de saisie, puce de tag. Tous basés sur `theme.rs`.
- Embarquer les polices (Inter + police mono) dans le programme.

**Terminé quand :** j'ouvre deux onglets SSH en parallèle et je passe de l'un à l'autre sans bug visuel.

### Étape 4 · Hôtes et stockage (M)

- Écrans 03 (liste en cartes, groupes, tags, favoris, recherche) et 04 (formulaire Nouvel hôte : général, authentification par mot de passe ou clé).
- Stockage serde + toml dans le dossier de configuration (`directories`). Import de `~/.ssh/config` (`ssh2-config`).

**Terminé quand :** je crée, modifie, cherche et importe des hôtes, et tout est encore là après un redémarrage.

### Étape 5 · Sécurité (L), à faire relire

- Empreintes d'hôtes (known_hosts) et écran 13 : alerte bloquante si l'empreinte change.
- Coffre (crate `vault`) : mot de passe maître, chiffrement, effacement mémoire (zeroize), verrouillage automatique après inactivité, écran 12.
- Trancher age ou argon2 + chacha20poly1305 et me justifier le choix avant d'écrire du code.
- Tests unitaires obligatoires pour le coffre et la vérification d'empreintes.

**Terminé quand :** le fichier de coffre est illisible dans un éditeur de texte, et une empreinte modifiée bloque la connexion.

### Étape 6 · Fiabilité (M)

- Reconnexion automatique, détection de coupure, messages d'erreur clairs (clé refusée, hôte injoignable, délai dépassé), pastilles d'état dans les onglets.

**Terminé quand :** je coupe le réseau 10 secondes et la session reprend d'elle-même.

### Étape 7 · SFTP basique (M)

- Écran 05 réduit : deux volets, navigation, envoi et téléchargement, file de transferts simple (`russh-sftp`).

**Terminé quand :** j'envoie et télécharge un fichier de 100 Mo sans bloquer l'interface.

### Étape 8 · Finitions et publication (M)

- Réglages essentiels (écran 11), thème clair et sombre via la structure de thème, `cargo-packager` pour les installeurs, README avec captures.

**Terminé quand :** quelqu'un d'autre installe l'application et se connecte à un serveur sans mon aide.

### Hors v1 (ne PAS implémenter sans que je le demande)

Tunnels, snippets multi-hôtes, bastion en chaîne, authentification multi-facteurs, synchronisation, équipe, enregistrement de sessions, monitoring, palette de commandes, mobile, web.

---

## Format de tes réponses

1. Rappelle en une ligne l'étape en cours et l'objectif de ce message.
2. Donne le code sous forme de petits diffs ou de fichiers courts, avec le chemin du fichier.
3. Explique en quelques phrases ce que ça fait et pourquoi.
4. Termine par la commande à lancer pour vérifier (`cargo build`, `cargo test` ou test manuel) et ce que je dois observer.
5. Signale les risques ou points à relire (sécurité, performance, API instable).