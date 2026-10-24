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


## Analyse ROM et import des résultats (manuel)

Le workflow **Private ROM analysis (manual)** vit dans le dépôt privé `rakmahefa/roms`. Il checkout la ROM localement au runner privé, checkout le code public `gbarecomp-rust@main` sans secret cross-repo, construit `gbarecomp-recomp`, produit les rapports `inspect`/`analyze` et exécute REA/Ghidra sur le binaire natif public.

Il publie deux artefacts séparés dans le dépôt privé :
- `rom-analysis-private` conserve les détails d'inspection ROM pour 14 jours.
- `rom-analysis-public` contient uniquement un résumé agrégé, les checksums et le résultat REA du binaire natif du dépôt public. Il exclut l'empreinte de la ROM, son nom, les rapports bruts et le code généré.

Le workflow **Import private ROM analysis result (manual)** dans le dépôt public prend un Run ID privé. Avec un jeton fine-grained limité à la seule permission `Actions: read` sur `rakmahefa/roms`, il vérifie le succès du run, la liste blanche des fichiers, le schéma et les checksums, puis importe uniquement `rom-analysis-public`. L'artefact importé devient public : les données détaillées de la ROM ne doivent donc jamais y être incluses. La procédure est dans [private-rom-workflow.md](private-rom-workflow.md).

Cette analyse demeure bornée. REA analyse le binaire natif de l'outil public et non la ROM GBA ; aucun succès ne signifie que Pokémon Emerald a été entièrement recompilé ou démarre. Après validation de la nouvelle chaîne, retirer l'ancien secret `roms-audit / ROMS_DEPLOY_KEY` et l'ancienne deploy key si elle ne sert plus.
