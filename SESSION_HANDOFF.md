# Session handoff

Dernière mise à jour : 2026-10-22
Groupe de travail : Phase 1 — abstraction d'image ROM et mapping des fenêtres Game Pak
Branche : `phase-1/rust-bevy-static-only`
PR : [#2](https://github.com/rakmahefa/gbarecomp-rust/pull/2)

## But et invariant non négociable

Analyser le code GBA ARM7TDMI (ARM/Thumb) et **générer du code source Rust**, compilé avant exécution avec Cargo/rustc. Bevy héberge l'exécutable natif et présente le modèle matériel ; il n'interprète pas le code invité.

**Interdits : aucun interpréteur d'instructions invité, aucun JIT, aucune traduction de code invité à l'exécution.** Instruction non supportée, cible indirecte non résolue ou code invité dynamique entraîne un diagnostic de recompilation, jamais un fallback interprété/JIT/no-op.

## État au début du groupe

- PR #1 de Phase 0 fusionnée sur `main`, commit `fed900f090e15779e02faa42abea0bf04568f73a`.
- PR #2 ouverte en brouillon.
- La ROM de développement a été retirée de l'arbre courant ; le blob subsiste dans l'historique antérieur. Vérifier les droits et organiser une purge coordonnée si sa redistribution publique n'est pas permise.
- Le run CI `39746990377` a validé le scaffold au commit `a59ffc334db03e54c473d5367f5b9afcd1f84393`. Le dernier code commit déjà validé avant cette tranche, `e811b898346d654948266ef3989b4497aef70bad`, passe la CI au run `39749349856`.

## Changements de ce groupe

- Ajout de `src/rom/mod.rs`, `src/rom/header.rs` et maintenant `src/rom/image.rs` ; exposition de `pub mod rom` dans `src/lib.rs`.
- Le parseur lit l'en-tête fixe GBA de 0xC0 octets, expose le mot d'instruction ARM de branchement stocké aux offsets 0..4 (pas sa destination résolue), les champs de titre/game code/maker code/fixed value/unit code/device type/version et calcule le complement checksum.
- Erreur structurelle dédiée quand l'entrée est plus courte que 0xC0 octets.
- Un checksum incorrect reste une métadonnée diagnostique : les autres champs sont renvoyés au lieu de masquer la ROM à cause du checksum seul.
- Trois tests unitaires utilisent exclusivement un en-tête synthétique : champs et checksum valide, entrée trop courte, checksum incorrect.
- Aucun fichier ROM réel, BIOS, save ou donnée dérivée n'est utilisé ou inclus.
- Architecture, développement, README, audit de risque, inventaire de fichiers et ce handoff mis à jour.

## Validation

- À la clôture précédente, le run CI `39746990377` était vert pour le scaffold : filtre de ROM, `cargo fmt`, Clippy et `cargo test`.
- Le premier run du parseur, `39747920168`, a échoué sur `cargo fmt --check` car rustfmt demande de réordonner les noms dans le `pub use` de `src/rom/mod.rs`. Le correctif a été commité dans `c96edb7bed71ba14f09bb857160d09621a9d849e`.
- **Validation réussie pour la version précédant la clarification sémantique :** le run CI `39748310841` au commit `c96edb7bed71ba14f09bb857160d09621a9d849e` a passé les contrôles de patch, le garde-fou ROM, `cargo fmt --all -- --check`, Clippy et `cargo test --workspace --all-features --locked` ; les trois tests du header ont passé.
- Le champ `entry_point` a été renommé `entry_branch_opcode` : la CI `39749349856` a validé le nom explicite et les tests du header.
- Aucun build local n'a été exécuté dans l'environnement de préparation ; la fenêtre Bevy n'a pas été testée par lancement interactif.
- `GbaRom` possède une image fournie en mémoire et mappe les fenêtres CPU du Game Pak aux offsets. Le mapping power-of-two et ses erreurs conservatrices sont couverts par des tests synthétiques ajoutés dans ce commit. La CI de la tranche actuelle reste à exécuter.
- Le chargeur de fichier, la cartographie mémoire complète, l'analyse de fonctions, le décodeur ARM/Thumb et l'émission Rust ne sont pas encore implémentés.

## Maintenance documentaire obligatoire

À chaque groupe de travail significatif, relire et mettre à jour **README.md**, **audit.md**, **docs/repository-audit.md** et **SESSION_HANDOFF.md**. Si un fichier est ajouté, supprimé, renommé ou change de responsabilité, refléter le changement dans l'inventaire. Le README doit décrire l'état réel, pas les objectifs futurs. Le handoff doit toujours noter les vérifications exécutées et échouées, le SHA final et la prochaine tâche. Faire une revue complète du dépôt et des artefacts avant chaque release.

## Prochaine étape

1. Confirmer par CI la compilation, Clippy et les tests synthétiques de la nouvelle abstraction d'image ROM.
2. Ajouter un chargeur local de fichier ROM et les validations de taille/hash, en gardant la ROM exclue de Git et des artefacts.
3. Construire une découverte statique des régions code/données et du graphe de fonctions, avec chaque hypothèse traçable.
4. Définir une IR ARM/Thumb, commencer un ensemble borné d'instructions, puis générer des fonctions Rust compilées lors du build. Aucun interpréteur/JIT ne doit apparaître.

Chaque commit qui clôt un groupe de travail doit mettre à jour ce handoff.
