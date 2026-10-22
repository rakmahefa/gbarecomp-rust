# Architecture cible

## Objectif

Analyser le code machine ARM7TDMI (ARM et Thumb) d'une ROM GBA et générer du **code source Rust** qui conserve les effets observables du programme invité. Cargo/rustc compilent ce code en fonctions natives avant l'exécution. Bevy est le runtime hôte pour la fenêtre, le rendu, l'input et l'audio hôte ; il ne réécrit pas le jeu et ne remplace pas le modèle matériel GBA.

**Aucun interpréteur invité ou JIT n'est autorisé.** Le binaire distribué ne décode ni ne compile du code invité à l'exécution. Voir [ADR-0001](decisions/ADR-0001-static-rust-recompilation.md).

La référence [mstan/gbarecomp](https://github.com/mstan/gbarecomp) sert à étudier les fonctions, les frontières CPU/matériel et les cas de fidélité. Son implémentation C++ et sa licence PolyForm Noncommercial restent distinctes. Ses chemins de repli à l'exécution ne font pas partie de l'architecture cible de ce projet.

## Couches proposées

1. **En-tête et identification de cartouche** — le premier composant lit l'en-tête fixe GBA de 0xC0 octets, expose le point d'entrée et les champs de titre/code/maker/version, puis calcule le complement checksum. Un checksum incohérent est actuellement signalé comme métadonnée diagnostique ; les politiques de rejet relèveront d'une étape de validation ultérieure. Le chargement de l'image complète, son hash et sa cartographie mémoire restent à implémenter.
2. **Analyse statique ARM/Thumb** — découverte du code et des blocs, état ARM/Thumb, branches, appels, tables littérales, données vs code et cibles indirectes résolues. Les hypothèses doivent être traçables.
3. **IR et validation sémantique** — représentation intermédiaire indépendante de Bevy ; sémantique des flags, conditions, shifter, effets mémoire et contrat d'appel. Tests sur cas isolés avant émission native.
4. **Émission Rust** — génération lisible et reproductible des fonctions Rust. La sortie est compilée par Cargo/rustc pendant la construction, avant livraison ou lancement du jeu recompilé.
5. **Bus et modèle matériel GBA** — carte mémoire, registres E/S, IRQ, DMA, timers, PPU, audio, entrée et sauvegardes. Les effets suivent les contrats matériels documentés.
6. **Runtime hôte Bevy** — ordonnance l'application et présente l'image/audio produits par le modèle matériel. La surface de présentation reste distincte de l'état matériel canonique.
7. **Validation différentielle** — jeux de tests ARM/Thumb, traces reproductibles et comparaison des effets observables avec un oracle documenté.

## Contrat en cas d'échec statique

- Les instructions sans abaissement correct, cibles de branche non résolues, appels indirects à ensemble de cibles incomplet, chevauchements code/données ambigus et régions exécutables générées/modifiées au runtime entraînent des erreurs de recompilation contextualisées par adresse.
- Aucun de ces cas n'est remplacé par un no-op, un interpréteur, un compilateur JIT ou du décodage runtime.
- Un titre qui dépend d'un chemin impossible à résoudre statiquement reste explicitement non pris en charge jusqu'à l'implémentation d'une solution statique correcte.
- Chaque optimisation doit être couverte par des tests sémantiques ; une compilation réussie n'est pas une preuve de fidélité.

## Décisions restant à prendre

- Format et invariants de l'IR.
- Représentation des flags ARM7TDMI et des transferts mémoire.
- Contrat d'appel des fonctions générées et convention d'interworking ARM/Thumb.
- Résolution des tables de dispatch indirectes et des données imbriquées dans les régions de code.
- Stratégie statique pour les cas de code dépendant d'état mutable ; les cas non résolus doivent être rejetés.
- Modélisation déterministe des événements matériels et cycle d'exécution.
- Politique de conservation des sources Rust générées et des symboles d'analyse.
