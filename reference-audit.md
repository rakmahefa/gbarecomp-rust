# Audit de référence GBARecomp — port Rust

Date : 2026-10-22
Dépôt cible : [rakmahefa/gbarecomp-rust](https://github.com/rakmahefa/gbarecomp-rust)
Référence principale : [mstan/gbarecomp](https://github.com/mstan/gbarecomp), branche `main`, arbre observé au commit `9266573123fb950dfdee2d19040a2daf5e63e39d`.
But : transformer la référence en plan d'implémentation concret pour une recompilation statique GBA en Rust, Linux x86_64 d'abord, avec Bevy limité à la couche hôte.

## 1. Conclusion exécutive

La référence est une base technique très utile, mais **il n'existe pas de port Rust complet et automatique**. L'arbre inspecté contient 491 fichiers texte/binaires suivis, dont un analyseur et un runtime C++ substantiels, des modèles matériels, des outils d'oracle, du packaging multi-plateforme et du code généré. Une réécriture correcte demande de traduire les contrats sémantiques, pas seulement la syntaxe.

Le dépôt cible était au départ un squelette de Phase 0 : documentation/workflows, mais aucun `Cargo.toml`, source Rust ou crate compilable. La première livraison réalisable est donc une base Rust testable (inspection ROM, primitives CPU/mémoire, décodeurs ARM/Thumb et analyse exploratoire), puis une montée en couverture par tranches validées. Elle ne doit pas être annoncée comme un recompilateur complet avant l'émission de code natif exécutable et la validation différentielle.

### Décision de portage et licence

Le fichier `LICENSE` de la référence contient la **PolyForm Noncommercial License 1.0.0**, copyright Matthew Stanley (2026). Elle autorise les usages non commerciaux décrits par ses termes et permet modifications/œuvres dérivées dans ces limites ; elle ne confère pas de droit d'usage commercial et interdit de supposer que l'on peut re-licencier librement la référence. La licence complète et ses définitions priment sur ce résumé.

- Avant tout portage direct/derivé, faire confirmer que l'usage et la distribution visés restent non commerciaux au sens de cette licence, ou obtenir une licence distincte du mainteneur.
- Pour un port direct autorisé, garder les avis de copyright et le texte de licence requis, marquer les fichiers dérivés et examiner les licences des dépendances/sous-modules individuellement.
- Le dépôt Rust n'a actuellement pas de licence propre définie. Ne pas lui ajouter par défaut une licence permissive incompatible avec le code importé.
- L'implémentation de la première tranche peut être écrite indépendamment depuis les contrats publics de l'ISA ARM7TDMI et du matériel GBA, en utilisant le dépôt comme référence de comportement et de structure. Aucun extrait de code source C++ n'est nécessaire pour ce bootstrap.
- Ne pas copier le BIOS, une ROM, du code généré depuis une ROM/BIOS, des tables dérivées propriétaires, des artefacts build ou des données de jeu. Les droits de ces éléments sont distincts de ceux du framework.

Ce document est une revue technique d'ingénierie, pas un avis juridique.

## 2. Cartographie fonctionnelle de la référence

| Zone de référence | Rôle observé | Valeur pour le projet Rust | Traitement |
| --- | --- | --- | --- |
| `src/armv4t/` | État ARM7TDMI, interprétation, sémantique du pipeline, dispatch, symbol lookup | Critique : socle de vérité pour flags, shifts, PC visible, banked regs, interworking | Réécriture prioritaire et testée instruction par instruction ; aucune instruction inconnue silencieusement ignorée |
| `src/recompile/` | Découverte de fonctions/callbacks, génération de fonctions, dispatch et templates RAM | Critique : passage de la découverte au code natif | Réimplémenter sous forme d'étapes séparées : découverte CFG, IR documentée, validation, émission Rust et dispatch |
| `src/gba/gba_bus.*`, `gba_memory.*`, `gba_io.*` | Carte mémoire, bus, accès MMIO et effets | Critique : frontière observable entre code invité et matériel | Commencer avec cartographie stricte et erreurs explicites ; ajouter ensuite side-effects, accès alignés, open-bus et timing selon tests |
| `src/gba/gba_irq.*`, `gba_dma.*`, `gba_timers.*`, scheduler/waitstates | Interruptions, transferts, horloges et séquencement | Très élevée ; erreurs de timing peuvent casser de nombreux jeux | Modéliser en événements déterministes ; tests spécifiques et oracle |
| `src/gba/gba_ppu.*`, `gba_audio.*`, `gba_m4a.*`, `mp2k_shadow.*` | Vidéo, rendu des lignes/sprites, canaux audio et DSP | Très élevée mais volumineuse | Port progressif ; garder état matériel canonique indépendant du rendu/audio hôte Bevy |
| `src/gba/gba_save.*`, `gba_rtc.*`, GPIO, gyro, solar, wireless, serial, link hub, matrix memory | Sauvegardes et périphériques de cartouche | Importants selon la ROM cible | Interfaces de périphériques, implémentations au besoin ; aucun comportement simulé comme « réussi » sans effet correct |
| `src/runtime/bios_hle.*`, `src/runtime/heal_gate.*`, `self_heal.h`, runtime dispatch/overlays | Services BIOS, repli d'exécution, auto-guérison vers un cache natif | Important pour la couverture statique et les zones dynamiques | Port après le noyau : chaque miss doit être visible, compté, rapporté et traité explicitement |
| `src/debug/` | Replay, snapshots, cosimulation, traces, serveur de debug et provenance | Indispensable pour les régressions reproductibles | Port par priorité : traces/replay d'abord, serveur réseau plus tard |
| `tests/armv4t/` et autres tests | Smoke tests de décodeur, interpréteur, symboles et dispatch | Très réutilisables comme catalogue de cas/contrats | Réexprimer en tests Rust indépendants ; ajouter cas négatifs et tests de propriétés |
| `oracle/` | Scripts de comparaison et pilotage mGBA/oracle, diff de mémoire/vidéo/audio/timing | Très forte valeur QA ; majoritairement Python/C | Conserver comme référence de stratégie, réécrire les outils essentiels en Python/Rust ou isoler ceux sans dépendance propriétaire ; documenter setup |
| `tools/`, config d'analyse et scripts auxiliaires | Génération et diagnostic développeur | Variable selon dépendances et hypothèses d'environnement | Inventorier au fur et à mesure ; pas d'import massif non compris |
| `platform/`, `packaging/` | SDL, Android, web/WASM, paquets natifs et scripts de release | Faible priorité pour le premier objectif Linux + Bevy | Ne pas porter en bloc ; remplacer la couche hôte par Bevy et les workflows du projet |
| `src/runtime/generated_bios/` et sorties générées | Tables générées depuis des éléments externes | Risque de provenance/redistribution | Exclure du bootstrap et exiger une provenance/licence claire avant toute génération |
| `third_party/`, submodules | Dépendances et bibliothèques externes | Réutilisation conditionnée à leurs licences/versions | Examiner individuellement ; ne jamais conclure que LICENSE racine couvre tout sans vérifier |

## 3. Ordre de port recommandé

### P0 — Contrats de base
1. Manifestes Cargo et CLI reproductibles.
2. Chargement ROM local, vérification de l'en-tête et des bornes.
3. `CpuState`, flags/conditions, règles PC ARM (+8) et Thumb (+4).
4. Carte mémoire GBA documentée et accès explicites/read-only ROM.
5. Décodeurs ARM/Thumb couvrant d'abord familles de contrôle de flux et data-processing.
6. Tests unitaires à partir des encodages ARM7TDMI et cas synthétiques.

### P1 — Analyse et IR
1. Découverte par worklist des blocs accessibles, branches conditionnelles, appels et sorties indirectes.
2. Provenance de chaque découverte : adresse, état ARM/Thumb, opcode brut, type de transition et niveau de confiance.
3. IR sans ambiguïté pour flags/shifter, mémoire, appels, SWI et contrôle de flux.
4. Représentation explicite des instructions non prises en charge ; aucune conversion en NOP.
5. Rapport exploitable par la CI/REA et inventaire des chemins non résolus.

### P2 — Émission et exécution
1. Émettre du code Rust natif à partir de l'IR ; commencer par un sous-ensemble étroit et testable.
2. Valider les sorties générées (formatage, compilation, linking, erreurs structurées).
3. Préciser ABI, dispatch, appel indirect, exceptions, code en RAM et self-heal avant de qualifier la couverture.
4. Faire diff du même cas contre l'interpréteur de référence et mGBA.

### P3 — Périphériques/temps réel
1. Bus/MMIO avec effets et largeur d'accès.
2. IRQ, timers, DMA et ordonnanceur cycle/event-aware.
3. PPU, audio, sauvegardes, RTC et périphériques selon le jeu cible.
4. Traces déterministes, snapshots et tests de non-régression par frame.

### P4 — Runtime hôte et livraison
1. Bevy pour fenêtre, entrées et surface d'affichage ; aucune sémantique matérielle dans le seul état de rendu.
2. Pipeline audio hôte sans bloquer la simulation.
3. Artefacts Linux, symboles pour REA, provenance des données et rapports de couverture.
4. Autres plateformes seulement après stabilité du modèle commun.

## 4. Contrats de fidélité à préserver

- ARM/Thumb sont deux états du même CPU. BX/BLX, chargement vers PC, retour d'exception et cibles indirectes doivent conserver les transitions d'état.
- La valeur visible du PC dépend de l'état et de l'instruction ; ne pas l'implémenter comme une lecture simple de `regs[15]`.
- Condition codes, carry-out du barrel shifter, débordements et opérations de drapeau doivent être testés séparément.
- Les accès mémoire peuvent déclencher des effets matériels ; ne pas traiter les MMIO comme de la RAM ordinaire.
- Les priorités et le séquencement IRQ/IE/IF/IME, DMA, timers et PPU doivent être déterministes.
- Les appels indirects, données interprétées comme code, code RAM/dynamique et BIOS manquant doivent produire une trace ou un état non pris en charge, jamais une réussite silencieuse.
- Le rendu Bevy reflète l'état de la PPU ; il ne le remplace pas.
- « Exécuté » ne signifie pas « recompilé statiquement » : rapporter le nombre/les PC natifs, interprétés ou guéris par cache lorsqu'une telle exécution existera.

## 5. Éléments exclus de toute réutilisation automatique

- ROM de jeu, BIOS matériel ou leurs dumps, sorties de désassemblage contenant des blocs dérivés et fichiers de données extraits.
- Tables générées à partir du BIOS, code généré spécifique à un jeu et configurations de ROM dont les droits/provenance n'ont pas été examinés.
- Binaire compilé, snapshots ou saves des utilisateurs.
- Couche Android/web/SDL sans adaptation explicite à la cible Linux/Bevy.
- Tout fichier d'un sous-module ou tiers avant audit de sa licence propre.

## 6. Critères d'acceptation du port

Une tranche est considérée intégrée uniquement si :
1. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (quand configuré) et `cargo test` réussissent dans l'environnement cible.
2. Les tests documentent les instructions/cas couverts et les limites.
3. Toute opération non prise en charge renvoie une erreur contextualisée (PC/opcode), sans NOP implicite.
4. Aucun ROM/BIOS ou fichier généré interdit n'est ajouté.
5. Le handoff, l'audit et l'architecture décrivent ce qui est réellement implémenté et les validations exécutées.
6. Le différentiel mGBA/oracle reste une étape requise avant d'affirmer la fidélité matérielle ; compilation seule ne suffit pas.

## 7. Tranche initiale visée après cet audit

Ajouter un crate sans dépendances externes au noyau afin de rendre les tests reproductibles :
- parsing d'en-tête ROM ;
- primitives d'état CPU/conditions ;
- bus mémoire borné avec ROM read-only ;
- décodeurs ARM et Thumb explicitant les opcodes inconnus ;
- exploration CFG limitée aux adresses dans la ROM et rapport d'analyse ;
- CLI `inspect` et `analyze`.

Cette tranche constitue un **bootstrap technique**, pas encore un interpréteur ARM7TDMI complet, un générateur de code natif complet, ni un modèle matériel fidèle. Elle permettra de continuer les ports prioritaires avec des tests au lieu de copier aveuglément les 491 fichiers de la référence.
