# Références techniques

## GBARecomp — référence principale

Dépôt : [mstan/gbarecomp](https://github.com/mstan/gbarecomp)

Le README et le guide d'architecture décrivent une recompilation statique ARM/Thumb vers du code natif, liée à un runtime partagé qui modélise les effets matériels GBA. C'est la référence technique principale pour identifier les problèmes à couvrir : découverte de fonctions, transitions ARM/Thumb, dispatch, accès mémoire, PPU/audio/DMA/timers/IRQ, sauvegardes et chemins de repli.

À lire en priorité :

- [README](https://github.com/mstan/gbarecomp/blob/main/README.md)
- [Architecture](https://github.com/mstan/gbarecomp/blob/main/docs/ARCHITECTURE.md)
- [Getting started](https://github.com/mstan/gbarecomp/blob/main/docs/GETTING_STARTED.md)
- [Principles](https://github.com/mstan/gbarecomp/blob/main/PRINCIPLES.md)
- [GBA accuracy burndown](https://github.com/mstan/gbarecomp/blob/main/GBA_ACCURACY_BURNDOWN.md)

Le guide Getting started actuel du dépôt référence un chemin C++/CMake/Windows. Ne pas copier ses commandes comme si elles constituaient un build Rust/Linux. Le modèle de couche et les problèmes de fidélité sont pertinents ; l'outillage doit être adapté séparément.

**Licence :** le dépôt indique PolyForm Noncommercial 1.0.0. Toute adaptation directe doit rester dans les limites de cette licence ou être couverte par une autorisation distincte. Voir [reference-audit.md](../reference-audit.md) pour le plan de portage détaillé.

## REA — outil d'analyse inverse

Dépôt : [morluto/rea](https://github.com/morluto/rea)

REA fournit une interface CLI/MCP et des workflows d'analyse pour les binaires et les applications. Certaines tâches d'analyse native dépendent d'un moteur configuré (par exemple Ghidra, Hopper ou IDA) et d'une session d'agent connectée. Le workflow Actions de ce dépôt ne prétend pas installer ni piloter automatiquement un environnement GUI/agent complet.

- [README et installation](https://github.com/morluto/rea#readme)
- [Guide d'installation](https://github.com/morluto/rea/blob/main/docs/installation.md)
- [Catalogue des outils MCP](https://github.com/morluto/rea/blob/main/docs/mcp-contracts.md)
- [Licence MIT](https://github.com/morluto/rea/blob/main/LICENSE)

Le package REA utilisé par le workflow est épinglé à rea-agents@6.7.0. Mettre à jour cette version seulement lors d'un groupe de travail qui vérifie la compatibilité et note le changement dans SESSION_HANDOFF.md.
