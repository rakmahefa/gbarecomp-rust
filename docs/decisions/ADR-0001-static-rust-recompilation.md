# ADR-0001 — Génération Rust statique uniquement

- **Statut : accepté**
- **Date : 2026-10-22**
- **Portée :** analyseur, générateur de code, binaire livré et runtime.

## Décision

Le recompilateur analyse le code machine GBA ARM7TDMI (ARM et Thumb) et **génère du code source Rust**. Cargo/rustc compilent ce code en code natif avant la livraison.

Le binaire distribué n'embarque **aucun interpréteur d'instructions invité ni compilateur JIT**. Le runtime Bevy héberge le programme natif généré et modélise le matériel GBA ; il n'est pas chargé de décoder ou de traduire du code invité à l'exécution.

## Conséquences

- Les chemins de contrôle doivent être résolus statiquement avec une confiance suffisante pour générer un ensemble complet de fonctions Rust.
- Une instruction sans traduction correcte, un appel indirect sans ensemble statique complet de cibles, une région code/données ambiguë ou du code invité généré/modifié à l'exécution entraîne un diagnostic explicite pendant l'analyse ou la génération.
- Un échec de recompilation n'est jamais transformé en instruction vide, interprétation de secours ou compilation JIT.
- Les tests doivent vérifier la sémantique des fonctions Rust générées et leurs effets matériels observables par rapport à un oracle documenté.
- Si un titre dépend d'un mécanisme qui ne peut pas être résolu dans cette architecture, il reste non pris en charge jusqu'à l'ajout d'une solution **statique** correcte.

## Pourquoi

Ce contrat garde le modèle d'exécution simple, inspectable et déterministe au niveau du code généré. Il évite qu'un chemin de secours à l'exécution masque les lacunes de l'analyse statique ou introduise une deuxième implémentation sémantique des instructions.

## Hors périmètre de cette décision

Le premier scaffold ne contient pas encore l'analyse ARM/Thumb complète, l'IR, le générateur de fonctions ou le modèle matériel. Cette ADR fixe le contrat à respecter par ces composants futurs ; elle ne prétend pas que la recompilation d'un jeu fonctionne déjà.
