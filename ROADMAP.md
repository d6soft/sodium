<!-- TANTALE:CONTEXT -->
Sodium est un outil TUI (Terminal User Interface) de gestion Git multi-projets et multi-utilisateurs, écrit en Rust avec ratatui. Il remplace le script bash git.sh existant par un binaire unique cross-platform (Linux, macOS, Windows). Le projet est structuré en 6 phases : Phase 1 (port bash → Rust TUI) est terminée, Phase 2 (multi-projets) est quasi terminée, Phase 3 (améliorations workflow) a le commit amélioré fait mais le reste est en attente, Phases 4 à 6 (multi-utilisateurs, fonctionnalités avancées, distribution) sont entièrement à faire. Stack : Rust edition 2021+, ratatui 0.29, crossterm 0.28, git2 0.19, config TOML. Le projet dispose d'un thème visuel dark-ops avec effets glitch et palette neon.
<!-- /TANTALE:CONTEXT -->

<!-- ATTENTION CLAUDE CODE :
Ne JAMAIS supprimer ni modifier les identifiants #TTxxx.
Pour les dates de debut et fin, utiliser la date et heure actuelles (ex: 2026-03-31 14:30), jamais T00:00:00.
Si tu modifies une tache, tu DOIS mettre a jour sa date Maj avec la date/heure courante.
A chaque fois que tu travailles sur une tache, mets a jour son temps passe (temps: en minutes) en estimant le temps reel de travail effectue.
Renseigne le champ Resultat apres chaque intervention sur une tache. Sois tres concis et utilise /humanizer.
Formats :
  - [ ] #TT123 : titre de la tache
    - debut: 2026-04-01 09:00 | fin: 2026-04-05 18:00 | Maj: 2026-04-01 14:30:00
    - Statut : En cours | temps: 150 | Assigné : Pierre V.
    - Description libre de la tache
    - Résultat : texte libre du resultat
  - [x] #TT124 : tache terminee
  - [ ] Nouvelle tache sans #TT (Titan l'attribuera)
-->

