# ROM de développement locale

Ce dépôt public ne doit contenir aucune ROM, aucun BIOS, aucune sauvegarde et aucun code généré depuis une ROM. Les fichiers de ce répertoire sont ignorés par Git, à l'exception de ce README. Pour les commandes de développement local, place une ROM de test fournie légalement ici et vérifie l'exclusion avec :

    git check-ignore -v rom/testrom.gba

La ROM Pokémon Emerald dédiée aux tests automatisés se trouve dans le dépôt séparé et privé `rakmahefa/roms`. L'inspection s'exécute dans ce dépôt privé via **Private ROM analysis (manual)**. Le dépôt public importe ensuite uniquement l'artefact assaini via **Import private ROM analysis result (manual)**. La configuration du jeton Actions:read et le déroulé sont décrits dans `docs/private-rom-workflow.md`.

Le workflow d'audit affiche seulement un résumé agrégé. Il ne publie pas la ROM, le désassemblage brut, le code généré ni des rapports détaillés en artefact. Les releases et artefacts de distribution ne contiennent jamais de ROM, BIOS, sauvegarde ou code dérivé. Cette règle reste applicable même si l'accès privé a été configuré avec succès.
