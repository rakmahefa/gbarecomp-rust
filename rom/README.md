# ROM de développement locale

Déposer ici, sur la machine de développement uniquement, la ROM de test fournie légalement pour les tests. Les fichiers de ce répertoire sont ignorés par Git, à l'exception de ce README.

La ROM suivie dans l'historique initial a été retirée de l'arbre courant par la Phase 0. Si elle doit être conservée pour le développement, sauvegarder le fichier hors du checkout avant de fusionner le changement, puis le replacer ici. Confirmer qu'elle est ignorée avec git check-ignore -v rom/testrom.gba.

Ne jamais ajouter la ROM, un BIOS, des sauvegardes ou des données dérivées à un artefact ou à une release.
