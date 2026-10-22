# GBARecomp Rust

Projet expérimental de **recompilation statique Game Boy Advance en Rust**, avec **Bevy comme runtime hôte**. Cible initiale : **Linux x86_64 uniquement**.

L'objectif est de traduire du code machine ARM7TDMI (ARM et Thumb) issu d'une ROM GBA en code natif, puis de l'exécuter contre un modèle explicite du matériel GBA. Bevy doit fournir l'intégration hôte — fenêtre, rendu et boucle d'application — sans devenir une réécriture du jeu ni remplacer le modèle matériel par des approximations de présentation.

## État du projet

Nous sommes en Phase 0 : préparation du dépôt et des workflows. Le dépôt ne contient pas encore de manifeste Cargo ni de crate compilable. Les workflows de CI sont prêts à évoluer avec le code ; le build et les artefacts sont conditionnés à l'existence de Cargo.toml et d'au moins une cible binaire. L'absence de build à ce stade ne sera pas présentée comme un build réussi.

## Références du projet

- **[mstan/gbarecomp](https://github.com/mstan/gbarecomp)** : référence technique principale et dépôt le plus complet identifié pour la recompilation GBA et le runtime matériel. Il s'agit d'un projet C++ avec ses propres contraintes de build et une licence PolyForm Noncommercial ; voir [l'audit](audit.md) avant tout portage de code.
- **[morluto/rea](https://github.com/morluto/rea)** : boîte à outils d'ingénierie inverse utilisable depuis un agent/MCP et des moteurs d'analyse natifs. La licence du dépôt REA est MIT. Le workflow de ce dépôt prépare un bundle d'analyse réutilisable ; il ne prétend pas exécuter une session d'agent interactive dans GitHub Actions.

## Développement local

1. Installer Rust stable et Cargo sur Linux.
2. Garder la ROM de test dans le répertoire rom/ uniquement sur la machine de développement. Les ROM ne sont pas des sources du projet et ne doivent pas être commitées ni incluses dans une release.
3. Lire [le guide de développement](docs/development.md), [l'architecture cible](docs/architecture.md) et [les workflows](docs/workflows.md).
4. Consulter SESSION_HANDOFF.md avant de reprendre une session et le mettre à jour dans chaque commit qui constitue un groupe de travail.

## Workflows GitHub Actions

| Workflow | Déclenchement | Rôle |
| --- | --- | --- |
| CI (Linux) | PR vers main, push sur main, lancement manuel | Hygiène du dépôt et vérifications Rust si Cargo est présent |
| Build (Linux) | Après la CI verte d'un push sur main | Un seul build release, deux artefacts réutilisables |
| Release (manual) | Manuel | Vérifie tag + build, puis publie l'artefact existant sans recompiler |
| REA analysis bundle (manual) | Manuel | Récupère l'artefact de build, vérifie son intégrité et prépare le paquet REA |

La stratégie détaillée et les limites actuelles sont décrites dans [docs/workflows.md](docs/workflows.md). L'audit initial se trouve dans [audit.md](audit.md).

## Licence et ROM

La licence du code propre à ce dépôt reste à choisir avant d'importer ou d'écrire une quantité significative de code. La licence des références n'est pas automatiquement celle de ce projet.

Les droits d'utilisation et de redistribution de la ROM de test doivent être vérifiés séparément. Le fichier ROM présent dans l'historique initial a été retiré de l'arbre de travail préparé par cette Phase 0 ; la suppression d'un fichier ne purge pas les anciens commits Git. Détails dans [audit.md](audit.md).
