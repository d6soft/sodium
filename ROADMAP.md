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
  - [ ] Nouvelle tache sans #TT (Shannon l'attribuera)
-->

<!-- TANTALE:TASKS -->
- [x] #TT658 : Commit : affichage fichiers + choix sélectif (régression git.sh)
  - Statut : Terminé
- [x] #TT664 : Commit amélioré (liste fichiers, choix sélectif, diff résumé)
  - Statut : Terminé
- [x] #TT654 : Scaffold projet (cargo init, structure, boucle événementielle, thème)
  - Statut : Terminé
- [x] #TT655 : Header / Dashboard (logo, GITCON, infos repo, branches, sparkline)
  - Statut : Terminé
- [x] #TT656 : Menu actions (navigation, overlays, actions git)
  - Statut : Terminé
- [x] #TT657 : Actions git (reinit, commit, push, fetch, merge, switch, backup, history, gitignore)
  - Statut : Terminé
- [ ] #TT659 : Pull (action manquante dans le workflow)
  - Statut : Backlog
- [x] #TT660 : Configuration multi-projets (sodium.toml, dev_root, remote)
  - Statut : Terminé
- [x] #TT661 : Vue projets (liste, statut, navigation, scroll)
  - Statut : Terminé
- [ ] #TT665 : Action git pull origin (rebase vs merge configurable)
  - Statut : Backlog
- [ ] #TT691 : Diff intégré dans la TUI (avant commit, avant merge)
  - Statut : Backlog
- [ ] #TT666 : Stash rapide (save, pop, liste, auto avant switch)
  - Statut : Backlog
- [ ] #TT667 : Raccourci [f] fetch depuis liste projets
  - Statut : Backlog
- [ ] #TT668 : Raccourci [p] push depuis liste projets
  - Statut : Backlog
- [ ] #TT669 : Raccourci [P] pull depuis liste projets
  - Statut : Backlog
- [ ] #TT671 : Batch operations (sélection multi-projets, fetch/pull all)
  - Statut : Backlog
- [ ] #TT675 : Recherche / filtre projets par nom en temps réel
  - Statut : Backlog
- [ ] #TT679 : Détection conflits potentiels avant merge (overlap fichiers)
  - Statut : Backlog
- [ ] #TT681 : Synchronisation assistée (fetch → pull → merge → résolution → push)
  - Statut : Backlog
- [ ] #TT682 : Diff avant merge (résumé fichiers impactés)
  - Statut : Backlog
- [ ] #TT688 : Vue log scrollable (git log --graph en TUI)
  - Statut : Backlog
- [ ] #TT662 : Ajout / suppression de projets depuis la TUI
  - Statut : Backlog
- [ ] #TT663 : Remote configurable par projet
  - Statut : Backlog
- [ ] #TT670 : Feedback inline sur ligne projet (spinner/ok/error)
  - Statut : Backlog
- [ ] #TT672 : Barre de progression globale batch
  - Statut : Backlog
- [ ] #TT673 : Résumé batch (fetched, errors)
  - Statut : Backlog
- [ ] #TT676 : Afficher branches remote avec auteur (dernier commit)
  - Statut : Backlog
- [ ] #TT677 : Indicateur visuel collaborateur actif sur branche
  - Statut : Backlog
- [ ] #TT680 : Indicateur conflit potentiel dans liste projets
  - Statut : Backlog
- [ ] #TT683 : Aide résolution conflits (liste fichiers, ouverture éditeur)
  - Statut : Backlog
- [ ] #TT684 : Contrôle d'accès SSH par repo (private/team, chmod/chgrp)
  - Statut : Backlog
- [ ] #TT685 : Déclaration utilisateurs dans sodium.toml
  - Statut : Backlog
- [ ] #TT689 : Détail d'un commit (diff, fichiers) sur Enter
  - Statut : Backlog
- [ ] #TT692 : Colorisation syntaxique diff (syntect)
  - Statut : Backlog
