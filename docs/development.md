# Développement local Linux

## État actuel

La Phase 0 ne contient pas encore de crate Rust. Avant le premier build réel, ajouter un manifeste Cargo, une cible binaire, un verrou de dépendances commité (Cargo.lock) et le squelette minimal du runtime Bevy. À ce moment, les commandes Cargo de la CI et du build seront activées automatiquement.

## Pré-requis

- Linux x86_64.
- Git.
- Rust stable + Cargo (rustup recommandé).
- Les dépendances système Linux correspondant aux fonctionnalités Bevy effectivement activées. La liste définitive doit être basée sur les versions choisies, plutôt que devinée avant le premier Cargo.toml.

## Commandes prévues quand le crate sera présent

- Format : cargo fmt --all -- --check
- Analyse : cargo clippy --workspace --all-targets --all-features -- -D warnings
- Tests : cargo test --workspace --all-features --locked
- Build : cargo build --release --locked --workspace --bins

Le workflow de build exige un manifeste et au moins une cible binaire. Il préserve des symboles de débogage dans le binaire construit pour l'analyse statique ; la même sortie sert à créer le paquet Linux et le bundle REA afin d'éviter une seconde compilation.

## ROM de test

La ROM de développement est un fichier local et ignoré, non une dépendance livrée. Garder une copie hors du checkout avant de fusionner le changement qui supprime le blob actuellement suivi, puis la replacer dans rom/. Vérifier l'ignorance avec git check-ignore -v rom/testrom.gba.

Ne pas inclure une ROM, un BIOS, une sauvegarde ou des données propriétaires dans des archives de release. Les artefacts doivent être construits depuis une liste blanche de sorties compilées et de ressources d'application, jamais depuis une archive complète du dépôt.

## Sorties

- target/ : sorties Cargo ignorées.
- dist/ : paquets locaux générés ignorés.
- Artefacts Actions : temporaires, retention de 14 jours ; ils ne sont pas la source de vérité.
- GitHub Release : canal explicite de publication après sélection manuelle d'un build et vérification du tag.
