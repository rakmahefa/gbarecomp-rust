# Architecture cible

## Objectif

Analyser le code machine ARM7TDMI (ARM et Thumb) d'une ROM GBA et générer du **code source Rust** qui conserve les effets observables du programme invité. Cargo/rustc compilent ce code en fonctions natives avant l'exécution. Bevy est le runtime hôte pour la fenêtre, le rendu, l'input et l'audio hôte ; il ne réécrit pas le jeu et ne remplace pas le modèle matériel GBA.

**Aucun interpréteur invité ou JIT n'est autorisé.** Le binaire distribué ne décode ni ne compile du code invité à l'exécution. Voir [ADR-0001](decisions/ADR-0001-static-rust-recompilation.md).

La référence [mstan/gbarecomp](https://github.com/mstan/gbarecomp) sert à étudier les fonctions, les frontières CPU/matériel et les cas de fidélité. Son implémentation C++ et sa licence PolyForm Noncommercial restent distinctes. Ses chemins de repli à l'exécution ne font pas partie de l'architecture cible de ce projet.

## État réellement implémenté

Le code contient désormais une première tranche pour le chargement d'une image fournie en mémoire, le mapping des trois fenêtres Game Pak, le décodage conservateur de familles ARM/Thumb, une IR typée, le parcours depuis l'instruction de branchement du header et un émetteur de source Rust. Le CLI correspondant est `gbarecomp-recomp`.

Pour les opérations ARM data-processing qui lisent R15, le PC visible est l'adresse de l'instruction + 8, pas la valeur persistée dans le registre hôte ; le générateur émet cette valeur explicitement. Les encodages ARM avec champ condition 0xF sont rejetés avant les masques d'instructions conditionnelles. La CI de ces correctifs reste à confirmer.

L'émetteur est volontairement limité : NOP ARM exact, certaines opérations ARM data-processing à immédiat, MOVS immédiat Thumb, branchements directs et conditionnels. Il bloque la sortie dès qu'une cible ou une instruction ne peut pas être résolue par cette tranche. Il ne modélise pas encore les effets mémoire ni les périphériques GBA, et la sortie émise n'est pas intégrée à un exécutable de jeu/Bevy. Les tests du générateur compilent et exécutent une fixture synthétique.

## Couches proposées

1. **En-tête et identification de cartouche** — le parseur lit l'en-tête fixe GBA de 0xC0 octets, expose le mot d'instruction ARM de branchement aux offsets 0..4 (pas la destination déjà résolue) et les champs de titre/code/maker/version, puis calcule le complement checksum. L'objet `GbaRom` conserve les octets et mappe les trois fenêtres Game Pak 0x08/0x0A/0x0C vers des offsets ; l'identité de la fenêtre wait-state reste disponible. Un checksum incohérent est actuellement signalé comme métadonnée diagnostique ; les politiques de rejet relèveront d'une étape de validation ultérieure. Le hash de l'image, l'analyse des fonctions et les régions non-ROM de la cartographie complète restent à implémenter. Voir [la carte mémoire GBA](https://gbadev.net/gbadoc/memory.html).
2. **Analyse statique ARM/Thumb** — une découverte initiale couvre déjà branches directes/conditionnelles et certaines instructions immédiates. Les appels indirects, tables, fonctions, pointeurs code, code/data overlap avancé et interworking complet restent à développer.
3. **IR et validation sémantique** — l'IR couvre actuellement NOP, opérations ARM immédiates choisies et MOVS Thumb immédiat. Les opérations de registres, décalages, accès mémoire, exceptions et modèle complet des flags doivent être ajoutés avec tests.
4. **Émission Rust** — l'émetteur natif génère actuellement des fonctions de blocs autonomes pour le sous-ensemble validé et une table de dispatch statique ; un test compile puis exécute le Rust généré. Le lien au modèle matériel et au build de distribution d'un jeu reste à faire.
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
