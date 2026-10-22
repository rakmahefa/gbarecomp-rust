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
- Le run PR `39745275263` a chargé le workflow, passé le filtre ROM, installé les dépendances Linux et Rust, puis échoué sur `cargo fmt --check` uniquement pour deux diffs de formatage dans `src/recompiler/mod.rs` et `src/runtime.rs`. Ces deux corrections sont dans le commit courant ; Clippy et les tests n'ont pas encore été atteints.
- Aucun build local n'a été exécuté dans l'environnement de préparation ; il ne disposait pas de Cargo/rustc.
- Le lockfile Bevy a été basé sur une résolution Cargo existante de Bevy 0.20 et adapté au package du projet ; **il faut le laisser à la CI avec `--locked`**. Toute erreur du resolver doit être corrigée via Cargo puis le nouveau lockfile commité.
- Le runtime ne fait qu'ouvrir une fenêtre Bevy et afficher une forme de remplacement ; pas de chargement ROM, décodeur ARM/Thumb, IR, émetteur Rust ou modèle matériel encore.

## Prochaine étape

1. Confirmer que le run CI de PR parse le workflow et passe le garde-fou ROM.
2. Corriger le lockfile/API/formattage/Clippy selon le vrai résultat de CI.
3. Une fois le scaffold validé, ajouter le parseur d'en-tête GBA et une carte des régions ROM.
4. Puis ajouter un ensemble limité d'instructions ARM/Thumb et des tests qui valident l'émission Rust et ses effets. Aucune de ces étapes ne doit ajouter un interpréteur/JIT.

Chaque commit de clôture de groupe doit mettre à jour ce handoff.
