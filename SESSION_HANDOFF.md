# Session handoff

Dernière mise à jour : 2026-10-22
Groupe de travail : Phase 0 — bootstrap Linux/CI/artefacts
Branche de préparation : phase-0/bootstrap-linux-workflows

## But du projet

Créer une recompilation statique GBA en Rust, avec Bevy comme runtime hôte. Linux x86_64 uniquement pour la première phase. La référence technique principale est [mstan/gbarecomp](https://github.com/mstan/gbarecomp) ; [morluto/rea](https://github.com/morluto/rea) servira à l'analyse inverse assistée.

## État observé à l'entrée

- Dépôt rakmahefa/gbarecomp-rust, branche par défaut main.
- Le dépôt n'avait qu'un README vide et le répertoire rom/ ; aucun Cargo.toml, Cargo.lock, source Rust ou workflow n'était présent.
- rom/testrom.gba faisait 16 MiB et était suivi par Git dans le dépôt public. Cette branche retire le fichier de l'arbre courant, ajoute des règles d'ignorance locales et garde les instructions dans rom/README.md.
- Le blob demeure accessible dans l'historique antérieur tant que l'historique n'est pas réécrit.
- mstan/gbarecomp est principalement un framework C++/CMake sous PolyForm Noncommercial 1.0.0. S'en servir comme référence technique ne donne pas automatiquement les droits de réutiliser son code.
- morluto/rea publie son dépôt sous MIT. Le package source annonce Node.js ^22.19.0 || ^24.11.0 || >=26.0.0 ; le workflow épingle REA à 6.7.0 pour la reproductibilité initiale.

## Travail inclus dans ce groupe

- README de projet et documentation Phase 0.
- audit.md, architecture cible, développement local, références et stratégie de workflows.
- CI Linux avec contrôles de dépôt toujours actifs et vérifications Rust activées dès que Cargo.toml existe.
- Build Linux séquencé après la CI réussie sur main, avec un artefact de distribution et un bundle d'entrée REA construits depuis le même binaire.
- Release et bundle REA manuels, fondés sur les artefacts produits par le build ; pas de compilation supplémentaire dans ces workflows.
- .gitignore pour les sorties de build et les ROM locales.

## Limites / éléments à vérifier ensuite

- Aucun code Rust ou runtime Bevy n'a encore été ajouté : le build doit donc signaler proprement l'absence de crate et ne produit pas encore d'artefact.
- La configuration des bibliothèques système Linux nécessaires à Bevy dépendra des versions et fonctionnalités réellement retenues.
- Le format du paquet de release devra évoluer si le runtime exige d'autres ressources, bibliothèques dynamiques ou données.
- Le workflow REA prépare un bundle statique (binaire + métadonnées + rapports ELF) ; l'analyse pilotée par agent et moteur natif s'effectue séparément dans l'environnement configuré.
- Il faut décider la licence propre au projet et la politique de purge de l'historique contenant l'ancien blob ROM.

## Règle pour les prochaines sessions

**Mettre à jour ce fichier dans chaque commit qui clôt un groupe de travail.** Ajouter la date, les changements, les validations réellement exécutées, les artefacts/commits concernés, les problèmes connus et la prochaine action. Ne jamais marquer une vérification comme réussie si elle n'a pas été exécutée.
