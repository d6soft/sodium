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

Sodium attend un fichier `~/.config/sodium/sodium.toml`. Tous les champs ci-dessous sont **obligatoires** — Sodium refuse de démarrer si le fichier est absent ou incomplet.

```toml
dev_root = "~/dev"
remote_host = "git.example.com"
remote_path = "repos"
pull_rebase = true
activity_show = true

# Miroirs (1..n, optionnel, par projet)
# Chaque miroir reçoit un force-push après le push réussi sur origin.
[projects.<nom-du-projet>.mirrors.github]
url = "git@github.com:<user>/<repo>.git"

[projects.<nom-du-projet>.mirrors.gitlab]
url = "git@gitlab.com:<user>/<repo>.git"
```

L'ancienne forme `github = "..."` au niveau `[projects.<nom>]` reste lue et fusionnée comme un miroir nommé `github`.

## API headless et CLI JSON

Sodium expose la même surface d'API via deux canaux : un **socket Unix** (mode serveur, plusieurs requêtes par connexion) et des **subcommands CLI** (invocation directe, une commande par appel). Les deux émettent le même format de réponse JSON.

### Subcommands CLI

Invocables depuis n'importe quel dossier projet — le repo est détecté via `git rev-parse --show-toplevel` sur `$PWD`, ou explicité avec `--path <dir>`.

```bash
sodium new-branch <name>
sodium commit -m "<message>"
sodium merge-main <feature>
sodium push
sodium remotes
sodium add-github [--owner <org>] [--name <repo>] [--public|--private] [--yes]
sodium init-remote [--name <repo>] [--yes] [--force]
```

- `sodium remotes` : vue croisée des remotes Git physiques (`git remote -v`) et des miroirs déclarés dans `sodium.toml`. Champ `source` ∈ `git`, `sodium-config`, `both` ; champ `mismatch` si les URLs divergent.
- `sodium add-github` : crée le repo côté GitHub via `gh repo create` (CLI `gh` requise et authentifiée), ajoute le remote local `github`, et propose d'ajouter la section `[projects.<repo>.mirrors.github]` à `sodium.toml`. Owner par défaut `d6soft`, nom du repo déduit du dossier courant. Prompts interactifs sur stderr (visibilité, confirmations) ; le JSON final reste sur stdout. Mode non-interactif : `--public|--private --yes`.
- `sodium init-remote` : crée le bare repo sur `remote_host:remote_path/<repo>.git` (par SSH, `git init --bare`) puis ajoute le remote local `origin`. Nom du repo déduit du dossier courant ou forcé via `--name`. Refuse si `origin` existe déjà localement. Si le bare existe déjà côté serveur : prompt interactif (réutiliser / écraser / annuler), ou `--force` en mode `--yes` pour écraser sans demander.

**Sortie : JSON systématique sur stdout, une ligne par invocation.** Aucune écriture human-friendly. Codes de retour : `0` succès, `1` échec d'exécution, `2` usage incorrect ou repo introuvable.

```bash
$ sodium new-branch feature/foo
{"ok":true,"action":"new-branch","message":"Branch 'feature/foo' created & active"}

$ sodium push --path /not-a-repo
{"ok":false,"action":"push","error":"not a git repository: /not-a-repo"}
```

Format unifié avec l'API socket : `{"ok": bool, "action": "...", "message": "...", "data": {...}, "error": "..."}` (champs absents quand non pertinents). Conçu pour scripts, hooks Git, CI, agents.

### API socket Unix

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
