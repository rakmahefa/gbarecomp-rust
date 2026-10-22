# Session handoff

Dernière mise à jour : 2026-10-22
Groupe de travail : Phase 1 — scaffold Rust/Bevy et invariants de recompilation
Branche de préparation : `phase-1/rust-bevy-static-only`

## But du projet

Créer un recompilateur statique GBA en Rust qui **génère du code Rust**, compilé avant lancement, avec Bevy comme runtime hôte. Linux x86_64 uniquement dans un premier temps. Aucune interprétation du code invité et aucun JIT ne sont autorisés.

## État au début du groupe

- La PR #1 de Phase 0 a été fusionnée sur `main` (commit `fed900f090e15779e02faa42abea0bf04568f73a`).
- La ROM GBA a été retirée de l'arbre courant lors de la Phase 0 ; l'ancien blob subsiste dans l'historique Git et doit être purgé si sa redistribution n'est pas autorisée.
- Le dépôt avait désormais la documentation et les quatre workflows GitHub Actions, mais pas encore de `Cargo.toml` ni de code Rust.
- La référence GBARecomp amont est principalement C++/CMake et sous PolyForm Noncommercial 1.0.0 ; REA est sous MIT.

## Travail inclus dans ce groupe

- Création du crate package et library `gbarecomp-rust`, du toolchain pin et du premier runtime fenêtre Bevy.
- Pin Bevy `=0.20.0`; Rust `1.98.1`; cible Linux x86_64.
- Création de l'ADR-0001 : génération source Rust statique, aucun interpréteur/JIT ; les cas non résolus échouent avec un diagnostic contextualisé.
- Types d'erreur de recompilation initiaux et tests pour les cibles indirectes non résolues et le code invité généré au runtime.
- Mise à jour du README, de l'audit, de l'architecture, des références, du guide Linux et de la stratégie CI/build.
- CI et build rendus stricts : plus de succès apparent causé par l'absence de manifeste Cargo.

## Validation et limites

- Le premier run CI (39744719658) a échoué sur le garde-fou ROM parce que le motif refusait aussi `rom/README.md`. Le filtre a été ajusté pour autoriser le README tout en refusant les ROM/BIOS/sauvegardes binaires.
- Aucun `rustc` ni `cargo` n'était installé dans l'environnement de préparation utilisé ; le build local n'a donc pas été exécuté.
- La nouvelle exécution CI doit encore valider le lockfile, les API de Bevy 0.20, le formatage, Clippy et les tests.
- La fenêtre Bevy est un placeholder : le chargeur de ROM, l'analyse ARM/Thumb, l'IR, l'émission Rust et le modèle matériel ne sont pas implémentés.
- Le choix de licence du code nouveau reste à décider.
- La ROM n'est pas intégrée au binaire, aux artefacts ou au bundle REA.

## Prochaine tâche

Exécuter et corriger le premier build CI. Ensuite commencer une première tranche statique testable : parseur d'en-tête GBA, cartographie ROM, puis quelques instructions ARM/Thumb et génération d'un module Rust natif. Chaque groupe de commits qui clôt une tâche doit ajouter ses changements, tests exécutés, SHA et risques ici.
