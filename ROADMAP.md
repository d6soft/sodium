# Sodium — Roadmap

## Vision

Outil TUI (Terminal User Interface) de gestion Git multi-projets, multi-utilisateurs.
Un seul binaire cross-platform (Linux, macOS, Windows) remplaçant le script `git.sh` bash.

**Stack** : Rust + ratatui

## Origine

Le fichier `git.sh` (copié dans ce dépôt) est le prototype bash actuel.
Il gère un seul dépôt avec : init, commit, push, merge, branches, fetch, historique.

---

## Phase 1 — Port du script bash vers Rust/ratatui

Reproduire les fonctionnalités existantes de `git.sh` dans une TUI ratatui.

### 1.1 Scaffold projet
- [ ] `cargo init` avec dépendances : `ratatui`, `crossterm`, `git2` (libgit2), `color-eyre`
- [ ] Structure de base : `main.rs`, `app.rs`, `ui.rs`, `git.rs`
- [ ] Boucle événementielle crossterm (clavier, resize)

### 1.2 Header / Dashboard
- [ ] Afficher le nom du dépôt, branche courante, dernier commit
- [ ] Tableau branches Local / Remote (comme le `print_header` actuel)
- [ ] Indicateurs ahead/behind par rapport à upstream
- [ ] Compteurs fichiers : modifiés, stagés, non suivis

### 1.3 Menu actions
- [ ] Navigation flèches + entrée (comme `menu_select`)
- [ ] Actions : nouvelle branche, commit, switch, fetch, merge, push, backup
- [ ] Séparateurs visuels dans le menu
- [ ] Confirmation pour actions destructives (reinit)

### 1.4 Actions git (via git2)
- [ ] `init` + création bare repo distant (SSH)
- [ ] `commit` avec sélection fichiers + message
- [ ] `push` / `pull` / `fetch --prune`
- [ ] `merge` branche → main
- [ ] `checkout` / `switch` branche
- [ ] Génération `.gitignore` auto (détection stack)

---

## Phase 2 — Multi-projets

### 2.1 Configuration
- [ ] Fichier config `~/.config/sodium/config.toml`
- [ ] Déclarer des projets : nom, chemin local, remote
- [ ] Remote configurable par projet (pas forcément le même serveur)

### 2.2 Vue projets
- [ ] Écran d'accueil listant tous les projets enregistrés
- [ ] Statut rapide par projet : branche, ahead/behind, fichiers modifiés
- [ ] Navigation pour entrer dans un projet → vue détaillée (Phase 1)
- [ ] Ajout / suppression de projets depuis la TUI

---

## Phase 3 — Multi-utilisateurs

### 3.1 Conscience des collaborateurs
- [ ] Afficher les branches remote avec leur auteur (dernier commit)
- [ ] Indicateur visuel : "Thierry travaille sur `feat-auth`"
- [ ] Détection de conflits potentiels (mêmes fichiers modifiés sur 2 branches)

### 3.2 Synchronisation assistée
- [ ] Workflow guidé : fetch → pull → merge → résolution conflits → push
- [ ] Diff avant merge (résumé des fichiers impactés)
- [ ] Aide à la résolution de conflits (liste des fichiers, ouverture éditeur)

---

## Phase 4 — Fonctionnalités avancées

### 4.1 Stash
- [ ] Sauvegarder / restaurer / lister / supprimer des stashs
- [ ] Stash rapide avant switch de branche

### 4.2 Historique / Log
- [ ] Vue log scrollable avec graph des branches
- [ ] Détail d'un commit (diff, fichiers)
- [ ] Export markdown (comme `action_history` actuel)

### 4.3 Tags
- [ ] Créer / lister / pousser des tags
- [ ] Convention de versioning configurable

### 4.4 Diff intégré
- [ ] Vue diff inline dans la TUI (avant commit, avant merge)
- [ ] Colorisation syntaxique basique

---

## Phase 5 — Cross-platform et distribution

### 5.1 Build
- [ ] CI GitHub Actions : Linux (musl), macOS (x86 + arm), Windows (msvc)
- [ ] Binaire unique sans dépendance

### 5.2 Installation
- [ ] Script d'install (`curl | sh`)
- [ ] Packages : `.deb`, `.rpm`, Homebrew, `winget`/`scoop`

---

## Contraintes techniques

| Aspect | Choix |
|--------|-------|
| Langage | Rust (edition 2021+) |
| TUI | ratatui + crossterm |
| Git | git2 (libgit2 bindings) |
| SSH | git2 avec credentials SSH agent |
| Config | TOML (`~/.config/sodium/config.toml`) |
| Cible | Linux x86_64, macOS arm64/x86_64, Windows x86_64 |
