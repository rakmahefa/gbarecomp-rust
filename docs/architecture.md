# Architecture cible

## Objectif

Recompiler le code machine ARM7TDMI (ARM et Thumb) d'une version de ROM GBA vers un programme natif Rust, puis l'exécuter contre un modèle explicite du matériel GBA. Ce projet ne doit pas devenir une réécriture du jeu en logique de haut niveau.

La référence mstan/gbarecomp est utile pour comprendre les frontières entre analyse, traduction et runtime matériel. Son implémentation C++ et sa licence propre restent distinctes de ce projet Rust.

## État de la première tranche

Déjà présents dans le crate : parsing de l'en-tête, lecture d'adresses du ROM, primitives CPU/flags/conditions, carte mémoire provisoire, décodeurs de familles ARM/Thumb et parcours de flot de contrôle limité à la ROM. Les commandes sont `gbarecomp inspect <rom.gba>` et `gbarecomp analyze <rom.gba> [max-instructions]`.

Ce niveau reste exploratoire : pas d'émission de code Rust natif, de table de dispatch exécutable, de sémantique d'interpréteur complète ni de MMIO cycle-accurate. Les sorties inconnues sont signalées ; le rapport d'analyse ne prouve pas encore que les octets identifiés sont tous du code.

## Couches proposées

1. **Entrée et identification** — lecture locale de la ROM, taille, hash, révision et cartographie des régions. Les données propriétaires ne sont pas intégrées à Git ou aux artefacts.
2. **Analyse statique ARM/Thumb** — découverte des fonctions et des blocs, état ARM/Thumb, branches et appels, références littérales, données vs code et cibles indirectes connues. Les hypothèses doivent être traçables.
3. **IR et recompilateur** — représentation intermédiaire documentée, sémantique des flags, conditions, shifter, accès mémoire et ABI d'appel ; génération native avec tests différentiels.
4. **Bus et modèle matériel GBA** — carte mémoire, E/S, IRQ, DMA, timers, PPU, audio, clavier et sauvegardes. Les effets doivent suivre les contrats matériels identifiés, pas simplement rendre un écran.
5. **Runtime hôte Bevy** — boucle application, fenêtre, rendu, gestion d'entrées, audio hôte et intégration plateforme. Bevy orchestre la présentation ; il ne remplace pas la sémantique du programme invité ou des registres GBA.
6. **Validation** — tests unitaires de l'IR, jeux de tests ARM/Thumb, traces reproductibles, comparaison avec un émulateur de référence et tests d'intégration utilisant la ROM uniquement en local.

## Règles de conception

- Préserver une frontière claire entre code invité recompilé et effets matériels.
- Aucune instruction, cible ou effet inconnu ne doit être transformé silencieusement en opération vide.
- Documenter le niveau de confiance des découvertes statiques et les chemins de repli (interpréteur ou dispatch dynamique) nécessaires pour les cas non résolus.
- Distinguer l'état matériel canonique de la surface de présentation Bevy, notamment si des améliorations visuelles sont explorées plus tard.
- Toute optimisation doit être validée par des tests qui comparent des effets observables, pas seulement par une compilation réussie.
- Garder l'analyse ROM et la génération de code dérivé dans des sorties locales ignorées. Ne jamais ajouter ces sorties aux artefacts de release par un glob du dépôt.

## Décisions ouvertes

- Format et niveau de détail de l'IR.
- Contrat ABI des fonctions recompilées et gestion des appels indirects.
- Stratégie pour le code en RAM / auto-modifiant.
- Granularité des événements matériel et déterminisme de l'ordonnanceur.
- Compatibilité de version de Rust, Bevy et bibliothèques Linux.
- Choix de licence du code nouveau et politique des dépendances.