<!-- TANTALE:TASKS -->
<!-- TT569 -->
- [ ] #TT569 : Pull (action manquante dans le workflow)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT569 -->
<!-- TT575 -->
- [ ] #TT575 : Action git pull origin (rebase vs merge configurable)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT575 -->
<!-- TT601 -->
- [ ] #TT601 : Diff intégré dans la TUI (avant commit, avant merge)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT601 -->
<!-- TT576 -->
- [ ] #TT576 : Stash rapide (save, pop, liste, auto avant switch)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT576 -->
<!-- TT577 -->
- [ ] #TT577 : Raccourci [f] fetch depuis liste projets
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT577 -->
<!-- TT578 -->
- [ ] #TT578 : Raccourci [p] push depuis liste projets
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT578 -->
<!-- TT579 -->
- [ ] #TT579 : Raccourci [P] pull depuis liste projets
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT579 -->
<!-- TT581 -->
- [ ] #TT581 : Batch operations (sélection multi-projets, fetch/pull all)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT581 -->
<!-- TT585 -->
- [ ] #TT585 : Recherche / filtre projets par nom en temps réel
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT585 -->
<!-- TT589 -->
- [ ] #TT589 : Détection conflits potentiels avant merge (overlap fichiers)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT589 -->
<!-- TT591 -->
- [ ] #TT591 : Synchronisation assistée (fetch → pull → merge → résolution → push)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT591 -->
<!-- TT592 -->
- [ ] #TT592 : Diff avant merge (résumé fichiers impactés)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT592 -->
<!-- TT598 -->
- [ ] #TT598 : Vue log scrollable (git log --graph en TUI)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT598 -->
<!-- TT572 -->
- [ ] #TT572 : Ajout / suppression de projets depuis la TUI
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT572 -->
<!-- TT573 -->
- [ ] #TT573 : Remote configurable par projet
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT573 -->
<!-- TT580 -->
- [ ] #TT580 : Feedback inline sur ligne projet (spinner/ok/error)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT580 -->
<!-- TT582 -->
- [ ] #TT582 : Barre de progression globale batch
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT582 -->
<!-- TT583 -->
- [ ] #TT583 : Résumé batch (fetched, errors)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT583 -->
<!-- TT586 -->
- [ ] #TT586 : Afficher branches remote avec auteur (dernier commit)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT586 -->
<!-- TT587 -->
- [ ] #TT587 : Indicateur visuel collaborateur actif sur branche
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT587 -->
<!-- TT590 -->
- [ ] #TT590 : Indicateur conflit potentiel dans liste projets
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT590 -->
<!-- TT593 -->
- [ ] #TT593 : Aide résolution conflits (liste fichiers, ouverture éditeur)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT593 -->
<!-- TT594 -->
- [ ] #TT594 : Contrôle d'accès SSH par repo (private/team, chmod/chgrp)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT594 -->
<!-- TT595 -->
- [ ] #TT595 : Déclaration utilisateurs dans sodium.toml
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT595 -->
<!-- TT599 -->
- [ ] #TT599 : Détail d'un commit (diff, fichiers) sur Enter
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT599 -->
<!-- TT602 -->
- [ ] #TT602 : Colorisation syntaxique diff (syntect)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT602 -->
<!-- TT603 -->
- [ ] #TT603 : Navigation fichier par fichier dans diff
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT603 -->
<!-- TT604 -->
- [ ] #TT604 : Gestion tags (créer, lister, pousser)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT604 -->
<!-- TT606 -->
- [ ] #TT606 : Watch mode (auto-refresh liste projets, détection push collègue)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT606 -->
<!-- TT609 -->
- [ ] #TT609 : CI GitHub Actions (Linux musl, macOS x86+arm, Windows msvc)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT609 -->
<!-- TT610 -->
- [ ] #TT610 : Binaire unique sans dépendance
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT610 -->
<!-- TT584 -->
- [ ] #TT584 : Card Activity optionnelle (show_activity config)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT584 -->
<!-- TT588 -->
- [ ] #TT588 : Colonne auteur dans card BRANCHES
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT588 -->
<!-- TT596 -->
- [ ] #TT596 : Menu action Access control dans vue projet
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT596 -->
<!-- TT597 -->
- [ ] #TT597 : Affichage statut accès dans liste projets
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT597 -->
<!-- TT605 -->
- [ ] #TT605 : Convention de versioning configurable
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT605 -->
<!-- TT607 -->
- [ ] #TT607 : Notification visuelle changement remote
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT607 -->
<!-- TT608 -->
- [ ] #TT608 : Notifications système (alerte push collègue, bell terminal)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT608 -->
<!-- TT611 -->
- [ ] #TT611 : Script d'install (curl | sh)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT611 -->
<!-- TT612 -->
- [ ] #TT612 : Packages distribution (.deb, .rpm, Homebrew, winget/scoop)
  - Màj: 2026-04-02 16:41
  - Statut : Backlog
<!-- /TT612 -->
<!-- TT1503 -->
- [x] #TT1503 : Subcommands CLI sodium pour usage depuis n'importe quel dossier projet
  - début: 2026-05-17 11:56 | fin: 2026-05-17 12:03 | Màj: 2026-05-17 10:04
  - Statut : Terminé | temps: 10
### Description
Ajouter des subcommands au binaire `sodium` pour invoquer les 4 ops Git les plus fréquentes depuis n'importe quel dossier projet, sans passer par la TUI ni par le daemon `--api`.

Commandes à exposer :
- `sodium new-branch <name>` — crée la branche depuis main
- `sodium commit -m <msg>` — mode global, stage tout puis commit (équivalent bouton « Global » TUI). Réutilise `git_ops::git_commit` qui invoque déjà `git_clean_tracked_ignored` en pré-staging.
- `sodium merge-main <feature>` — appelle la fonction extraite dans la tâche 2
- `sodium push` — équivalent `git_push_main` + miroir GitHub si configuré

