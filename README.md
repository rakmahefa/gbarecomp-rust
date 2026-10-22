# GBARecomp Rust

Projet expérimental de **recompilation statique GBA qui génère du code Rust natif**, avec **Bevy comme runtime hôte**. Cible initiale : **Linux x86_64 uniquement**.

L'objectif est d'analyser le code machine ARM7TDMI (ARM et Thumb) d'une ROM GBA, de générer des fonctions Rust équivalentes, puis de compiler ces sources avant lancement avec Cargo/rustc. Le modèle matériel GBA reste la source de vérité de l'état invité ; Bevy assure la fenêtre, la présentation, l'entrée et l'intégration hôte.

> **Invariant non négociable : génération Rust statique uniquement. Aucun interpréteur invité, aucun JIT, aucune traduction de code invité à l'exécution.** Un flux de contrôle qui ne peut pas être résolu statiquement doit échouer à l'étape de recompilation avec un diagnostic, sans no-op silencieux ni fallback d'exécution.

## État du projet

Le dépôt contient maintenant une première tranche verticale de recompilation **statique** : image ROM et mapping Game Pak, décodeurs ARM/Thumb conservateurs, IR typée, découverte CFG des branches directes et émission de fonctions Rust pour un sous-ensemble étroit. La CLI hors ligne gbarecomp-recomp propose inspect, analyze et recompile --emit. Le commit fusionné sur main (5fa506d) a échoué au contrôle rustfmt ; la branche de suivi corrige ce formatage et ajoute des tests sémantiques pour PC+8 ARM et le rejet des encodages ARM avec condition 0xF. Cette version de suivi attend encore la validation CI.

Cette tranche sait abaisser les NOP exacts, quelques opérations ARM data-processing immédiates, MOVS immédiat Thumb et branches directes/conditionnelles. Toute autre instruction, cible indirecte, code/data overlap ou budget épuisé entraîne un diagnostic et bloque l'émission. Ce n'est **pas** encore un recompilateur complet ou un jeu exécutable : pas de bus/MMIO avec effets, IRQ/DMA, PPU/audio, fonctions de support complètes, résolution exhaustive code/données ou runtime généré relié à la scène Bevy. Les tests utilisent des ROM synthétiques ; aucune ROM réelle n'est requise.

## Références

- **[mstan/gbarecomp](https://github.com/mstan/gbarecomp)** — référence technique principale pour l'analyse ARM/Thumb et le modèle matériel GBA. Le projet amont est principalement en C++ et utilise PolyForm Noncommercial 1.0.0 ; ses mécanismes de secours à l'exécution ne sont pas adoptés ici. Voir [l'audit technique](audit.md), [l'inventaire des fichiers](docs/repository-audit.md) et [les références](docs/reference-projects.md).
- **[morluto/rea](https://github.com/morluto/rea)** — outil d'ingénierie inverse via CLI/MCP et moteurs d'analyse natifs. Le workflow REA prépare un bundle d'analyse ; il ne prétend pas exécuter une session d'agent dans GitHub Actions.

## Développement Linux

Voir [le guide de développement](docs/development.md), [l'architecture](docs/architecture.md), [l'audit de référence](reference-audit.md), [les décisions](docs/decisions/ADR-0001-static-rust-recompilation.md) et [les workflows](docs/workflows.md). Bevy est épinglé à `0.20.0`, et `rust-toolchain.toml` fixe Rust à `1.98.1`.

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
