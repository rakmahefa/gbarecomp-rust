# Session handoff

Dernière mise à jour : 2026-10-22
Groupe de travail : Phase 1 — parseur initial d'en-tête GBA
Branche : `phase-1/rust-bevy-static-only`
PR : [#2](https://github.com/rakmahefa/gbarecomp-rust/pull/2)

## But et invariant non négociable

Analyser le code GBA ARM7TDMI (ARM/Thumb) et **générer du code source Rust**, compilé avant exécution avec Cargo/rustc. Bevy héberge l'exécutable natif et présente le modèle matériel ; il n'interprète pas le code invité.

**Interdits : aucun interpréteur d'instructions invité, aucun JIT, aucune traduction de code invité à l'exécution.** Instruction non supportée, cible indirecte non résolue ou code invité dynamique entraîne un diagnostic de recompilation, jamais un fallback interprété/JIT/no-op.

## État au début du groupe

- PR #1 de Phase 0 fusionnée sur `main`, commit `fed900f090e15779e02faa42abea0bf04568f73a`.
- PR #2 ouverte en brouillon.
- La ROM de développement a été retirée de l'arbre courant ; le blob subsiste dans l'historique antérieur. Vérifier les droits et organiser une purge coordonnée si sa redistribution publique n'est pas permise.
- Le run CI `39746990377` a validé le scaffold et la documentation au commit `a59ffc334db03e54c473d5367f5b9afcd1f84393`.

## Changements de ce groupe

- Ajout de `src/rom/mod.rs` et `src/rom/header.rs` ; exposition de `pub mod rom` dans `src/lib.rs`.
- Le parseur lit l'en-tête fixe GBA de 0xC0 octets, expose le point d'entrée et les champs de titre/game code/maker code/fixed value/unit code/device type/version et calcule le complement checksum.
- Erreur structurelle dédiée quand l'entrée est plus courte que 0xC0 octets.
- Un checksum incorrect reste une métadonnée diagnostique : les autres champs sont renvoyés au lieu de masquer la ROM à cause du checksum seul.
- Trois tests unitaires utilisent exclusivement un en-tête synthétique : champs et checksum valide, entrée trop courte, checksum incorrect.
- Aucun fichier ROM réel, BIOS, save ou donnée dérivée n'est utilisé ou inclus.
- Architecture, développement, README, audit de risque, inventaire de fichiers et ce handoff mis à jour.

## Validation

- À la clôture précédente, le run CI `39746990377` était vert pour le scaffold : filtre de ROM, `cargo fmt`, Clippy et `cargo test`.
- Le premier run du parseur, `39747920168`, a échoué sur `cargo fmt --check` car rustfmt demande de réordonner les noms dans le `pub use` de `src/rom/mod.rs`. Le correctif est dans le commit courant. Clippy et tests ont été ignorés sur ce run ; le run suivant doit les valider avant de déclarer cette tranche verte.
- Aucun build local n'a été exécuté dans l'environnement de préparation ; la fenêtre Bevy n'a pas été testée par lancement interactif.
- Le parseur ne charge pas la totalité de la ROM, ne mappe pas les régions ROM, ne décode pas ARM/Thumb et n'émet pas encore de code Rust.

## Maintenance documentaire obligatoire

À chaque groupe de travail significatif, relire et mettre à jour **README.md**, **audit.md**, **docs/repository-audit.md** et **SESSION_HANDOFF.md**. Si un fichier est ajouté, supprimé, renommé ou change de responsabilité, refléter le changement dans l'inventaire. Le README doit décrire l'état réel, pas les objectifs futurs. Le handoff doit toujours noter les vérifications exécutées et échouées, le SHA final et la prochaine tâche. Faire une revue complète du dépôt et des artefacts avant chaque release.

## Prochaine étape

1. Confirmer par CI la compilation, Clippy et les trois tests synthétiques de l'en-tête GBA.
2. Ajouter une représentation de la cartographie des régions ROM et des validations de taille sans inclure de ROM dans Git.
3. Définir une IR ARM/Thumb testable, puis commencer un ensemble borné d'instructions avec génération de fonctions Rust compilées durant le build.
4. Ajouter des tests sémantiques/différentiels avant d'élargir la couverture. Aucun interpréteur/JIT ne doit apparaître.

Chaque commit qui clôt un groupe de travail doit mettre à jour ce handoff.
