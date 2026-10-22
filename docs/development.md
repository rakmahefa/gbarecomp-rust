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

La CI Linux précédente a validé formatage, Clippy et tests du scaffold. La nouvelle tranche du parseur d'en-tête utilise uniquement un en-tête GBA synthétique dans les tests ; les tests ne dépendent pas de la ROM de développement locale. La fenêtre Bevy est compilée, mais son lancement interactif n'a pas encore été vérifié localement.

La première compilation Bevy est lourde ; le workflow CI met en cache le registre et les artefacts Cargo pour réutiliser le travail entre exécutions compatibles.

## Parseur d'en-tête et ROM locale

Le parseur actuel inspecte les champs fixes du header de 0xC0 octets, expose le mot d'instruction ARM de branchement aux offsets 0..4 (pas sa destination résolue) et rapporte si le complement checksum correspond. Une entrée trop courte produit une erreur structurelle. Un checksum invalide est signalé sans masquer les autres champs ; il ne provoque pas encore un rejet automatique. Ceci ne constitue ni un chargeur de ROM complet, ni un décodeur ARM/Thumb, ni une recompilation.

La ROM de test doit être présente localement dans `rom/` et ignorée par Git. Vérifier avec :

```bash
git check-ignore -v rom/testrom.gba
```

Ne jamais inclure une ROM, un BIOS, une sauvegarde ou des données propriétaires dans les releases ou les artefacts. Les paquets sont construits depuis une liste blanche de fichiers compilés, non depuis l'archive du dépôt.
