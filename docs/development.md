# Développement local Linux

## Outils et versions

- Linux x86_64 uniquement pour cette phase.
- Rust `1.98.1` est épinglé dans `rust-toolchain.toml` ; il dépasse le MSRV déclaré par Bevy `0.20.0` (`1.97.1`).
- Bevy est fixé exactement à `=0.20.0` dans `Cargo.toml` et `Cargo.lock`.
- Git, Cargo et les outils système listés ci-dessous.

Bevy 0.20 est la release stable publiée le 8 octobre 2026 ; consulter les [notes de version officielles](https://bevy.org/news/bevy-0-20/). Les dépendances système suivent le [guide Linux officiel de Bevy](https://github.com/bevyengine/bevy/blob/v0.20.0/docs/linux_dependencies.md).

## Dépendances Ubuntu

Pour compiler le runtime avec ses fonctionnalités 2D/X11/Wayland activées :

```bash
sudo apt-get update
sudo apt-get install --no-install-recommends \
  build-essential pkg-config \
  libx11-dev libasound2-dev libudev-dev \
  libxkbcommon-x11-0 libxkbcommon-dev \
  libwayland-dev libxcursor-dev libxrandr-dev libxi-dev
```

## Vérifications et exécution

Depuis la racine du dépôt :

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo build --release --locked --workspace --bins
cargo run --release --locked
```

La première tranche de décodeurs, découverte CFG, IR et émission est fusionnée par PR #4. Le run CI de fusion et les runs courants sont consultables dans [GitHub Actions](https://github.com/rakmahefa/gbarecomp-rust/actions). L'environnement de préparation de la session précédente ne disposait pas d'une toolchain Rust locale : ne pas présenter une compilation locale comme exécutée. Le test du générateur compile puis exécute le Rust émis avec rustc.

La première compilation Bevy est lourde ; le workflow CI met en cache le registre et les artefacts Cargo pour réutiliser le travail entre exécutions compatibles.

## Commandes de recompilation statique

Le binaire de développement `gbarecomp-recomp` est distinct du runtime Bevy `gbarecomp-rust`. Il charge une ROM locale fournie par l'utilisateur ; il n'inclut aucune image dans Git ou dans les artefacts.

```bash
cargo run --locked --bin gbarecomp-recomp -- inspect rom/testrom.gba
cargo run --locked --bin gbarecomp-recomp -- analyze rom/testrom.gba --max-instructions 250000
cargo run --locked --bin gbarecomp-recomp -- recompile rom/testrom.gba --emit target/generated/game.rs
```

`analyze` affiche les blocs détectés et les diagnostics. `recompile` ne produit le fichier qu'en absence de diagnostic et refuse d'écraser un fichier existant. La tranche actuelle émet un sous-ensemble ARM/Thumb limité ; elle ne prétend pas supporter un jeu complet. Les erreurs pour instruction inconnue, branche indirecte, cible non résolue ou chevauchement code/données sont intentionnelles : il n'y a aucun interpréteur ou JIT de secours. Avant d'étendre le décodeur ARM, corriger et tester le rejet des encodages avec condition `0xF` avant les masques BX/SWI (PR #5 fermée non fusionnée).

## Parseur d'en-tête et ROM locale

Le parseur actuel inspecte les champs fixes du header de 0xC0 octets, expose le mot d'instruction ARM de branchement aux offsets 0..4 (pas sa destination résolue) et rapporte si le complement checksum correspond. `GbaRom` mappe les trois fenêtres Game Pak CPU vers les offsets de fichier et conserve la fenêtre wait-state. Une taille ROM puissance de deux utilise un masque pour les alias ; une taille non puissance de deux hors fichier est rejetée explicitement. Ces règles de mapping sont une première tranche à valider, pas un modèle matériel complet. Les tests n'utilisent que des images synthétiques ; aucune ROM réelle n'est requise.

La ROM de test doit être présente localement dans `rom/` et ignorée par Git. Vérifier avec :

```bash
git check-ignore -v rom/testrom.gba
```

Ne jamais inclure une ROM, un BIOS, une sauvegarde ou des données propriétaires dans les releases ou les artefacts. Les paquets sont construits depuis une liste blanche de fichiers compilés, non depuis l'archive du dépôt.
