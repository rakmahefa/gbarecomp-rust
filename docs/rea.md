# Utiliser REA avec ce projet

## Ce que fait le workflow

Le workflow manuel REA analysis bundle ne lance pas une session d'agent autonome. Il télécharge l'artefact d'analyse produit par le build Linux (le binaire hôte avec symboles et les rapports ELF), vérifie son checksum et l'enveloppe avec une consigne d'analyse et un manifeste. Il n'effectue aucune compilation.

C'est volontaire : REA expose des outils CLI/MCP et des workflows pilotés par agent ; plusieurs voies d'analyse native ont besoin d'un moteur local configuré et d'une session d'agent. Un runner GitHub standard ne doit pas prétendre avoir ce contexte.

## Procédure

1. Laisser passer CI puis Build (Linux) sur le commit à examiner.
2. Copier l'identifiant du run Build (Linux) réussi.
3. Lancer manuellement REA analysis bundle, en passant cet identifiant.
4. Télécharger l'artefact rea-investigation-bundle depuis le run REA.
5. Décompresser le bundle dans l'environnement où REA et un moteur d'analyse sont configurés. Commencer par REA_TASK.md, manifest.json et les rapports ELF ; ne pas exécuter le binaire analysé par défaut.

Le workflow vérifie l'installation CLI de REA via le paquet épinglé rea-agents@6.7.0. Cela vérifie l'accès à l'outil, pas la réussite d'une analyse sémantique automatisée.

## Cibles d'analyse prioritaires

- Point d'entrée natif, dépendances dynamiques et résolution des ressources.
- Organisation des symboles, fonctions et chemins d'initialisation de Bevy.
- Frontière entre le runtime hôte et le code de recompilation invité lorsqu'il existera.
- Evidence : adresses, noms de symboles, extraits désassemblés et degré de certitude. Séparer les faits observés des hypothèses.
- Ne jamais mettre la ROM, un BIOS ou les données extraites dans le bundle. En phase initiale, le bundle porte uniquement le binaire hôte et les métadonnées de build.
