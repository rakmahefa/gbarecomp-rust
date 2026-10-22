# Audit technique — 2026-10-22

## Résumé

La CI Linux a validé le scaffold Rust/Bevy, son formatage, Clippy et ses tests au run [39746990377](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39746990377). Cette tranche ajoute un parseur de l'en-tête fixe GBA et des tests synthétiques. Elle ne constitue pas encore un recompilateur : le chargement complet de ROM, le décodeur ARM/Thumb, l'IR, l'émission Rust et le modèle matériel restent à implémenter.

## Constatations et actions

### A-01 — ROM GBA précédemment suivie dans un dépôt public

**Sévérité : élevée — action requise si les droits sont incertains.**

Le fichier `rom/testrom.gba` était suivi dans l'historique initial de `main` et faisait 16 777 216 octets (16 MiB). La Phase 0 l'a retiré de l'arbre courant et a ajouté des règles d'ignorance. Il n'est donc plus présent dans l'arbre courant, mais son blob reste accessible dans les anciens commits publics tant que l'historique n'est pas réécrit.

Si sa redistribution publique n'est pas autorisée, une purge d'historique coordonnée (par exemple avec `git-filter-repo`, suivie d'une mise à jour forcée contrôlée) reste nécessaire. La sauvegarde du développeur doit être restaurée localement après le checkout de la branche appropriée, pas ajoutée à Git.

### A-02 — Le recompilateur n'est pas encore implémenté

**Sévérité : élevée pour la couverture fonctionnelle ; attendue à ce stade.**

Un parseur d'en-tête fixe est maintenant présent. Il lit les 0xC0 premiers octets requis, décode le point d'entrée et les champs de titre, game code, maker code, unité, device type et version ; il calcule aussi le complement checksum. Il ne charge pas à lui seul la ROM entière, ne trouve pas les fonctions invitées et ne génère pas encore de code Rust. Il n'existe donc toujours pas de preuve de compatibilité avec un jeu GBA.

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

### A-07 — Cohérence de Cargo.lock sous --locked

**État : résolu et validé.**

Le run CI `39745465091` a révélé que le lockfile initial adapté depuis un projet Bevy plus large ne correspondait pas exactement au manifeste minimal. Il a été remplacé par une résolution Bevy 0.20.0 avec la feature 2d seule. Le run [39746308189](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39746308189) a validé Clippy et les tests avec `--locked`. Ne pas retirer `--locked` pour masquer une divergence future.

### A-08 — Type de retour du builder Bevy

**État : résolu et validé.**

Le run `39745818857` a détecté que la chaîne de configuration Bevy retournait `&mut App`, et non `App`. Le builder configure désormais une variable locale et retourne explicitement l'application. La CI [39746308189](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39746308189) passe depuis ce correctif.

### A-09 — Parseur de l'en-tête GBA : périmètre limité

**État : implémenté dans la branche ; validation CI du commit courant en attente.**

Le parseur vérifie qu'au moins 0xC0 octets sont disponibles, extrait le point d'entrée et les champs fixes, expose les octets de titre/code/maker sans les normaliser et signale le complement checksum calculé. Le checksum incohérent est une information diagnostique et ne fait pas échouer le parsing structurel. Les tests utilisent uniquement une fixture synthétique, jamais la ROM de développement.

Ce composant ne doit ni interpréter des instructions, ni exécuter du contenu ROM, ni déclencher une compilation de code invité. L'analyse et la génération de code restent des étapes statiques de build qui viendront plus tard.

## Cadence de mise à jour de l'audit

À chaque groupe de travail significatif, revoir le présent audit, le README et [l'inventaire détaillé](docs/repository-audit.md), puis noter dans [SESSION_HANDOFF.md](SESSION_HANDOFF.md) la réalité du dépôt, les résultats CI exécutés, le commit et les risques restant ouverts. Faire une revue exhaustive avant chaque release.

## Références de workflows

- CI automatique sur PR et push Linux ; le workflow échoue si une ROM/BIOS/sauvegarde est suivie dans Git.
- Build après succès de la CI sur un push à `main`, pour éviter la concurrence avec celle-ci.
- Release manuelle : vérifie que le tag existant pointe exactement sur le commit du build et réutilise l'artefact.
- Bundle REA manuel : réutilise l'artefact et ne relance aucune compilation.
- Les artefacts temporaires sont conservés 14 jours ; la release taguée est le canal de distribution durable.
