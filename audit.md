# Audit technique — 2026-10-22

## Résumé

La Phase 0 a préparé les garde-fous du dépôt. La Phase 1 ajoute le premier crate Rust et une application Bevy minimale. Il s'agit d'un scaffold de runtime, pas encore d'un recompilateur opérationnel : l'analyse ROM, l'IR, l'émission du code Rust et le modèle matériel restent à implémenter.

## Constatations et actions

### A-01 — ROM GBA précédemment suivie dans un dépôt public

**Sévérité : élevée — action requise si les droits sont incertains.**

Le fichier `rom/testrom.gba` était suivi dans l'historique initial de `main` et faisait 16 777 216 octets (16 MiB). La Phase 0 l'a retiré de l'arbre courant et a ajouté des règles d'ignorance. Il n'est donc plus présent dans l'arbre courant, mais son blob reste accessible dans les anciens commits publics tant que l'historique n'est pas réécrit.

Si sa redistribution publique n'est pas autorisée, une purge d'historique coordonnée (par exemple avec `git-filter-repo`, suivie d'une mise à jour forcée contrôlée) reste nécessaire. La sauvegarde du développeur doit être restaurée localement après le checkout de la branche appropriée, pas ajoutée à Git.

### A-02 — Le recompilateur n'est pas encore implémenté

**Sévérité : élevée pour la couverture fonctionnelle ; attendue à ce stade.**

Le scaffold a un crate compilable et la fenêtre Bevy de base, mais ne lit ni ne recompile encore une ROM. Il n'existe pas de preuve de compatibilité avec un jeu GBA tant que les tests du décodeur, de l'IR, des effets matériels et de l'émission Rust ne sont pas créés et exécutés.

### A-03 — Licence de la référence C++ à vérifier avant tout portage de code

**Sévérité : élevée si du code source est réutilisé.**

Le fichier LICENSE de [mstan/gbarecomp](https://github.com/mstan/gbarecomp/blob/main/LICENSE) indique **PolyForm Noncommercial License 1.0.0**. Cette licence ne doit pas être supposée compatible avec une distribution commerciale. Étudier les comportements, le matériel et la conception comme référence n'accorde pas automatiquement le droit de copier ou d'adapter son code source. Aucun code C++ de ce dépôt n'est copié dans le scaffold.

Le dépôt [morluto/rea](https://github.com/morluto/rea/blob/main/LICENSE) est sous MIT. Respecter les notices si des portions de son code sont un jour réutilisées.

### A-04 — Licence propre au nouveau code non définie

**Sévérité : moyenne.**

La licence du code écrit ici doit être décidée explicitement avant une distribution publique plus large. Ne pas ajouter une licence permissive par défaut sans accord du mainteneur. Documenter également les notices tierces et le traitement des ROM, BIOS, sauvegardes et données générées.

### A-05 — Recompilation Rust statique seulement, sans interpréteur ni JIT

**Sévérité : décision architecturale contraignante.**

Le code généré est du Rust compilé avant distribution avec Cargo/rustc. Aucun interpréteur de code invité ni JIT ne sera utilisé à l'exécution. Les cibles indirectes incomplètes, instructions non supportées, ambiguïtés code/données et régions de code générées à l'exécution doivent produire des erreurs de recompilation explicites. Elles ne doivent jamais devenir des opérations silencieuses ni des chemins de secours runtime. La décision est consignée dans [ADR-0001](docs/decisions/ADR-0001-static-rust-recompilation.md).

### A-06 — Fidélité du modèle matériel à établir

**Sévérité : élevée avant toute revendication de compatibilité.**

La recompilation doit préserver les effets observables du CPU ARM7TDMI et des services matériels GBA. Le rendu Bevy est une couche de présentation et ne constitue pas l'état matériel canonique. Les contrats du bus, registres E/S, interruptions, DMA, timers, PPU, audio et sauvegardes doivent être documentés et testés. Aucun support de jeu ne doit être déclaré à partir du seul fait que le binaire compile.

## Audit des fichiers et cadence documentaire

L'inventaire fichier par fichier se trouve dans [docs/repository-audit.md](docs/repository-audit.md). À chaque groupe de travail significatif, revoir ce document, le présent audit et `README.md`, puis consigner dans `SESSION_HANDOFF.md` lesquels ont été relus et ce qui a changé. Une mise à jour est requise même si la conclusion est qu'aucun changement n'est nécessaire ; ne pas laisser ces fichiers dériver du dépôt ou de l'état réel de CI.

## Références de workflows

- CI automatique sur PR et push Linux ; le workflow échoue si une ROM/BIOS/sauvegarde est suivie dans Git.
- Build après succès de la CI sur un push à `main`, pour éviter la concurrence avec celle-ci.
- Release manuelle : vérifie que le tag existant pointe exactement sur le commit du build et réutilise l'artefact.
- Bundle REA manuel : réutilise l'artefact et ne relance aucune compilation.
- Les artefacts temporaires sont conservés 14 jours ; la release taguée est le canal de distribution durable.
