# Stratégie GitHub Actions

## Dépendances et ordre d'exécution

1. **CI (Linux)** tourne automatiquement sur PR vers `main` et push à `main`. Elle vérifie le patch, rejette les ROM/BIOS/sauvegardes suivies et exécute formatage, Clippy et tests en mode verrouillé (`Cargo.lock`).
2. **Build (Linux)** s'exécute via `workflow_run` après succès de la CI d'un push sur `main`. Il compile en release une seule fois avec les symboles nécessaires à l'analyse.
3. Cette compilation alimente deux paquets : `gbarecomp-linux-x86_64`, dont le binaire de distribution est strip, et `rea-analysis-input`, qui conserve les symboles et les rapports ELF. Pas de second build pour REA.
4. **Release (manual)** prend l'ID d'un build réussi et un tag existant. Elle valide que le tag pointe exactement sur le commit sélectionné, contrôle le checksum et la présence de fichiers ROM/save, puis publie l'artefact existant. Aucun compilateur Rust ne tourne dans ce workflow.
5. **REA analysis bundle (manual)** télécharge l'artefact d'analyse d'un build donné, vérifie le checksum et produit un bundle documenté. Il vérifie la CLI REA épinglée mais ne prétend pas exécuter une investigation autonome menée par un agent dans GitHub Actions.

## Règles de concurrence et coût

- La CI annule les jobs obsolètes pour la même PR/branche.
- Le build ne démarre qu'après la CI verte sur `main`, jamais en parallèle des checks d'une PR.
- Le build réutilise le cache Cargo, n'effectue qu'une compilation native release et partage le résultat entre distribution et REA.
- Release et REA sont manuels et ne compilent rien ; ils sélectionnent un build avec son identifiant de run.
- Une seule plateforme cible est définie : Linux x86_64.

## Artefacts et sécurité

- Retention des artefacts Actions : 14 jours.
- La release refuse un tag qui ne correspond pas au commit du build et n'écrase pas une release existante.
- La CI échoue si des fichiers ROM/BIOS/sauvegarde restent suivis.
- L'archive release est construite par liste blanche et analysée avant publication ; aucune ROM, aucun BIOS, aucune sauvegarde ni code ROM dérivé n'est ajouté.
- Permissions du token limitées selon le workflow. Aucun secret externe n'est requis pour les builds de base.
- Les versions d'actions GitHub sont en tags majeurs ; avant une publication de long terme, envisager des pins SHA complets.

## Limite d'analyse

Le bundle REA contient le binaire hôte avec symboles, les rapports ELF, les métadonnées et les consignes. Il ne contient ni ROM, ni BIOS, ni code Rust généré à partir d'une ROM. REA sert à explorer les binaires et les traces hors de l'exécution principale ; le runtime livré reste exclusivement statique, sans interpréteur ni JIT.


## Audit de ROM privée (manuel)

Le workflow **Private ROM audit (manual)** est séparé du build de distribution et du bundle REA standard. Il doit être lancé depuis `main`, utilise l'environnement GitHub `roms-audit` et récupère `rakmahefa/roms` avec le secret `ROMS_DEPLOY_KEY` (deploy key SSH en lecture seule). La configuration est décrite dans [private-rom-workflow.md](private-rom-workflow.md).

Il compile la CLI `gbarecomp-recomp`, lance `inspect` et `analyze` sur l'unique fichier `.gba` à la racine du dépôt privé, avec un budget d'instructions limité. Il n'effectue pas de recompilation complète du jeu et n'essaie pas d'émettre du code généré.

**Sorties et confidentialité :** le dépôt contenant les workflows est public. Le résumé ne montre que des compteurs agrégés ; la ROM, les sorties détaillées de l'analyse, le désassemblage et les éventuels codes générés ne sont pas téléversés en artefact ni en release. Les rapports bruts temporaires sont supprimés à la fin du job. Ne pas ajouter de future étape qui imprime ou téléverse les détails ROM-dérivés sans mettre ces sorties dans un environnement privé.

Le secret est un secret d'environnement plutôt qu'un secret global du dépôt. L'environnement doit autoriser uniquement la branche `main`. Les workflows CI, Build, Release et REA existants ne reçoivent pas ce secret et restent indépendants de la ROM privée.