- [ ] #TT693 : Navigation fichier par fichier dans diff
  - Statut : Backlog
- [ ] #TT694 : Gestion tags (créer, lister, pousser)
  - Statut : Backlog
- [ ] #TT696 : Watch mode (auto-refresh liste projets, détection push collègue)
  - Statut : Backlog
- [ ] #TT699 : CI GitHub Actions (Linux musl, macOS x86+arm, Windows msvc)
  - Statut : Backlog
- [ ] #TT700 : Binaire unique sans dépendance
  - Statut : Backlog
- [ ] #TT674 : Card Activity optionnelle (show_activity config)
  - Statut : Backlog
- [ ] #TT678 : Colonne auteur dans card BRANCHES
  - Statut : Backlog
- [ ] #TT686 : Menu action Access control dans vue projet
  - Statut : Backlog
- [ ] #TT687 : Affichage statut accès dans liste projets
  - Statut : Backlog
- [ ] #TT695 : Convention de versioning configurable
  - Statut : Backlog
- [ ] #TT697 : Notification visuelle changement remote
  - Statut : Backlog
- [ ] #TT698 : Notifications système (alerte push collègue, bell terminal)
  - Statut : Backlog
- [ ] #TT701 : Script d'install (curl | sh)
  - Statut : Backlog
- [ ] #TT702 : Packages distribution (.deb, .rpm, Homebrew, winget/scoop)
  - Statut : Backlog
- [x] #TT703 : Détection proactive des dossiers suspects trackés
  - début: 2026-04-02 12:38 | fin: 2026-04-02 12:45
  - Statut : Terminé | temps: 4
  - Alerte GITCON à l'entrée d'un projet quand des dossiers build (target/, node_modules/, .next/, etc.) sont trackés. Détection root + sous-projets imbriqués.
  - Résultat : Ajout de detect_suspect_tracked() dans git_ops.rs + notification dans enter_project(). Scanne les dossiers suspects racine et les sous-projets (Cargo.toml, package.json, go.mod, pubspec.yaml).
- [x] #TT704 : Génération .gitignore : détection des sous-projets imbriqués
  - début: 2026-04-02 12:38 | fin: 2026-04-02 12:45
  - Statut : Terminé | temps: 1
  - generate_gitignore() scanne les sous-dossiers pour Cargo.toml, package.json, go.mod, pubspec.yaml et ajoute les patterns build correspondants.
  - Résultat : Section "Nested subprojects" ajoutée au .gitignore généré (subdir/target/, subdir/node_modules/, etc.).
- [x] #TT705 : Nettoyage auto du cache git au commit
  - début: 2026-04-02 12:38 | fin: 2026-04-02 12:47
  - Statut : Terminé | temps: 1
  - Avant staging, git_commit() appelle git_clean_tracked_ignored() qui exécute git rm --cached sur les fichiers trackés matchant le .gitignore. Silencieux, fichiers conservés sur disque.
  - Résultat : Nettoyage auto intégré dans git_commit(). Le nombre de fichiers nettoyés est affiché dans la notification de commit.
<!-- /TANTALE:TASKS -->

<!-- TANTALE:NOTES -->
Aucune correspondance trouvée avec les tâches Tantale existantes (#TT). Le projet Sodium est un outil Git TUI indépendant qui ne correspond à aucun des projets suivis dans Tantale (CRM Resalice, Catalibris, Geo, Pulse, Echo, etc.). Si Sodium doit être suivi dans Tantale, il faudra créer un nouveau projet dédié et importer ces tâches. Points d'attention : (1) La Phase 1 pull manquant est aussi listée en Phase 3.2 — c'est la même tâche, à ne pas dupliquer. (2) Le commit sélectif (Phase 1.5) est marqué comme corrigé en Phase 3.1 — cohérent. (3) Les Phases 4-6 sont ambitieuses (multi-utilisateurs, cross-platform) et pourraient nécessiter un re-priorisation selon l'usage réel.
<!-- /TANTALE:NOTES -->
