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
<!-- /TANTALE:TASKS -->

<!-- TANTALE:NOTES -->
Aucune correspondance trouvée avec les tâches Tantale existantes (#TT). Le projet Sodium est un outil Git TUI indépendant qui ne correspond à aucun des projets suivis dans Tantale (CRM Resalice, Catalibris, Geo, Pulse, Echo, etc.). Si Sodium doit être suivi dans Tantale, il faudra créer un nouveau projet dédié et importer ces tâches. Points d'attention : (1) La Phase 1 pull manquant est aussi listée en Phase 3.2 — c'est la même tâche, à ne pas dupliquer. (2) Le commit sélectif (Phase 1.5) est marqué comme corrigé en Phase 3.1 — cohérent. (3) Les Phases 4-6 sont ambitieuses (multi-utilisateurs, cross-platform) et pourraient nécessiter un re-priorisation selon l'usage réel.
<!-- /TANTALE:NOTES -->
