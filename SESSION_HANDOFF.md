# Session handoff

Dernière mise à jour : 2026-10-22
Groupe de travail : Phase 1 — scaffold Rust/Bevy, audit des fichiers et correction CI
Branche : `phase-1/rust-bevy-static-only`
PR : [#2](https://github.com/rakmahefa/gbarecomp-rust/pull/2)

## But et invariant non négociable

Le projet analyse du code GBA ARM7TDMI (ARM/Thumb) et **génère du code source Rust** qui sera compilé avant exécution avec Cargo/rustc. Bevy fournit le runtime hôte et la présentation ; il n'interprète pas le code invité.

**Interdits : aucun interpréteur d'instructions invité, aucun JIT, aucune traduction de code invité à l'exécution.** Une cible indirecte non résolue ou une instruction non prise en charge doit échouer à l'étape de recompilation avec un diagnostic contextualisé. Ne jamais ajouter de fallback interprété/JIT.

## État avant ce groupe

- PR #1 de Phase 0 fusionnée sur `main`, commit `fed900f090e15779e02faa42abea0bf04568f73a`.
- La ROM a été supprimée de l'arbre courant mais le blob subsiste dans l'historique public antérieur ; vérifier les droits et envisager une purge coordonnée si la redistribution est non autorisée.
- Le dépôt ne possédait pas encore de crate Rust ni de runtime Bevy.

## Changements de ce groupe

- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `src/main.rs`, `src/lib.rs`, `src/runtime.rs` et `src/recompiler/mod.rs) ajoutés pour le premier scaffold.
- Bevy `=0.20.0`, Rust `1.98.1`, cible de départ Linux x86_64.
- `docs/decisions/ADR-0001-static-rust-recompilation.md` formalise la décision Rust statique sans interpréteur/JIT.
- `docs/repository-audit.md` inventorie chaque fichier suivi et définit le cycle de revue documentaire.
- README, audit de risque, architecture, guide de développement, références, workflows et ce handoff mis à jour.
- CI rendue stricte et dépendances Linux de Bevy ajoutées aux workflows CI/build.
- La CI précédente a échoué avant les commandes Rust ; les incidents du filtre ROM et de la syntaxe du workflow ont été identifiés et corrigés dans les commits de suivi.

## Maintenance documentaire obligatoire

À chaque groupe de travail significatif, relire et mettre à jour **README.md**, **audit.md**, **docs/repository-audit.md** et **SESSION_HANDOFF.md**. Si un fichier change de rôle ou est ajouté/supprimé/renommé, refléter ce changement dans l'inventaire. Le README doit décrire l'état vrai du dépôt, pas les objectifs futurs. Le handoff note la documentation revue, les résultats et échecs de validation, le SHA final et la prochaine étape. Faire aussi une revue complète avant chaque release.

## Validation et limites

- Le run CI `39744719658` a échoué car le filtre refusait `rom/README.md`.
- Le run push `39744961191` s'est terminé immédiatement en échec avec nom de workflow `.github/workflows/ci.yml` et sans jobs : la version de workflow sur la branche était syntaxiquement corrompue. Le contenu de `.github/workflows/ci.yml` a été réécrit proprement.
- Le run PR `39745275263` a chargé le workflow, passé le filtre ROM, installé les dépendances Linux et Rust, puis échoué sur `cargo fmt --check` pour deux diffs dans `src/recompiler/mod.rs` et `src/runtime.rs`. Ces diffs sont corrigés.
- Le run PR `39745465091` a ensuite passé le filtre ROM et le formatage, puis Clippy a échoué : `Cargo.lock` nécessitait une mise à jour sous `--locked`. Le lockfile est remplacé par une résolution générée pour Bevy 0.20 feature `2d` seule.
- Le run PR `39745818857` a passé filtre ROM, formatage et résolution du lockfile, puis la compilation a détecté que `build_app()` retournait indirectement `&mut App`. Le builder a été corrigé dans le commit `bcd0b8f8c124b6cc9a5ae61357fe245120817183`.
- **Validation réussie :** le run CI `39746308189` sur ce commit a passé le filtre ROM, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` et `cargo test --workspace --all-features --locked`. Clippy et les tests sont donc validés pour le scaffold actuel.
- Le workflow Build (Linux) est volontairement ignoré pour l'événement PR ; le build release, le packaging et l'artefact REA restent à confirmer sur un push à `main` après fusion.
- Aucun build local n'a été exécuté dans l'environnement de préparation ; il ne disposait pas de Cargo/rustc.
- Le lockfile Bevy a été basé sur une résolution Cargo existante de Bevy 0.20 et adapté au package du projet ; **il faut le laisser à la CI avec `--locked`**. Toute erreur du resolver doit être corrigée via Cargo puis le nouveau lockfile commité.
- Le runtime configure une fenêtre Bevy et une forme de remplacement ; le lancement interactif n'a pas encore été testé localement. Aucun chargement ROM, décodeur ARM/Thumb, IR, émetteur Rust ou modèle matériel n'est encore implémenté.

## Prochaine étape

1. Après fusion, confirmer que Build (Linux) produit le binaire de distribution et l'entrée REA depuis le même build release, sans inclure la ROM.
2. Ajouter le parseur d'en-tête GBA et la cartographie des régions ROM avec des tests sur fixtures non propriétaires.
3. Définir une IR ARM/Thumb testable, puis implémenter des traductions limitées et générer du code Rust que Cargo compile comme étape de build.
4. Ajouter des tests différentiels pour la sémantique de l'IR et des effets matériels. Aucune de ces étapes ne doit ajouter un interpréteur/JIT.

Chaque commit de clôture de groupe doit mettre à jour ce handoff.
