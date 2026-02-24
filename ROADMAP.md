# Sodium — Roadmap

## Vision

Outil TUI (Terminal User Interface) de gestion Git multi-projets, multi-utilisateurs.
Un seul binaire cross-platform (Linux, macOS, Windows) remplaçant le script `git.sh` bash.

**Stack** : Rust + ratatui

## Origine

Le fichier `git.sh` (copié dans ce dépôt) est le prototype bash actuel.
Il gère un seul dépôt avec : init, commit, push, merge, branches, fetch, historique.

---

## Phase 1 — Port du script bash vers Rust/ratatui ✅

Reproduire les fonctionnalités existantes de `git.sh` dans une TUI ratatui.

### 1.1 Scaffold projet ✅
- [x] `cargo init` avec dépendances : `ratatui`, `crossterm`, `git2` (libgit2), `color-eyre`, `chrono`, `rand`
- [x] Structure de base : `main.rs`, `app.rs`, `ui.rs`, `git.rs`, `theme.rs`, `config.rs`
- [x] Boucle événementielle crossterm (clavier, tick 100ms)
- [x] Thème dark-ops (palette neon sur fond deep space)
- [x] Effet glitch au démarrage + transitions

### 1.2 Header / Dashboard ✅
- [x] Logo ASCII SODIUM animé + sous-titre rotatif
- [x] Barre GITCON (niveau 1-5 avec couleur + clignotement)
- [x] Nom du dépôt, branche courante, dernier commit, URL remote (sur 2 lignes)
- [x] Tableau branches Local / Remote
- [x] Indicateurs ahead/behind par rapport à upstream
- [x] Compteurs fichiers : modifiés, stagés, non suivis, conflits
- [x] Sparkline activité 14 jours + stats commits

### 1.3 Menu actions ✅
- [x] Navigation flèches + vim (j/k) + entrée
- [x] Actions : nouvelle branche, commit, switch, fetch, merge, push, backup, history, reinit
- [x] Séparateurs visuels dans le menu
- [x] Overlay text input (branch name, commit message, repo name)
- [x] Overlay confirmation pour actions destructives (reinit)
- [x] Overlay sélection pour switch branch et merge

### 1.4 Actions git ✅
- [x] `reinit` : CONFIRM → nom repo → SSH bare repo (rm + init) → rm .git → init → remote → gitignore → commit → push
- [x] `commit` : git add -A + git commit -m (text input)
- [x] `push` main → origin + cleanup auto branches mergées (local + remote)
- [x] `fetch --prune` origin
- [x] `merge` branche → main (popup sélection si on main, direct sinon)
- [x] `checkout` / `switch` branche (popup sélection)
- [x] `backup` branche feature → origin
- [x] `history` : export markdown `docs/git-<name>-<date>.md`
- [x] Génération `.gitignore` auto (détection Node/Svelte/Capacitor/Rust/Go/Flutter)
- [x] Notifications avec expiration auto (~4s) + clignotement

### 1.5 Manques par rapport à git.sh ⚠️
- [x] **Commit : affichage fichiers + choix sélectif** — git.sh affichait la liste des fichiers modifiés et proposait "Oui (git add -A)" / "Non, choisir manuellement" / "Annuler". Corrigé en Phase 3.1.
- [ ] **Pull** — git.sh ne l'avait pas non plus, mais c'est un trou dans le workflow standard

---

## Phase 2 — Multi-projets ✅

### 2.1 Configuration ✅
- [x] Fichier config `~/.config/sodium/sodium.toml` (TOML, serde)
- [x] Création automatique au premier lancement avec valeurs par défaut
- [x] `dev_root` : chemin vers le dossier de développement (expansion `~`)
- [x] `remote_host` / `remote_path` : serveur SSH pour les bare repos
- [x] Fallback mono-projet si config absente ou dev_root invalide

### 2.2 Vue projets ✅
- [x] Écran d'accueil listant tous les sous-dossiers de `dev_root`
- [x] Statut rapide par projet : branche, ahead/behind, dirty count, dernier commit + âge
- [x] Barre résumé : total projets, clean, dirty, no repo
- [x] Navigation ↑↓/jk + Enter pour ouvrir → vue détaillée (Phase 1)
- [x] Esc/Backspace/q pour revenir à la liste
- [x] Scroll avec indicateur position
- [x] Skip dossiers cachés (.hidden)
- [x] Dossiers sans .git affichés en "NO REPO" (rouge)
- [ ] Ajout / suppression de projets depuis la TUI
- [ ] Remote configurable par projet

---

## Phase 3 — Améliorations workflow quotidien

Quick wins à fort impact pour l'usage multi-projet au quotidien.

### 3.1 Commit amélioré (régression git.sh) ✅
- [x] Afficher la liste des fichiers modifiés/stagés/untracked avant commit
- [x] Choix : "Tout ajouter" / "Sélection manuelle" / "Annuler"
- [x] Overlay sélection multi-fichiers (espace pour toggle, entrée pour valider)
- [x] Résumé du diff (nombre de lignes +/-) par fichier

