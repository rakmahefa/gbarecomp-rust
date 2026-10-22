# GBARecomp Rust

Projet expérimental de **recompilation statique Game Boy Advance en Rust**, avec **Bevy comme runtime hôte**. Cible initiale : **Linux x86_64 uniquement**.

L'objectif est de traduire du code machine ARM7TDMI (ARM et Thumb) issu d'une ROM GBA en code natif, puis de l'exécuter contre un modèle explicite du matériel GBA. Bevy doit fournir l'intégration hôte — fenêtre, rendu et boucle d'application — sans devenir une réécriture du jeu ni remplacer le modèle matériel par des approximations de présentation.

## État du projet

Le crate Rust et la CLI de base sont maintenant présents. Cette tranche comprend l'inspection de l'en-tête ROM, l'état architectural CPU, un bus mémoire initial, les décodeurs ARM/Thumb et une exploration conservatrice du flot de contrôle. Elle ne constitue **pas** encore un recompilateur complet : l'IR, l'émission de code natif, l'interpréteur complet et les effets matériels (MMIO, IRQ, DMA, PPU/audio) restent à construire et à valider. L'absence de build à ce stade ne sera pas présentée comme un build réussi.

## Références du projet

- **[mstan/gbarecomp](https://github.com/mstan/gbarecomp)** : référence technique principale. Son arbre principal inspecté contient 491 fichiers suivis et sa licence est PolyForm Noncommercial 1.0.0 ; voir [l'audit de portage](reference-audit.md) avant toute réutilisation directe ou dérivée.
- **[morluto/rea](https://github.com/morluto/rea)** : boîte à outils d'ingénierie inverse utilisable depuis un agent/MCP et des moteurs d'analyse natifs. La licence du dépôt REA est MIT. Le workflow de ce dépôt prépare un bundle d'analyse réutilisable ; il ne prétend pas exécuter une session d'agent interactive dans GitHub Actions.

## Développement local

1. Installer Rust stable et Cargo sur Linux.
2. Garder la ROM de test dans le répertoire rom/ uniquement sur la machine de développement. Les ROM ne sont pas des sources du projet et ne doivent pas être commitées ni incluses dans une release.
3. Lire [le guide de développement](docs/development.md), [l'architecture cible](docs/architecture.md), [l'audit de référence](reference-audit.md) et [les workflows](docs/workflows.md).
4. Consulter SESSION_HANDOFF.md avant de reprendre une session et le mettre à jour dans chaque commit qui constitue un groupe de travail.

## Workflows GitHub Actions

| Workflow | Déclenchement | Rôle |
| --- | --- | --- |
| CI (Linux) | PR vers main, push sur main, lancement manuel | Hygiène du dépôt et vérifications Rust si Cargo est présent |
| Build (Linux) | Après la CI verte d'un push sur main | Un seul build release, deux artefacts réutilisables |
| Release (manual) | Manuel | Vérifie tag + build, puis publie l'artefact existant sans recompiler |
| REA analysis bundle (manual) | Manuel | Récupère l'artefact de build, vérifie son intégrité et prépare le paquet REA |

La stratégie détaillée des workflows se trouve dans [docs/workflows.md](docs/workflows.md). [audit.md](audit.md) conserve l'audit initial de Phase 0 ; [reference-audit.md](reference-audit.md) décrit la cartographie des composants à porter et les limites de licence.

## Licence et ROM

La licence propre au dépôt reste à décider. Le bootstrap ajouté ici est écrit indépendamment, sans copie d'extraits du code C++ de référence. Tout portage direct/dérivé de mstan/gbarecomp doit respecter PolyForm Noncommercial 1.0.0 ou faire l'objet d'une autorisation commerciale distincte ; voir reference-audit.md.

Les droits d'utilisation et de redistribution de la ROM de test doivent être vérifiés séparément. Le fichier ROM présent dans l'historique initial a été retiré de l'arbre de travail préparé par cette Phase 0 ; la suppression d'un fichier ne purge pas les anciens commits Git. Détails dans [audit.md](audit.md).
