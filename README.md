# GBARecomp Rust

Projet expérimental de **recompilation statique GBA qui génère du code Rust natif**, avec **Bevy comme runtime hôte**. La cible initiale est **Linux x86_64 uniquement**.

L'objectif est d'analyser le code machine ARM7TDMI (ARM et Thumb) d'une ROM GBA et de générer des fonctions Rust équivalentes, puis de compiler ces sources avec Cargo/rustc. Le runtime Bevy fournit l'intégration hôte — fenêtre, rendu, boucle d'application et, à terme, audio — tandis qu'un modèle explicite représente le matériel GBA.

> **Invariant d'architecture : génération Rust statique uniquement. Aucun interpréteur invité, aucun JIT, aucun décodage/traduction d'instructions à l'exécution.** Un chemin de contrôle qui ne peut pas être résolu statiquement doit produire un diagnostic de recompilation, jamais une solution de secours à l'exécution.

## État du projet

Le scaffold Rust/Bevy et ses tests passent la CI Linux ([run 39746308189](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39746308189), commit `bcd0b8f8c124b6cc9a5ae61357fe245120817183`). Le lancement interactif de la fenêtre n'a pas encore été testé localement. **L'analyse ROM, la découverte ARM/Thumb, l'IR, l'émission des fonctions Rust et le modèle matériel ne sont pas encore implémentés.** La fenêtre est un placeholder et ne représente pas une ROM exécutée.

## Références

- **[mstan/gbarecomp](https://github.com/mstan/gbarecomp)** — référence technique principale pour l'analyse statique ARM/Thumb et le modèle matériel GBA. Le projet amont est principalement en C++ et utilise PolyForm Noncommercial 1.0.0 ; ses mécanismes de secours à l'exécution ne sont pas adoptés ici. Voir [l'audit technique](audit.md), [l'inventaire des fichiers](docs/repository-audit.md) et [les références](docs/reference-projects.md).
- **[morluto/rea](https://github.com/morluto/rea)** — outil d'ingénierie inverse via CLI/MCP et moteurs d'analyse natifs. Le workflow REA prépare un bundle d'analyse ; il ne prétend pas exécuter une session d'agent dans GitHub Actions.

## Développement Linux

Voir [le guide de développement](docs/development.md), [l'architecture](docs/architecture.md), [les décisions](docs/decisions/ADR-0001-static-rust-recompilation.md) et [les workflows](docs/workflows.md). La version de Bevy est épinglée à `0.20.0`, et `rust-toolchain.toml` fixe la version de Rust utilisée.

La ROM de développement doit être fournie localement et rester ignorée par Git. Une ROM, un BIOS, une sauvegarde ou des données dérivées ne doivent jamais être inclus dans les artefacts ou releases.

## Workflows

| Workflow | Déclenchement | Rôle |
| --- | --- | --- |
| CI (Linux) | PR vers `main`, push sur `main`, lancement manuel | Hygiène, formatage Rust, Clippy et tests |
| Build (Linux) | Après la CI verte d'un push sur `main` | Une compilation release, paquet de distribution + entrée REA |
| Release (manual) | Déclenchement manuel | Validation tag/build, publication de l'artefact existant sans recompiler |
| REA analysis bundle (manual) | Déclenchement manuel | Vérification et préparation d'un bundle REA depuis l'artefact existant |

## Maintenance de la documentation

À chaque groupe de travail significatif, relire et mettre à jour si nécessaire **README.md**, **audit.md**, **docs/repository-audit.md** et **SESSION_HANDOFF.md**. L'inventaire et l'audit doivent refléter le dépôt réel et les résultats de CI, pas des intentions. Le handoff consigne explicitement les fichiers documentaires revus, les validations réellement exécutées, le commit et les risques restant ouverts.

Voir [la stratégie des workflows](docs/workflows.md) et la [passation de session](SESSION_HANDOFF.md).