### 3.2 Pull
- [ ] Action `git pull origin <branch>` dans le menu
- [ ] Gestion rebase vs merge (configurable)

### 3.3 Stash rapide
- [ ] `[s]` stash save depuis le detail view
- [ ] `[S]` stash pop
- [ ] Liste des stashs avec restore/drop
- [ ] Stash automatique avant switch de branche si dirty

### 3.4 Raccourcis depuis la liste projets
- [ ] `[f]` fetch le projet sélectionné sans l'ouvrir
- [ ] `[p]` push le projet sélectionné sans l'ouvrir
- [ ] `[P]` pull le projet sélectionné sans l'ouvrir
- [ ] Feedback inline sur la ligne du projet (icône spinner / ok / error)

### 3.5 Batch operations
- [ ] Espace pour sélectionner/désélectionner des projets dans la liste
- [ ] `[F]` fetch all selected
- [ ] `[A]` pull all selected
- [ ] Barre de progression globale
- [ ] Résumé à la fin : "12 fetched, 2 errors"

### 3.6 Card Activity optionnelle
- [ ] Option `show_activity = true/false` dans `sodium.toml`
- [ ] Si désactivée, redistribuer l'espace aux cards BRANCHS et FILES

### 3.7 Recherche / filtre dans la liste
- [ ] `/` ouvre un champ de recherche, filtre les projets par nom en temps réel
- [ ] Esc pour annuler le filtre
- [ ] Highlight du match dans le nom

---

## Phase 4 — Multi-utilisateurs

### 4.1 Conscience des collaborateurs
- [ ] Afficher les branches remote avec leur auteur (dernier commit)
- [ ] Indicateur visuel : "Thierry travaille sur `feat-auth` (2h ago)"
- [ ] Colonne auteur dans la card BRANCHES

### 4.2 Détection de conflits potentiels
- [ ] Avant merge : lister les fichiers modifiés des deux côtés
- [ ] Avertir si overlap (mêmes fichiers touchés sur 2 branches)
- [ ] Indicateur dans la liste projets si conflit potentiel détecté

### 4.3 Synchronisation assistée
- [ ] Workflow guidé : fetch → pull → merge → résolution conflits → push
- [ ] Diff avant merge (résumé des fichiers impactés)
- [ ] Aide à la résolution de conflits (liste des fichiers, ouverture éditeur)

### 4.4 Contrôle d'accès SSH par repo
- [ ] Déclarer les utilisateurs dans `sodium.toml` (`[[server.users]]`)
- [ ] Par projet : accès `private` (owner seul, `700`) ou `team` (groupe partagé, `770 + g+s`)
- [ ] Menu action "Access control" dans la vue projet avec sélecteur private/team
- [ ] Exécution via SSH : `chgrp`/`chmod` sur le bare repo distant
- [ ] Affichage du statut accès dans la liste projets (🔒/👥)

---

## Phase 5 — Fonctionnalités avancées

### 5.1 Vue log scrollable
- [ ] Écran dédié historique (nouvel écran ou panneau)
- [ ] `git log --oneline --graph` en TUI, scrollable
- [ ] Détail d'un commit (diff, fichiers) sur Enter
- [x] Export markdown (action "Export history")

### 5.2 Diff intégré
- [ ] Vue diff inline dans la TUI (avant commit, avant merge)
- [ ] Colorisation syntaxique (crate `syntect`)
- [ ] Navigation fichier par fichier

### 5.3 Tags
- [ ] Créer / lister / pousser des tags
- [ ] Convention de versioning configurable

### 5.4 Watch mode
- [ ] Auto-refresh de la liste projets en background (configurable, ex: 30s)
- [ ] Détection de changements remote (nouveau push d'un collègue)
- [ ] Notification visuelle sur la ligne du projet concerné
- [ ] Opt-in dans la config : `watch_interval = 30`

### 5.5 Notifications système
- [ ] Alerte quand un collègue push sur un projet ouvert
- [ ] Via fetch périodique background + comparaison ahead/behind
- [ ] Son terminal (bell) optionnel

---

## Phase 6 — Cross-platform et distribution

### 6.1 Build
- [ ] CI GitHub Actions : Linux (musl), macOS (x86 + arm), Windows (msvc)
- [ ] Binaire unique sans dépendance

### 6.2 Installation
- [ ] Script d'install (`curl | sh`)
- [ ] Packages : `.deb`, `.rpm`, Homebrew, `winget`/`scoop`

---

## Contraintes techniques

| Aspect | Choix |
|--------|-------|
| Langage | Rust (edition 2021+) |
| TUI | ratatui 0.29 + crossterm 0.28 |
| Git | git2 0.19 (libgit2 bindings) + shell-out pour mutations |
| SSH | shell-out (`ssh` pour bare repos) |
| Config | TOML (`~/.config/sodium/sodium.toml`) via toml 0.8 |
| Cible | Linux x86_64, macOS arm64/x86_64, Windows x86_64 |
