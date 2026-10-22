# GBARecomp Rust

Projet expérimental de **recompilation statique GBA qui génère du code Rust natif**, avec **Bevy comme runtime hôte**. Cible initiale : **Linux x86_64 uniquement**.

L'objectif est d'analyser le code machine ARM7TDMI (ARM et Thumb) d'une ROM GBA, de générer des fonctions Rust équivalentes, puis de compiler ces sources avant lancement avec Cargo/rustc. Le modèle matériel GBA reste la source de vérité de l'état invité ; Bevy assure la fenêtre, la présentation, l'entrée et l'intégration hôte.

> **Invariant non négociable : génération Rust statique uniquement. Aucun interpréteur invité, aucun JIT, aucune traduction de code invité à l'exécution.** Un flux de contrôle qui ne peut pas être résolu statiquement doit échouer à l'étape de recompilation avec un diagnostic, sans no-op silencieux ni fallback d'exécution.

## État du projet

Le scaffold Rust/Bevy et les contrôles fmt, Clippy et tests passent la CI Linux ([run 39746990377](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39746990377), commit `a59ffc334db03e54c473d5367f5b9afcd1f84393`). Le parseur initial de l'en-tête GBA et ses tests synthétiques ont passé la CI Linux au commit `c96edb7bed71ba14f09bb857160d09621a9d849e` ([run 39748310841](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39748310841)). Le champ du branchement initial est renommé ici pour distinguer le mot d'instruction de sa destination, que l'analyse statique devra résoudre ; ce correctif attend sa CI.

Le parseur lit les champs fixes de l'en-tête de 0xC0 octets, expose le mot d'instruction ARM de branchement aux offsets 0..4 (sans prétendre avoir résolu sa destination), et signale la validité du checksum. Ses tests utilisent une fixture synthétique ; aucune ROM réelle n'est nécessaire pour ces tests. Le chargement complet de ROM, la cartographie des régions, le décodeur ARM/Thumb, l'IR, l'émission de fonctions Rust et le modèle matériel ne sont pas encore implémentés. La fenêtre Bevy reste un placeholder et ne représente pas un jeu exécuté.

## Références

- **[mstan/gbarecomp](https://github.com/mstan/gbarecomp)** — référence technique principale pour l'analyse ARM/Thumb et le modèle matériel GBA. Le projet amont est principalement en C++ et utilise PolyForm Noncommercial 1.0.0 ; ses mécanismes de secours à l'exécution ne sont pas adoptés ici. Voir [l'audit technique](audit.md), [l'inventaire des fichiers](docs/repository-audit.md) et [les références](docs/reference-projects.md).
- **[morluto/rea](https://github.com/morluto/rea)** — outil d'ingénierie inverse via CLI/MCP et moteurs d'analyse natifs. Le workflow REA prépare un bundle d'analyse ; il ne prétend pas exécuter une session d'agent dans GitHub Actions.

## Développement Linux

Voir [le guide de développement](docs/development.md), [l'architecture](docs/architecture.md), [les décisions](docs/decisions/ADR-0001-static-rust-recompilation.md) et [les workflows](docs/workflows.md). Bevy est épinglé à `0.20.0`, et `rust-toolchain.toml` fixe Rust à `1.98.1`.

La ROM de développement doit être fournie localement et rester ignorée par Git. Une ROM, un BIOS, une sauvegarde ou des données dérivées ne doivent jamais être inclus dans les artefacts ou releases.

## Workflows

| Workflow | Déclenchement | Rôle |
| --- | --- | --- |
| CI (Linux) | PR vers `main`, push sur `main`, lancement manuel | Hygiène, formatage Rust, Clippy et tests |
| Build (Linux) | Après la CI verte d'un push sur `main` | Une compilation release, paquet de distribution + entrée REA |
| Release (manual) | Déclenchement manuel | Validation tag/build, publication de l'artefact existant sans recompiler |
| REA analysis bundle (manual) | Déclenchement manuel | Vérification et préparation d'un bundle REA depuis l'artefact existant |

## Maintenance de la documentation

À chaque groupe de travail significatif, relire et mettre à jour si nécessaire **README.md**, **audit.md**, **docs/repository-audit.md** et **SESSION_HANDOFF.md**. L'audit et l'inventaire doivent refléter le dépôt réel et les résultats de CI, pas des intentions. Le handoff consigne les fichiers revus, les validations réellement exécutées, le SHA du commit et les risques restant ouverts. Faire une revue complète de l'inventaire avant chaque release.
