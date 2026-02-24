# Sodium - Guide utilisateur

## Qu'est-ce que Sodium ?

Sodium est un outil TUI (Terminal User Interface) de gestion Git multi-projets. Il remplace les commandes git manuelles par une interface visuelle dans le terminal, avec un thème "dark-ops" inspiré des consoles de surveillance.

```
███████  ██████  ██████  ██ ██    ██ ███    ███
██      ██    ██ ██   ██ ██ ██    ██ ████  ████
███████ ██    ██ ██   ██ ██ ██    ██ ██ ████ ██
     ██ ██    ██ ██   ██ ██ ██    ██ ██  ██  ██
███████  ██████  ██████  ██  ██████  ██      ██
```

---

## Installation

### Prérequis communs

- **Git** installé et dans le PATH (clé SSH configurée pour les remotes)
- **Rust / Cargo** — installer via [rustup.rs](https://rustup.rs)

### Linux / macOS

Les outils de compilation C sont nécessaires pour les dépendances natives (`libgit2`, `libssh2`, `openssl`).

**Linux (Debian/Ubuntu)** :

```bash
sudo apt install build-essential pkg-config libssl-dev cmake
```

**macOS** :

```bash
xcode-select --install
```

**Build** :

```bash
git clone <url-du-repo> sodium
cd sodium
cargo build --release
# Le binaire est dans target/release/sodium
```

### Windows

**Prérequis** :

1. **Git for Windows** — [git-scm.com](https://git-scm.com/download/win)
2. **Visual Studio Build Tools** — [visualstudio.microsoft.com](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   - Cocher **"Desktop development with C++"** lors de l'installation
   - Cela fournit le compilateur MSVC, les headers Windows et CMake

**Build** :

```powershell
git clone <url-du-repo> sodium
cd sodium
cargo build --release
# Le binaire est dans target\release\sodium.exe
```

**Terminal recommandé** : Windows Terminal ou PowerShell moderne. Le vieux `cmd.exe` ne supporte pas bien les couleurs RGB et les caractères Unicode utilisés par Sodium (heatmap, logo, indicateurs).

### Vérifier l'installation

```bash
./target/release/sodium
# ou sur Windows :
.\target\release\sodium.exe
```

Au premier lancement, Sodium crée automatiquement le fichier de configuration (voir section suivante).

---

## Configuration

Au premier lancement, Sodium crée automatiquement `~/.config/sodium/sodium.toml` :

```toml
dev_root = "~/dev"
remote_host = "git-PM7"
remote_path = "repos"
pull_rebase = true
activity_show = true
```

| Clé | Description | Defaut |
|-----|-------------|--------|
| `dev_root` | Dossier contenant vos projets (expansion `~` supportée) | `~/dev` |
| `remote_host` | Serveur SSH pour les bare repos | `git-PM7` |
| `remote_path` | Chemin sur le serveur vers les repos bare | `repos` |
| `pull_rebase` | `true` = pull --rebase, `false` = pull merge | `true` |
| `activity_show` | Afficher le panneau ACTIVITY (heatmap) | `true` |

**Mode multi-projet** : si `dev_root` existe et contient des dossiers, Sodium affiche la liste de tous vos projets au lancement.

**Mode mono-projet** : si pas de config ou `dev_root` invalide, Sodium s'ouvre sur le dossier courant.

### Miroir GitHub (optionnel)

Pour activer le push miroir vers GitHub sur certains projets, ajoutez une table `[projects.<nom>]` avec la cle `github` :

```toml
dev_root = "~/dev"
remote_host = "git-PM7"
remote_path = "repos"
pull_rebase = true
activity_show = true

[projects.sodium]
github = "git@github.com:user/sodium.git"

[projects.mon-api]
github = "git@github.com:user/mon-api.git"
```

Le nom de la table (`sodium`, `mon-api`) doit correspondre exactement au nom du dossier dans `dev_root`.

Quand le miroir est configure pour un projet :
- **Push main -> origin** pousse aussi vers GitHub automatiquement
- **Backup branche -> origin** pousse aussi vers GitHub automatiquement
- **Reinitialize repo** ajoute le remote `github` lors du setup
- Le remote `github` est cree automatiquement s'il n'existe pas encore
- Si le push GitHub echoue, le push origin reste valide (pas de blocage)
- Un indicateur `◆ GitHub` apparait dans la barre repo

Les projets sans table `[projects.xxx]` ne sont pas affectes.

---

## Les deux ecrans

### 1. Liste des projets

L'ecran d'accueil quand le mode multi-projet est actif. Chaque projet affiche :

```
  ▸ mon-projet       main      ● CLEAN      2h ago  "dernier message de commit"
    autre-projet      feature   ▲1 ▪3 dirty  1d ago  "fix: correction bug"
    vieux-truc        —         NO REPO
```

- **Nom** du dossier
- **Branche** courante
- **Statut** : CLEAN (vert), dirty count + ahead/behind (orange), NO REPO (rouge)
- **Age** du dernier commit
- **Message** du dernier commit (tronque)

**Barre de resume** en haut : `12 PROJECTS — 8 clean | 3 dirty | 1 no repo`

#### Raccourcis

| Touche | Action |
|--------|--------|
| `↑` / `k` | Projet precedent |
| `↓` / `j` | Projet suivant |
| `Enter` | Ouvrir le projet |
| `r` | Rafraichir la liste |
| `q` | Quitter |
| `Ctrl+C` | Quitter (forcer) |

---

### 2. Vue detail d'un projet

L'ecran principal une fois dans un projet. Compose de :

#### Barre GITCON

Indicateur d'alerte sur l'etat du repo, inspiré du systeme DEFCON :

| Niveau | Couleur | Signification |
|--------|---------|---------------|
| GITCON 1 | Vert | Tout est synced, arbre propre |
| GITCON 2 | Vert-jaune | Changements mineurs (fichiers non-tracked, petit ahead) |
| GITCON 3 | Orange | Divergence significative (behind > 5 ou > 20 fichiers dirty) |
| GITCON 4 | Orange-rouge | Conflits detectes |
| GITCON 5 | Rouge | Pas de repo / etat casse |

#### Panneau BRANCHES

Tableau deux colonnes montrant les branches locales et remote :

```
  Local                Remote
▸ main                ● main
  feature-auth        ● feature-auth
                      ○ fix-typo          ← branche remote-only (pas en local)
  old-branch            —                 ← branche locale sans remote
```

- `▸` = branche courante (en rouge)
- `●` = branche presente sur le remote (synchronisee)
- `○` = branche remote-only (disponible via Checkout remote)
- `—` = pas de remote

#### Panneau ACTIVITY

- Sparkline des commits sur les 14 derniers jours
- Total commits + commits recents
- Statut sync : `● SYNCED` (vert) ou `▲2 ▼1` (ahead/behind)

#### Panneau FILES

Compteurs des fichiers modifies :

- `▪ 3 modified` (orange)
- `▪ 1 staged` (vert)
- `▪ 2 untracked` (gris)
- `▪ 1 CONFLICT` (rouge)
- Ou `● CLEAN` si rien a committer

#### Raccourcis

| Touche | Action |
|--------|--------|
| `↑` / `k` | Action precedente dans le menu |
| `↓` / `j` | Action suivante dans le menu |
| `Enter` | Executer l'action selectionnee |
| `r` | Rafraichir les donnees du repo |
| `Esc` / `Backspace` | Retour a la liste (mode multi-projet) |
| `q` | Retour a la liste ou quitter (mode mono-projet) |
| `Ctrl+C` | Quitter |

---

## Actions du menu

### New branch

Cree une nouvelle branche et bascule dessus.

```
> Sodium demande le nom de la branche
> Equivalent : git checkout -b <nom>
```

**Exemple** : vous commencez une feature, tapez `fix-login`, Sodium cree la branche et vous y place.

---

### Commit [branche]

Commit interactif avec review des fichiers. Le nom de la branche courante est affiche entre crochets.

**Etape 1 — Review** : Sodium affiche la liste des fichiers modifies avec les stats de diff :

```
 ▸ M  src/app.rs                        +45    -12
   M  src/config.rs                      +8     -2
   ?  tests/new_test.rs                 +30     -0
   D  old_file.txt                       +0    -15
```

- `M` = modifie (orange)
- `A` = ajoute au staging (vert)
- `D` = supprime (rouge)
- `?` = non suivi (gris)
- `C` = conflit (rouge gras)
- `R` = renomme (magenta)

**Etape 2 — Choix** :

| Touche | Action |
|--------|--------|
| `a` | Tout ajouter → passer au message de commit |
| `Enter` | Mode selection manuelle |
| `Esc` | Annuler |

**Etape 3 — Selection manuelle** (si `Enter`) :

| Touche | Action |
|--------|--------|
| `Space` | Cocher/decocher le fichier |
| `a` | Tout selectionner |
| `n` | Tout deselectionner |
| `Enter` | Confirmer la selection → passer au message |
| `Esc` | Annuler |

**Etape 4 — Message** : saisissez le message de commit, `Enter` pour valider.

**Exemple concret** : vous avez modifie 5 fichiers mais ne voulez committer que 2. Selectionnez Commit → `Enter` pour le mode selection → `Space` sur les 2 fichiers voulus → `Enter` → tapez "fix: correction du login" → `Enter`.

---

### Switch branch

Bascule vers une autre branche **locale**.

Sodium affiche un selecteur avec toutes les branches locales sauf la courante. Selectionnez avec `↑`/`↓` et `Enter`.

```
Equivalent : git checkout <branche>
```

**Note** : si vous n'avez qu'une seule branche locale, Sodium affiche "No other branches available".

---

### Fetch (refresh)

Recupere l'etat du serveur distant sans modifier votre code local.

```
Equivalent : git fetch --prune origin
```

- Met a jour la liste des branches remote
- Supprime les references vers des branches remote supprimees (`--prune`)
- Rafraichit l'affichage Sodium (compteurs, branches, GITCON)

**Quand l'utiliser** : en debut de journee ou avant un pull, pour voir si des collegues ont pousse des changements.

---

### Pull origin

Tire les changements du remote pour la branche courante.

```
Si pull_rebase = true  → git pull --rebase origin <branche>
Si pull_rebase = false → git pull origin <branche>
```

**Prerequis** : la branche courante doit exister sur le remote (avoir ete pushee au prealable). Sinon Sodium affiche : `[INTEL] '<branche>' has no remote — nothing to pull`.

**Workflow type** : Fetch → voir que le remote a avance → Pull origin.

**Rebase vs merge** : configurable via `pull_rebase` dans `sodium.toml`. Le rebase (defaut) garde un historique lineaire, le merge cree un commit de fusion.

---

### Checkout remote branch

Recupere en local une branche qui n'existe que sur le remote.

Sodium affiche un selecteur avec les branches remote-only (celles qui apparaissent avec `○` dans le panneau BRANCHES, hors `main`).

```
Equivalent : git checkout -b <branche> origin/<branche>
```

**Exemple** : un collegue a pousse `feat-auth` sur origin. Apres un Fetch, la branche apparait dans la colonne Remote avec `○`. Selectionnez "Checkout remote branch" → choisissez `feat-auth` → Sodium cree la branche locale et bascule dessus.

---

### Merge into main / Merge branche -> main

Fusionne une branche dans main.

**Si vous etes sur main** : Sodium affiche un selecteur pour choisir quelle branche merger.

**Si vous etes sur une feature** : Sodium bascule automatiquement sur main et merge votre branche.

```
Equivalent :
  git checkout main  (si pas deja dessus)
  git merge <branche>
```

**En cas de conflit** : Sodium affiche l'erreur et le GITCON passe a 4. Vous devrez resoudre les conflits manuellement dans votre editeur, puis committer via Sodium.

---

### Backup branche -> origin

Pousse la branche courante (hors main) vers origin. Utile pour sauvegarder votre travail en cours sans merger.

```
Equivalent : git push origin <branche>
```

Si le miroir GitHub est configure pour ce projet, la branche est aussi pushee vers `github` automatiquement. La notification affiche `+ GitHub` en cas de succes.

**Note** : cette action n'apparait que si vous n'etes **pas** sur main. Pour main, utilisez "Push main -> origin".

---

### Push main -> origin

Pousse la branche main vers le serveur distant.

```
Equivalent : git push origin main
```

**Bonus** : apres un push reussi, Sodium nettoie automatiquement les branches deja mergees dans main (locale + remote). Le message indique combien de branches ont ete supprimees.

Si le miroir GitHub est configure pour ce projet, main est aussi pushee vers `github` automatiquement. La notification affiche `+ GitHub` en cas de succes.

---

### Export history

Genere un fichier Markdown avec l'historique Git du projet :

```
Genere : docs/git-<nom-du-projet>-<date>.md
```

Le fichier contient :
- Metadonnees (date, branche, remote)
- Historique des commits des 10 derniers jours
- Liste de toutes les branches
- Statistiques (total commits, contributeurs)

---

### Reinitialize repo

Action **destructive** qui reinitialise completement le depot Git.

**Etape 1** : tapez `CONFIRM` pour confirmer
**Etape 2** : saisissez le nom du repo (pre-rempli avec le nom actuel)

Sodium va :
1. Supprimer le bare repo sur le serveur SSH (si existant)
2. Creer un nouveau bare repo sur le serveur
3. Supprimer le `.git` local
4. `git init -b main`
5. Ajouter le remote origin
6. Ajouter le remote github (si configure dans `sodium.toml`)
7. Generer un `.gitignore` adapte au projet (detection automatique : Node/Svelte/Capacitor/Rust/Go/Flutter)
8. Faire un commit initial
9. Push vers origin

**Quand l'utiliser** : quand un repo est trop pollue ou que vous voulez repartir de zero tout en gardant vos fichiers.

---

## Workflow quotidien type

### Solo

```
1. Lancer sodium
2. Ouvrir votre projet (Enter)
3. Fetch (refresh)          ← voir l'etat du remote
4. Pull origin              ← synchroniser si des changements
5. ... travailler ...
6. Commit [branche]         ← committer votre travail
7. Si feature terminee :
   - Merge branche -> main  ← fusionner dans main
   - Push main -> origin    ← envoyer + cleanup branches
8. Sinon :
   - Backup branche -> origin  ← sauvegarder la feature
```

### En equipe

```
1. Lancer sodium
2. Ouvrir le projet
3. Fetch (refresh)          ← voir les branches des collegues
4. Pull origin              ← recuperer les changements sur votre branche
5. Si besoin de la branche d'un collegue :
   - Checkout remote branch ← recuperer la branche en local
6. ... travailler ...
7. Commit → Backup          ← sauvegarder regulierement
8. Quand c'est pret :
   - Switch main → Merge → Push
```

---

## Notifications

Les notifications apparaissent dans le footer avec clignotement pendant ~4 secondes :

| Prefixe | Signification |
|---------|---------------|
| `[INTEL]` | Information (vert) — operation reussie ou info neutre |
| `[SIGINT]` | Action importante terminee (vert) — commit, push, merge |
| `[ERROR]` | Erreur (rouge) — l'operation a echoue |
| `[ABORT]` | Annulation (rouge) — action annulee ou impossible |

---

## Gitignore automatique

Lors d'un `Reinitialize repo`, Sodium genere un `.gitignore` adapte en detectant les fichiers du projet :

| Detection | Ignore |
|-----------|--------|
| Toujours | `.DS_Store`, `.env`, `.vscode/`, `*.log`, `*.tmp` |
| `package.json` | `node_modules/`, `build/`, `dist/`, `.svelte-kit/` |
| `capacitor.config.ts` | `android/`, `ios/` |
| `Cargo.toml` | `target/`, `*.rs.bk` |
| `go.mod` | `bin/`, `vendor/` |
| `pubspec.yaml` | `.dart_tool/`, `build/`, `.flutter-plugins` |

---

## FAQ

**Q : Sodium ne detecte pas mes projets**
R : Verifiez que `dev_root` dans `~/.config/sodium/sodium.toml` pointe vers le bon dossier. Les dossiers caches (`.hidden`) sont ignores.

**Q : "Pull origin" affiche "has no remote"**
R : La branche courante n'a jamais ete pushee. Utilisez d'abord "Backup" pour la pousser sur origin, ou travaillez sur une branche qui existe deja sur le remote.

**Q : Le GITCON est orange/rouge, que faire ?**
R : Fetch + Pull pour synchroniser, puis committez ou resolvez les conflits. Le GITCON repasse au vert quand le repo est propre et synced.

**Q : Comment changer rebase/merge pour le pull ?**
R : Editez `~/.config/sodium/sodium.toml` et mettez `pull_rebase = false` pour utiliser merge au lieu de rebase.

**Q : Un dossier affiche "NO REPO"**
R : Ce dossier dans `dev_root` n'a pas de `.git`. Utilisez Reinitialize repo pour en creer un, ou initialisez-le manuellement avec `git init`.

**Q : Comment activer le miroir GitHub ?**
R : Ajoutez une section `[projects.nom-du-dossier]` avec `github = "git@github.com:user/repo.git"` dans `~/.config/sodium/sodium.toml`. Le nom doit correspondre au dossier du projet. L'indicateur `◆ GitHub` apparait dans la barre repo quand le remote est detecte.

**Q : Le push GitHub echoue mais le push origin fonctionne**
R : Le miroir GitHub est non-bloquant. Verifiez que votre cle SSH a acces au repo GitHub, et que l'URL dans `sodium.toml` est correcte. Le push origin reste valide meme si GitHub echoue.
