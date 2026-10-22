# Développement local Linux

## État actuel

Le crate initial est présent : Cargo.toml, Cargo.lock, une bibliothèque sans dépendances tierces, une CLI et des tests unitaires intégrés. Bevy n'est pas encore dépendance : la couche hôte sera ajoutée après stabilisation des frontières du modèle matériel, afin que le rendu ne devienne pas la source de vérité de l'émulation.

## Pré-requis

- Linux x86_64.
- Git.
- Rust stable + Cargo (rustup recommandé).
- Les dépendances système Linux correspondant aux fonctionnalités Bevy effectivement activées. La liste définitive doit être basée sur les versions choisies, plutôt que devinée avant le premier Cargo.toml.

## Commandes

- Format : cargo fmt --all -- --check
- Analyse : cargo clippy --workspace --all-targets --all-features -- -D warnings
- Tests : cargo test --workspace --all-features --locked
- Build : cargo build --release --locked --workspace --bins

Le workflow de build exige un manifeste et au moins une cible binaire. Il compile une seule fois avec les symboles de débogage : la copie de distribution est strip après compilation, tandis que le bundle REA conserve les symboles et les rapports ELF. Aucun second build n'est nécessaire.

## ROM de test

La ROM de développement est un fichier local et ignoré, non une dépendance livrée. Le blob qui avait été suivi précédemment peut demeurer dans l'historique Git ancien ; cette tranche ne réécrit pas l'historique. Vérifier l'ignorance avec git check-ignore -v rom/testrom.gba.

Ne pas inclure une ROM, un BIOS, une sauvegarde ou des données propriétaires dans des archives de release. Les artefacts doivent être construits depuis une liste blanche de sorties compilées et de ressources d'application, jamais depuis une archive complète du dépôt.

## Sorties

- target/ : sorties Cargo ignorées.
- dist/ : paquets locaux générés ignorés.
- Artefacts Actions : temporaires, retention de 14 jours ; ils ne sont pas la source de vérité.
- GitHub Release : canal explicite de publication après sélection manuelle d'un build et vérification du tag.
