# Sodium

**Git TUI dashboard — dark ops style**

Sodium remplace les commandes git manuelles par une interface visuelle dans le terminal, avec gestion multi-projets et un theme dark-ops inspire des consoles de surveillance.

![Project view](sodium-project.png)

## Features

- **Multi-projets** — vue d'ensemble de tous vos repos dans `~/dev`
- **Commit interactif** — review des fichiers avec stats de diff, selection manuelle ou globale
- **Branches** — creation, switch, checkout remote, merge dans main
- **Push / Backup** — push main ou sauvegarde de feature branches vers origin
- **Miroir GitHub** — push automatique vers un 2e remote GitHub (configurable par projet)
- **GITCON** — indicateur d'alerte visuel sur l'etat du repo (inspire du systeme DEFCON)
- **Heatmap d'activite** — grille commits/merges/pulls sur 91 jours
- **Export historique** — generation d'un rapport Markdown
- **Clone / Delete server repos** — cloner ou supprimer des bare repos depuis le serveur SSH
- **Reinitialisation** — reset complet d'un repo avec generation automatique du `.gitignore`
- **API headless** — socket Unix pour piloter les operations Git par script/automatisation

## Installation

```bash
# Prerequis : Rust/Cargo, Git, outils de compilation C
# Linux : sudo apt install build-essential pkg-config libssl-dev cmake
# macOS : xcode-select --install

git clone https://github.com/d6soft/sodium.git
cd sodium
cargo build --release
./target/release/sodium
```

## Configuration

Au premier lancement, Sodium cree `~/.config/sodium/sodium.toml` :

```toml
dev_root = "~/dev"
remote_host = "git-PM7"
remote_path = "repos"
pull_rebase = true
activity_show = true

# Miroir GitHub (optionnel, par projet)
[projects.sodium]
github = "git@github.com:d6soft/sodium.git"
```

## API headless

Sodium peut tourner en mode serveur sans TUI, exposant un socket Unix pour l'automatisation :

```bash
# Lancer le serveur API
sodium --api /chemin/vers/repo

# Socket custom
sodium --api /chemin/vers/repo --socket /tmp/custom.sock

# Requetes (une ligne JSON in, une ligne JSON out)
echo '{"action":"status"}' | socat - UNIX-CONNECT:/tmp/sodium-api.sock | jq
echo '{"action":"branches"}' | socat - UNIX-CONNECT:/tmp/sodium-api.sock | jq
echo '{"action":"gitcon"}' | socat - UNIX-CONNECT:/tmp/sodium-api.sock | jq
echo '{"action":"projects"}' | socat - UNIX-CONNECT:/tmp/sodium-api.sock | jq
echo '{"action":"files"}' | socat - UNIX-CONNECT:/tmp/sodium-api.sock | jq
echo '{"action":"fetch"}' | socat - UNIX-CONNECT:/tmp/sodium-api.sock | jq
echo '{"action":"pull"}' | socat - UNIX-CONNECT:/tmp/sodium-api.sock | jq
echo '{"action":"commit","message":"fix typo"}' | socat - UNIX-CONNECT:/tmp/sodium-api.sock | jq
```

Actions disponibles : `status`, `branches`, `files`, `gitcon`, `projects`, `fetch`, `pull`, `push`, `backup`, `commit`, `new_branch`, `switch_branch`. Chaque action accepte un `path` optionnel pour cibler un repo different du defaut.

## Documentation

Voir [SODIUM-USER-GUIDE.md](SODIUM-USER-GUIDE.md) pour le guide complet.

## Stack

- **Rust** avec [ratatui](https://github.com/ratatui/ratatui) pour le TUI
- **git2** (libgit2) pour les operations Git natives
- **crossterm** pour le rendu terminal cross-platform
- **serde_json** pour l'API socket Unix

## Licence

Usage personnel.
