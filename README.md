# GBARecomp Rust

Projet expérimental de **recompilation statique GBA qui génère du code Rust natif**, avec **Bevy comme runtime hôte**. Cible initiale : **Linux x86_64 uniquement**.

L'objectif est d'analyser le code machine ARM7TDMI (ARM et Thumb) d'une ROM GBA, de générer des fonctions Rust équivalentes, puis de compiler ces sources avant lancement avec Cargo/rustc. Le modèle matériel GBA reste la source de vérité de l'état invité ; Bevy assure la fenêtre, la présentation, l'entrée et l'intégration hôte.

> **Invariant non négociable : génération Rust statique uniquement. Aucun interpréteur invité, aucun JIT, aucune traduction de code invité à l'exécution.** Un flux de contrôle qui ne peut pas être résolu statiquement doit échouer à l'étape de recompilation avec un diagnostic, sans no-op silencieux ni fallback d'exécution.

## État du projet

La PR #4 a été fusionnée dans `main` au commit `fe9e1df5b74ccb686723adeea495419249e08d2f`. Elle apporte une première tranche verticale de recompilation **statique** : image ROM et mapping Game Pak, décodeurs ARM/Thumb conservateurs, IR typée, découverte CFG des branches directes et émission de fonctions Rust pour un sous-ensemble étroit. La CLI hors ligne `gbarecomp-recomp` propose `inspect`, `analyze` et `recompile --emit`. La validation post-fusion est suivie dans [GitHub Actions](https://github.com/rakmahefa/gbarecomp-rust/actions).

Cette tranche sait abaisser les NOP exacts, un sous-ensemble ARM data-processing immédiat (dont RSB/CMN), MOVS/CMP/ADDS/SUBS immédiats Thumb et branches directes/conditionnelles. Toute autre instruction, cible indirecte, code/data overlap ou budget épuisé entraîne un diagnostic et bloque l'émission. Ce n'est **pas** encore un recompilateur complet ou un jeu exécutable : le bus/MMIO, IRQ/DMA, PPU/audio, la résolution exhaustive code/données et le runtime généré relié à Bevy restent à construire. Les tests utilisent des ROM synthétiques ; aucune ROM réelle n'est requise.

**Point de reprise connu :** la PR #5 a été fermée sans fusion. Son correctif qui rejette la condition ARM `0xF` avant les masques BX/SWI n'est donc pas dans `main`. Voir [SESSION_HANDOFF.md](SESSION_HANDOFF.md) : ce bug de décodage doit être traité avant d'étendre la couverture ARM.

## Références

- **[mstan/gbarecomp](https://github.com/mstan/gbarecomp)** — référence technique principale pour l'analyse ARM/Thumb et le modèle matériel GBA. Le projet amont est principalement en C++ et utilise PolyForm Noncommercial 1.0.0 ; ses mécanismes de secours à l'exécution ne sont pas adoptés ici. Voir [l'audit technique](audit.md), [l'inventaire des fichiers](docs/repository-audit.md) et [les références](docs/reference-projects.md).
- **[morluto/rea](https://github.com/morluto/rea)** — outil d'ingénierie inverse via CLI/MCP et moteurs d'analyse natifs. Le workflow REA prépare un bundle d'analyse ; il ne prétend pas exécuter une session d'agent dans GitHub Actions.

## Développement Linux

Voir [le guide de développement](docs/development.md), [l'architecture](docs/architecture.md), [l'audit de référence](reference-audit.md), [les décisions](docs/decisions/ADR-0001-static-rust-recompilation.md), [les workflows](docs/workflows.md) et la [procédure d'accès à la ROM privée](docs/private-rom-workflow.md). Bevy est épinglé à `0.20.0`, et `rust-toolchain.toml` fixe Rust à `1.98.1`.

Pour le développement local, la ROM de test reste locale et ignorée par Git. La ROM Emerald du dépôt privé `rakmahefa/roms` peut aussi être inspectée via le workflow manuel **Private ROM audit (manual)**, après configuration du secret SSH en lecture seule. Voir [la procédure d'accès privé](docs/private-rom-workflow.md). Ce workflow ne prouve pas la compatibilité complète et ne publie ni ROM, ni désassemblage détaillé, ni code généré. Les releases et artefacts de distribution restent indépendants de la ROM.

## Workflows

| Workflow | Déclenchement | Rôle |
| --- | --- | --- |
| CI (Linux) | PR vers `main`, push sur `main`, lancement manuel | Hygiène, formatage Rust, Clippy et tests |
| Build (Linux) | Après la CI verte d'un push sur `main` | Une compilation release, paquet de distribution + entrée REA |
| Release (manual) | Déclenchement manuel | Validation tag/build, publication de l'artefact existant sans recompiler |
| REA analysis bundle (manual) | Déclenchement manuel | Vérification et préparation d'un bundle REA du binaire hôte depuis l'artefact existant |
| Private ROM audit (manual) | Déclenchement manuel depuis `main` | Inspection et analyse bornée de la ROM privée ; résumé agrégé seulement, aucun artefact ROM-dérivé détaillé |

## Maintenance de la documentation

À chaque groupe de travail significatif, relire et mettre à jour si nécessaire **README.md**, **audit.md**, **docs/repository-audit.md** et **SESSION_HANDOFF.md**. L'audit et l'inventaire doivent refléter le dépôt réel et les résultats de CI, pas des intentions. Le handoff consigne les fichiers revus, les validations réellement exécutées, le SHA du commit et les risques restant ouverts. Faire une revue complète de l'inventaire avant chaque release.
