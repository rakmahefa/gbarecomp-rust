# ROM de développement locale

Ce dépôt public ne doit contenir aucune ROM, aucun BIOS, aucune sauvegarde et aucun code généré depuis une ROM. Les fichiers de ce répertoire sont ignorés par Git, à l'exception de ce README. Pour les commandes de développement local, place une ROM de test fournie légalement ici et vérifie l'exclusion avec :

    git check-ignore -v rom/testrom.gba

La ROM Pokémon Emerald dédiée aux tests automatisés se trouve dans le dépôt séparé et privé rakmahefa/roms. Le workflow manuel Private ROM audit (manual) la récupère temporairement sous .private/roms au moyen d'un deploy key SSH en lecture seule. La procédure exacte de configuration est dans docs/private-rom-workflow.md.

Le workflow d'audit affiche seulement un résumé agrégé. Il ne publie pas la ROM, le désassemblage brut, le code généré ni des rapports détaillés en artefact. Les releases et artefacts de distribution ne contiennent jamais de ROM, BIOS, sauvegarde ou code dérivé. Cette règle reste applicable même si l'accès privé a été configuré avec succès.