Détails :
- Auto-détection du repo via `git rev-parse --show-toplevel` sur `$PWD`
- Flag `--path <dir>` optionnel pour override
- Sortie : 1 ligne human-friendly sur stdout, code retour 0/1 pour scripting
- Dispatcher partagé avec `api.rs` (factoriser `handle_request` en fonction commune)
- Tooling : étendre l'usage de clap
### Résultat
Nouveau module `src/cli.rs` avec dispatch sur argv[1] (parsing manuel, pas besoin d'ajouter clap). Les 4 subcommands résolvent le repo via `git rev-parse --show-toplevel` ou `--path`, partagent les fonctions de `git_ops`, et reprennent la logique du miroir GitHub (extraite dans `git_ops::mirror_to_github`). `main.rs` invoque `cli::try_dispatch` avant le mode TUI / `--api`. Build release OK, tests d'usage et de path absent passent (exit 2 + message).
<!-- /TT1503 -->
<!-- TT1504 -->
- [x] #TT1504 : Action API `merge_into_main` + extraction dans git_ops
  - début: 2026-05-17 11:56 | fin: 2026-05-17 12:03 | Màj: 2026-05-17 10:05
  - Statut : Terminé | temps: 6
### Description
Extraire la logique merge actuellement inline dans `app.rs` (lignes ~1255-1399, flow stash → switch main → git merge feature → stash pop) vers une fonction `git_ops::git_merge_into_main(repo: &Path, feature: &str) -> Result<String, String>`.

Puis :
- Ajouter le variant `ApiRequest::MergeIntoMain { path: Option<String>, branch: String }` dans `src/api.rs`
- Brancher l'action dans `handle_request`
- La TUI doit aussi utiliser la nouvelle fonction (suppression du code dupliqué dans app.rs)
- La subcommand `sodium merge-main <feature>` (tâche 1) appelle directement cette fonction
### Résultat
Fonction `git_ops::git_merge_into_main(path, feature)` extraite avec garde-fous (rejet de `feature` vide ou égale à `main`). `app.rs::do_merge` réduit à un simple appel + notification (11 lignes au lieu de 85). Variant `ApiRequest::MergeIntoMain { path, branch }` ajouté et branché dans `handle_request`. Bonus : `git_ops::mirror_to_github` extrait aussi pour parité TUI/CLI.
<!-- /TT1504 -->
<!-- TT1505 -->
- [x] #TT1505 : Skill Claude Code `/sodium-git` + amendement CLAUDE.md global
  - début: 2026-05-17 11:56 | fin: 2026-05-17 12:03 | Màj: 2026-05-17 10:05
  - Statut : Terminé | temps: 5
### Description
Créer un skill Claude Code (`~/.claude/skills/sodium-git/SKILL.md`) qui décrit les 4 commandes Sodium autorisées pour Claude Code dans n'importe quel projet :
- `new-branch`, `commit`, `merge-main`, `push`
- Convention de message commit (1-2 lignes courtes)
- Nomenclature des branches (feature/xxx)
- Note explicite : Claude peut invoquer ces 4 commandes sans confirmation, car Sodium applique les garde-fous projet (GITCON, miroir GitHub, scan suspects tracked, gitignore généré, clean tracked ignored).

Amender `~/.claude/CLAUDE.md` global pour ajouter l'exception à la règle « pas de git » pour ces 4 commandes via Sodium uniquement.
### Résultat
Skill `~/.claude/skills/sodium-git/SKILL.md` créé, déjà reconnu dans la liste des skills disponibles. Documente les 4 commandes, conventions (message en français impératif, branches feature/fix), limites (pas de rebase/tag/reset/force-push), comportement attendu (annoncer, exécuter, ne pas redemander confirmation). `~/.claude/CLAUDE.md` global amendé : section « Déploiement et git » contient maintenant un bloc d'exception listant explicitement les 4 commandes autorisées via `sodium` uniquement.
<!-- /TT1505 -->
<!-- TT1506 -->
- [x] #TT1506 : Audit log Sodium pour subcommands CLI et API socket
  - début: 2026-05-17 12:08 | fin: 2026-05-17 12:11 | Màj: 2026-05-17 10:12
  - Statut : Terminé | temps: 3
### Description
Ajouter un journal d'audit dédié à Sodium pour tracer toutes les invocations des subcommands CLI (`sodium new-branch | commit | merge-main | push`) ainsi que les requêtes reçues sur le socket Unix de l'API headless.

**Motivation** : depuis la skill Claude Code `/sodium-git` (TT1505), ces 4 commandes peuvent être invoquées de manière autonome par Claude sans confirmation préalable. Un log centralisé facilite l'audit : « qui a déclenché quoi, quand, depuis quel cwd ».

**Spécifications** :
- Fichier : `~/.config/sodium/audit.log`
- Rotation : pas pour l'instant (à voir si volume devient gênant)
- Format ligne (à choisir entre tab-separated et pipe-separated lors de l'implémentation) :
  - timestamp ISO-8601 local (ex: `2026-05-17T12:07:42+02:00`)
  - source : `cli` ou `api`
  - path résolu du repo (absolu)
  - action (`new-branch`, `commit`, `merge-main`, `push`, `fetch`, `pull`, `status`, `branches`, etc.)
  - arguments pertinents (nom de branche, message commit tronqué à 80 chars…)
  - résultat : `ok` ou `err: <message tronqué>`

**Implémentation** :
- Nouveau module `src/audit.rs` exposant `audit::log(source, repo, action, args, result)`
- Appelé depuis `cli.rs` pour chaque subcommand et depuis `api.rs::handle_request` pour chaque variant
- Tolérant aux erreurs : si l'écriture échoue, ne pas faire échouer la commande (juste un eprintln warning en mode debug)
- Création du dossier `~/.config/sodium/` à la volée si absent (déjà géré ailleurs dans le code via `config.rs`, voir s'il y a un helper réutilisable)

**Rotation** ajoutée à la demande de Pierre : 1 fichier par semaine ISO + purge auto au-delà de 5 semaines.
### Résultat
Module `src/audit.rs` créé. Format **TSV** (timestamp ISO-8601 local | source | repo absolu | action | args | result). **Rotation hebdo** : un fichier `audit-YYYY-Www.log` par semaine ISO, purge auto des fichiers > 5 semaines à chaque appel. Tolérant : écriture en best-effort, `SODIUM_AUDIT_DEBUG=1` pour surfacer les erreurs sur stderr. Branché dans `cli.rs` (4 subcommands) et dans `api.rs::handle_request` via un dispatcher centralisé `describe_request` + `dispatch` (13 actions auditées uniformément, source=`api`). Test OK : `sodium new-branch feature/test-audit` dans un repo jetable → ligne `2026-05-17T12:11:18+02:00\tcli\t/tmp/sodium-audit-test\tnew-branch\tfeature/test-audit\tok` écrite dans `~/.config/sodium/audit-2026-W20.log`.
<!-- /TT1506 -->
<!-- TT1507 -->
- [x] #TT1507 : Sortie JSON obligatoire pour toutes les subcommands CLI
  - début: 2026-05-17 12:17 | fin: 2026-05-17 12:24 | Màj: 2026-05-17 10:26
  - Statut : Terminé | temps: 3
### Description
**Décision** : la sortie des subcommands `sodium <action>` devient **JSON uniquement** — pas de flag opt-in, pas de mode texte alternatif. Toutes les invocations émettent une ligne JSON unique sur stdout, dans tous les cas (succès, échec git, usage incorrect, repo introuvable).

Motivation : usages avancés prévus (hooks Git côté serveur, CI, scripts d'automatisation, autres agents qui pilotent Sodium). Un format unique simplifie le contrat d'API et évite la divergence text/json.

**Format de sortie** (1 ligne, sur stdout) :
- Succès : `{"ok": true, "action": "<action>", "message": "<msg human-friendly>", "data": {...}}` (`data` optionnel selon l'action — ex : `{"branches_cleaned": 2}` pour push)
- Échec git : `{"ok": false, "action": "<action>", "error": "<message git>"}` (code retour 1)
- Usage incorrect / repo introuvable : `{"ok": false, "action": "<action>", "error": "<message>"}` (code retour 2)

**Spécifications** :
- Réutiliser le format du `ApiResponse` existant dans `src/api.rs` (champs `ok`, `data`, `error`) pour cohérence entre CLI et API socket
- Ajouter le champ `action` dans la struct partagée (CLI + API)
- Supprimer **toute** écriture human-friendly sur stdout/stderr depuis `cli.rs`. Seule exception : `SODIUM_AUDIT_DEBUG=1` peut surfacer des erreurs d'écriture du fichier audit sur stderr.
- Codes de retour conservés : 0 = ok, 1 = échec exécution, 2 = usage / repo introuvable
- Documentation : README — la section « API headless » est étendue ou renommée pour couvrir aussi le mode CLI JSON ; même format de réponse des deux côtés
- Skill `~/.claude/skills/sodium-git/SKILL.md` : Claude doit parser le JSON pour extraire `message` ou `error` à présenter à Pierre. Plus de texte libre à relayer tel quel.

**Impact** : la sortie courante (`Branch 'feature/foo' created & active`) devient `{"ok":true,"action":"new-branch","message":"Branch 'feature/foo' created & active"}`. Tout consommateur actuel (s'il y en a) doit basculer sur le JSON. Pas de période de transition souhaitée — c'est le format définitif.
### Résultat
Struct `ApiResponse` étendue dans `src/api.rs` avec les champs `action: Option<&'static str>` et `message: Option<String>`, exposée publiquement. Constructeurs `ok_msg / ok_with / ok_data / err / with_action`. Les 13 arms du dispatch API migrés ; `handle_request` tagge l'action une fois en sortie de `dispatch`. CLI (`src/cli.rs`) entièrement refactorée : helpers `emit_ok` / `emit_err`, plus aucun `println!`/`eprintln!` human-friendly, JSON systématique sur stdout. Tests OK :
- `sodium new-branch feature/json-test` → `{"ok":true,"action":"new-branch","message":"Branch 'feature/json-test' created & active"}`
- `sodium new-branch` (sans arg) → `{"ok":false,"action":"new-branch","error":"usage: …"}` + exit 2
- `sodium push --path /tmp` → `{"ok":false,"action":"push","error":"not a git repository: /tmp"}` + exit 2

Skill `~/.claude/skills/sodium-git/SKILL.md` mise à jour : tableau du format JSON + consigne « parser et extraire `message`/`error` plutôt que coller le JSON brut ». README : section renommée « API headless et CLI JSON » avec exemples côte à côte.
<!-- /TT1507 -->
<!-- /TANTALE:TASKS -->

<!-- TANTALE:INBOX -->
Aucune demande en cours.
<!-- /TANTALE:INBOX -->


<!-- TANTALE:NOTES -->
Aucune correspondance trouvée avec les tâches Tantale existantes (#TT). Le projet Sodium est un outil Git TUI indépendant qui ne correspond à aucun des projets suivis dans Tantale (CRM Resalice, Catalibris, Geo, Pulse, Echo, etc.). Si Sodium doit être suivi dans Tantale, il faudra créer un nouveau projet dédié et importer ces tâches. Points d'attention : (1) La Phase 1 pull manquant est aussi listée en Phase 3.2 — c'est la même tâche, à ne pas dupliquer. (2) Le commit sélectif (Phase 1.5) est marqué comme corrigé en Phase 3.1 — cohérent. (3) Les Phases 4-6 sont ambitieuses (multi-utilisateurs, cross-platform) et pourraient nécessiter un re-priorisation selon l'usage réel.
<!-- /TANTALE:NOTES -->
