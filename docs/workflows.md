# Stratégie GitHub Actions

## Dépendances et ordre d'exécution

La stratégie évite de compiler dans quatre workflows distincts :

1. **CI (Linux)** démarre automatiquement sur pull request vers main et sur push à main. Elle réalise les contrôles génériques de dépôt et, quand Cargo.toml existe, cargo fmt, Clippy et les tests.
2. **Build (Linux)** est déclenché par l'événement workflow_run après la fin de CI. Il ne poursuit que pour un push réussi sur main. La compilation release se déroule donc après la CI, pas en concurrence avec elle.
3. Le build produit deux artefacts à partir de la même compilation : gbarecomp-linux-x86_64 (paquet distribuable) et rea-analysis-input (binaire avec symboles et rapports ELF). L'archive de release est construite par liste blanche ; le répertoire ROM n'est jamais copié.
4. **Release (manual)** reçoit l'identifiant d'un run Build et un tag. Il vérifie que le run a réussi, que le tag désigne exactement le même commit, puis télécharge l'artefact existant et crée la release. Aucune compilation n'y est faite.
5. **REA analysis bundle (manual)** reçoit l'identifiant d'un run Build, télécharge rea-analysis-input, vérifie le checksum et produit un bundle annoté. Aucune compilation n'y est faite.

## Règles de concurrence

- La CI annule une exécution devenue obsolète pour le même PR/branche.
- Le build Linux utilise un groupe de concurrence commun à main et peut annuler un build plus ancien si un nouveau commit arrive.
- La release et REA sont manuels et ne déclenchent pas de build. Ils réutilisent les artefacts par run ID.
- Pas de planification périodique, de matrice OS ou d'architecture : Linux x86_64 seulement pour la première phase.

## Artefacts et sécurité

- Durée de rétention : 14 jours. Pour une publication tardive, relancer le build au bon commit plutôt que reconstruire implicitement dans la release.
- Le workflow de release refuse un tag qui ne désigne pas le commit du build sélectionné.
- CI échoue si une ROM ou une sauvegarde reste suivie dans le dépôt.
- Les archives de release sont inspectées pour vérifier qu'elles ne contiennent pas de fichiers ROM/save reconnus.
- Aucun secret externe n'est requis ; les permissions du token sont restreintes par workflow.
- Les workflows s'appuient sur des versions majeures d'actions GitHub courantes. Avant un usage de publication à long terme, épingler les actions tierces par SHA complet selon la politique de sécurité du projet.

## Limite actuelle de Phase 0

Sans Cargo.toml, aucun binaire ni artefact n'est produit : le workflow Build termine avec un résumé explicite indiquant que le projet Rust n'est pas encore initialisé. Après ajout du crate, il exige au moins une cible binaire. Il faudra étendre la liste blanche de packaging si le runtime Bevy nécessite des ressources supplémentaires.
