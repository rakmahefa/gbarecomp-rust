# Références techniques

## GBARecomp — référence technique principale

Dépôt : [mstan/gbarecomp](https://github.com/mstan/gbarecomp)

Le README et l'architecture amont documentent une recompilation statique ARM/Thumb vers du code natif C++ ainsi qu'un runtime partagé modélisant le matériel GBA. Les problèmes à étudier comprennent la découverte de fonctions, l'interworking ARM/Thumb, les cibles indirectes, les accès mémoire, la PPU, l'audio, le DMA, les timers, les IRQ et les sauvegardes.

À lire en priorité :

- [README](https://github.com/mstan/gbarecomp/blob/main/README.md)
- [Architecture](https://github.com/mstan/gbarecomp/blob/main/docs/ARCHITECTURE.md)
- [Getting started](https://github.com/mstan/gbarecomp/blob/main/docs/GETTING_STARTED.md)
- [Principles](https://github.com/mstan/gbarecomp/blob/main/PRINCIPLES.md)
- [GBA accuracy burndown](https://github.com/mstan/gbarecomp/blob/main/GBA_ACCURACY_BURNDOWN.md)

Le guide amont contient un parcours C++/CMake/Windows ; il ne décrit pas le build de ce projet Rust/Linux. La référence sert à étudier l'architecture et les comportements matériels, pas à copier automatiquement le code.

**Différence explicite :** ce projet émet du **Rust** à compiler avant distribution. Les mécanismes amont d'interpréteur/résolution dynamique en runtime ne sont pas adoptés ; les inconnues statiques deviennent des erreurs explicites jusqu'à l'implémentation d'un traitement statique correct.

**Licence :** le dépôt amont indique PolyForm Noncommercial 1.0.0. Étudier ses comportements n'accorde pas le droit de copier son code. Voir [audit.md](../audit.md) et [l'audit de portage](../reference-audit.md) pour les priorités et les limites de réutilisation.

## REA — outil d'analyse inverse

Dépôt : [morluto/rea](https://github.com/morluto/rea)

REA fournit une CLI/MCP et des workflows d'analyse pour binaires et applications. Certaines analyses natives requièrent un moteur configuré (par exemple Ghidra, Hopper ou IDA) et une session d'agent connectée ; le workflow GitHub de ce projet prépare le bundle mais n'émule pas ce contexte.

- [README et installation](https://github.com/morluto/rea#readme)
- [Guide d'installation](https://github.com/morluto/rea/blob/main/docs/installation.md)
- [Catalogue des outils MCP](https://github.com/morluto/rea/blob/main/docs/mcp-contracts.md)
- [Licence MIT](https://github.com/morluto/rea/blob/main/LICENSE)

Le workflow épingle `rea-agents@6.7.0`. Modifier cette version seulement dans un groupe de travail qui vérifie sa compatibilité et met à jour `SESSION_HANDOFF.md`.


## Carte mémoire GBA

Pour les adresses du bus et les fenêtres Game Pak, consulter aussi le [guide mémoire GBA de gbadev](https://gbadev.net/gbadoc/memory.html). Il documente les trois fenêtres ROM de 32 MiB aux bases 0x08000000, 0x0A000000 et 0x0C000000, avec des réglages de wait-state distincts. Ce dépôt implémente son propre mapping Rust ; les détails de mirroring restent couverts par des tests, et les cas qu'on ne sait pas déterminer statiquement échouent explicitement.
