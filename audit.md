# Audit initial — 2026-10-22

## Résumé

Le dépôt est à l'état de squelette. Cette Phase 0 met en place les garde-fous et l'automatisation, mais ne peut pas démontrer qu'une recompilation ou qu'un runtime démarre : aucun crate Rust n'existe encore.

## Constatations

### A-01 — ROM GBA suivie dans un dépôt public

**Sévérité : élevée — action requise.**

Le fichier rom/testrom.gba était suivi par Git dans main et sa taille observée était de 16 777 216 octets (16 MiB). L'exclusion du paquet de release ne suffit pas : un fichier suivi est également distribué par GitHub dans l'arbre source du dépôt et dans les archives source créées à partir d'un tag.

Cette branche retire la ROM de l'arbre courant, ajoute des règles d'ignorance pour rom/ et conserve les instructions d'utilisation locale. Sauvegarder la ROM hors du checkout avant de fusionner cette branche si elle doit rester disponible pour le développement, puis la replacer localement ; elle sera ignorée par Git.

**Limite importante :** le blob demeure dans les anciens commits publics. La branche ne réécrit pas l'historique et ne prétend donc pas purger ce contenu. Si la ROM n'est pas autorisée à être redistribuée publiquement, une purge d'historique coordonnée (par exemple avec git-filter-repo, suivie d'une mise à jour forcée contrôlée) reste nécessaire. Vérifier les droits de la ROM avant toute redistribution.

### A-02 — Absence de projet Rust compilable

**Sévérité : élevée pour la livraison, attendue pour la Phase 0.**

Aucun Cargo.toml, Cargo.lock, fichier source Rust ou exécutable n'était présent. La CI exécute donc les contrôles génériques de dépôt et ignore explicitement les commandes Cargo jusqu'à l'ajout du crate. Le build Linux s'exécute après la CI réussie sur main, mais publie un résumé indiquant l'absence de manifeste et n'émet aucun artefact dans cet état.

### A-03 — Licence de la référence C++ à vérifier avant tout portage de code

**Sévérité : élevée si du code source est réutilisé.**

Le fichier LICENSE de [mstan/gbarecomp](https://github.com/mstan/gbarecomp/blob/main/LICENSE) indique **PolyForm Noncommercial License 1.0.0**. Cette licence autorise certains usages non commerciaux selon ses conditions ; elle ne doit pas être supposée compatible avec une distribution commerciale. La consulter intégralement et demander l'autorisation appropriée avant de copier ou d'adapter du code de cette référence. Les documents et comportements matériels peuvent être étudiés comme références, mais ce n'est pas une licence de code.

Le dépôt [morluto/rea](https://github.com/morluto/rea/blob/main/LICENSE) est sous MIT. Respecter ses notices si des portions de son code sont un jour réutilisées.

### A-04 — Licence et provenance propres au projet non définies

**Sévérité : moyenne.**

Ne pas ajouter une licence permissive par défaut sans décision du mainteneur. Choisir et documenter la licence du code écrit ici, les notices de tiers et la façon de traiter les ROM, BIOS, fichiers de sauvegarde et données générées.

### A-05 — Sémantique du runtime à définir avant l'optimisation

**Sévérité : moyenne.**

La recompilation statique doit préserver les effets observables du code ARM/Thumb et les frontières avec le modèle matériel GBA. Bevy doit rester une couche hôte/presentation ; ses systèmes de rendu ne doivent pas devenir la source de vérité de l'état matériel. Documenter les contrats d'accès mémoire, IRQ, DMA, timers, PPU, audio et sauvegardes au fur et à mesure de l'implémentation. Les chemins indirects ou les zones de code dynamiques nécessiteront une stratégie de repli explicite plutôt que des stubs silencieux.

## Architecture des workflows

- CI automatique sur PR/push Linux. Les vérifications Rust s'activent lorsque le manifeste est ajouté.
- Build déclenché après succès du workflow CI d'un push sur main, jamais en parallèle de la CI de ce commit.
- Release seulement manuelle. Elle exige un tag pointant exactement sur le commit du build sélectionné et réutilise ses artefacts.
- Le bundle REA est lancé manuellement et réutilise l'artefact d'analyse du build ; il ne recompile pas et ne prétend pas exécuter l'agent REA en arrière-plan.
- Les artefacts sont temporaires (14 jours) ; le tag/release est le canal de distribution explicite.
