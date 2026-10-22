# Session handoff

Dernière mise à jour : 2026-10-22 (revue des workflows)
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

## Revue de configuration après le premier commit

- Le workflow REA installe explicitement Node.js 24.11.0, compatible avec les engines déclarés par la version REA épinglée.
- La vérification d'existence d'une release reçoit GH_TOKEN ; elle ne doit pas dépendre d'une lecture GitHub non authentifiée.
- Le verrou de concurrence du build est limité au job éligible : un événement PR ignoré ne peut pas annuler un build main.
- Le paquet public utilise une copie strip du binaire ; le bundle REA conserve les symboles, à partir de la même compilation.
- Cette correction est dans un commit séparé et ce handoff est mis à jour avec lui.

## Règle pour les prochaines sessions

**Mettre à jour ce fichier dans chaque commit qui clôt un groupe de travail.** Ajouter la date, les changements, les validations réellement exécutées, les artefacts/commits concernés, les problèmes connus et la prochaine action. Ne jamais marquer une vérification comme réussie si elle n'a pas été exécutée.
