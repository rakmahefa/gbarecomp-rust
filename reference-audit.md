# Audit de référence — portage Rust de GBARecomp

- Date : 2026-10-22
- Projet cible : [rakmahefa/gbarecomp-rust](https://github.com/rakmahefa/gbarecomp-rust), branche de travail `phase-1/rust-bevy-static-only`
- Référence inspectée : [mstan/gbarecomp](https://github.com/mstan/gbarecomp), branche `main`, commit `9266573123fb950dfdee2d19040a2daf5e63e39d`
- Périmètre : inventaire fonctionnel du code source, contrat de licence, recommandations de portage et ordre d’implémentation. Ce document ne constitue pas un avis juridique.

## 1. Décision principale

La référence est un framework C++/CMake complet et beaucoup plus avancé qu’un parseur ARM/Thumb isolé. L’arbre analysé contenait **491 fichiers suivis**. Il comprend la recherche de fonctions, l’émission de code, les sémantiques ARM7TDMI, le modèle matériel GBA, des tests, des outils de comparaison avec un émulateur oracle et des couches de distribution multiplateformes.

Le dépôt Rust n’est pas destiné à devenir une traduction syntaxique de 491 fichiers C++. La bonne méthode est un portage par contrats observables : choisir un sous-système, documenter ses invariants, écrire une implémentation Rust autonome, puis la valider par tests unitaires et différentiels avant d’élargir la couverture.

### Licence : bloqueur pour tout port direct

Le fichier `LICENSE` de la référence indique **PolyForm Noncommercial License 1.0.0** (copyright Matthew Stanley, 2026). Cette licence autorise les usages non commerciaux définis par ses termes et permet des modifications dans ces limites, mais ne confère pas un droit d’usage commercial ni un droit général de re-licencier le code dérivé comme permissif.

- Avant tout portage copiant ou adaptant le code source, vérifier l’usage projeté et la distribution au regard du texte complet de la licence, ou obtenir une licence distincte auprès du mainteneur (contact commercial indiqué dans la licence : https://1379.tech).
- Conserver les avis de copyright/licence et identifier les fichiers dérivés lorsque le port est autorisé.
- Auditer séparément les sous-modules et dépendances de `third_party/` : la licence racine ne remplace pas nécessairement leurs licences.
- L’implémentation initiale doit être écrite indépendamment depuis la spécification publique ARM7TDMI/GBA et ses propres tests, en utilisant la référence pour identifier les contrats et scénarios à couvrir. Les extraits C++ ne sont pas copiés dans cette tranche.
- Ne pas importer de ROM de jeu, dump de BIOS, code/tables générés depuis une ROM ou un BIOS, assets, saves ou données dérivées d’un jeu. Ces contenus ont une provenance et des droits distincts.

Le dépôt Rust doit encore choisir explicitement sa licence propre avant une distribution publique étendue. Ne pas annoncer une compatibilité de licence supposée.

## 2. Carte des sous-systèmes de référence

| Source amont | Responsabilité constatée | Valeur pour Rust | Stratégie d’adaptation |
| --- | --- | --- | --- |
| `src/armv4t/` | État CPU ARM7TDMI, décodage/interprétation, pipeline, conditions, symboles et dispatch | Fondamental : sémantique des instructions ARM/Thumb | Construire une IR typée et un décodeur incrémental ; tester PC visible (+8 ARM / +4 Thumb), shifter/carry, flags, exceptions et interworking. Aucun fallback d’interpréteur à l’exécution, conformément à ADR-0001 |
| `src/recompile/` | Découverte de fonctions et callbacks, génération de fonctions, dispatch, templates RAM | Le noyau du portage | Séparer le parcours de graphe de flot de contrôle (CFG), la classification code/données, la IR, la validation de couverture et l’émission source Rust compilée par Cargo |
| `src/gba/gba_bus.*`, `gba_memory.*`, `gba_io.*` | Carte d’adresses, bus et registres mémoire-mappés | Frontière canonique entre CPU invité et matériel | Ajouter une représentation de régions et des accès typés ; les lectures/écritures MMIO devront déclencher les effets exacts au lieu de traiter les registres comme de simples octets |
| `src/gba/gba_irq.*`, `gba_dma.*`, `gba_timers.*`, ordonnanceur/waitstates | Interruptions, DMA, timers et coûts d’accès | Critique pour la progression du CPU et la fidélité temporelle | Ordonnanceur déterministe et tests indépendants par périphérique ; définir ordre des événements avant l’optimisation |
| `src/gba/gba_ppu.*`, `gba_audio.*`, `gba_m4a.*`, `mp2k_shadow.*` | Vidéo, sprites, canaux audio et DSP | Très grand volume fonctionnel | Porter progressivement et valider par images/audio ; Bevy n’est que le frontend et ne devient jamais l’état matériel canonique |
| `src/gba/gba_save.*`, RTC, GPIO, gyroscope, capteur solaire, serial/link, wireless, matrix memory | Matériel de sauvegarde et périphériques de cartouche | Variable suivant le jeu, parfois indispensable au démarrage | Interfaces de périphériques puis implémentations ciblées, avec tests du protocole observable |
| `src/runtime/bios_hle.*`, dispatch, overlays, self-heal et runtime principal | Services BIOS et repli/cache dynamique dans l’implémentation amont | Informations utiles sur les cas difficiles et la couverture | Adapter seulement les contrats compatibles avec l’architecture Rust. L’ADR-0001 interdit le chemin d’interprétation/JIT en runtime : instruction non supportée, code invité dynamique ou cible inconnue doivent échouer à la recompilation |
| `src/debug/` | Replay, snapshots, cosimulation, traces et interface de diagnostic | Forte valeur pour la reproductibilité | Commencer par journaux structurés et traces déterministes, puis replay/snapshots ; le serveur réseau vient plus tard |
| `tests/armv4t/` et autres tests | Cas de décodage, d’interpréteur, de symbol lookup et dispatch | Catalogue de scénarios et de régressions | Réexprimer les cas de manière indépendante en Rust ; enrichir avec cas limites et comparaisons sémantiques |
| `oracle/` | Outils Python/C de pilotage mGBA et différences mémoire/vidéo/audio/timing | Très forte valeur de validation | Conserver le protocole de test comme modèle ; isoler les outils de développement, écrire les comparateurs indispensables et documenter les versions d’émulateurs |
| `tools/` et configurations d’analyse | Scripts de génération et de diagnostic | Utile après compréhension de chaque contrat | Inventorier et porter à la demande, ne pas copier en masse des scripts liés au build CMake ou à des chemins locaux |
| `platform/`, `packaging/` | SDL, Android, web/WASM et packaging natif | Priorité faible pour Linux x86_64 + Bevy | Remplacer par la frontière hôte déjà choisie dans `src/runtime.rs`; réexaminer les plateformes seulement après stabilité du noyau commun |
| `src/runtime/generated_bios/` et autres fichiers générés | Données générées ou liées à des sources externes | Risque de droits et provenance | Ne pas copier dans le bootstrap ; exiger source, procédure de génération et droits documentés |
| `third_party/`, submodules | Dépendances externes | Réutilisation conditionnelle | Vérifier licence, version, patchs et capacité de remplacement individuellement |

## 3. Contrats techniques à préserver

1. **ARM/Thumb est un seul CPU avec deux états d’instruction.** Les branches BX/BLX, chargements vers PC et retours d’exception doivent conserver la transition d’état.
2. **Le PC visible n’est pas la simple valeur stockée de R15.** ARM expose en général l’adresse courante +8 et Thumb +4, selon le contexte de l’instruction.
3. **Les codes conditionnels, carry, décalages et débordements font partie de la sémantique.** Ils ne peuvent pas être approximés par des opérations Rust ordinaires sans préserver les flags ARM.
4. **L’accès mémoire possède des effets matériels.** Largeur, alignement, aliasing, MMIO, open-bus et side-effects doivent être explicités avant de qualifier le bus.
5. **La découverte statique doit être auditable.** Toute cible calculée non résolue et toute ambiguïté code/données doit produire un diagnostic exploitable.
6. **L’émission est effectuée avant exécution.** Les fonctions Rust générées sont compilées par Cargo/rustc ; le binaire livré ne traduit pas le code invité.
7. **Pas de JIT ou d’interpréteur runtime.** Une instruction non supportée, un appel indirect non résolu ou du code RAM généré à l’exécution échoue avec une erreur située (adresse, opcode/état). Pas de NOP silencieux.
8. **Le rendu hôte n’est pas le modèle du matériel.** Bevy reçoit l’état de la PPU, les entrées et le son depuis les sous-systèmes correspondants ; il ne remplace pas leurs registres ni leur ordonnanceur.
9. **Une compilation réussie ne prouve pas la fidélité.** Le support doit être qualifié par des tests sémantiques et, dès que l’environnement oracle existe, des comparaisons reproductibles avec mGBA (et autres témoins ciblés).

## 4. Ordre d’implémentation proposé

### Phase A — entrée ROM et exploration conservative

- Charger une image ROM locale complète et parser l’en-tête.
- Valider les bornes d’adresse sans inclure une ROM dans Git ou les artefacts.
- Calculer la cible du branchement initial à partir du mot d’instruction brut situé aux offsets 0..3.
- Définir les structures de blocs et les diagnostics de décodeur ; commencer par les branches directes.
- Ne pas classifier automatiquement chaque mot accessible en code : la zone header contient des données, les tables et littéraux peuvent être entrelacés aux fonctions.

### Phase B — décodeur et IR ARM/Thumb

- Ajouter les familles par catégories avec tests couvrant encodages normaux et limites.
- Représenter les effets de flags, accès mémoire, appels, retours et SWI de façon typée.
- Marquer explicitement les encodages non supportés ; le codegen doit refuser les blocs qui ne sont pas prouvés réécrits correctement.
- Résoudre les branches directes d’abord, puis littéraux et tables de saut lorsque la découverte est fiable.

### Phase C — génération de code Rust natif

- Émettre une fonction native par bloc/fonction avec noms et métadonnées d’adresses stables.
- Définir l’ABI, le dispatch entre blocs et les contrats avec CPU/bus.
- Générer à une étape de build et compiler les sources générées ; ne pas créer du code à l’exécution.
- Refuser le build d’un jeu si le rapport indique des cibles critiques non résolues, des instructions non supportées ou une ambiguïté code/données.
- Tester les fonctions générées et le résultat final, pas seulement le texte produit.

### Phase D — matériel et validation différentielle

- Ajouter MMIO et ses effets, puis IRQ/DMA/timers/ordonnanceur, puis PPU/audio/saves/périphériques.
- Mettre en place les fixtures unitaires et le replay déterministe.
- Comparer les états observables avec mGBA au démarrage, par frame et par catégories de périphériques.
- Rapporter les cas non comparés comme non validés plutôt que les déclarer fonctionnels.

### Phase E — hôte et livraison

- Garder Bevy pour fenêtre, entrée et présentation, sans sémantique CPU cachée dans la couche visuelle.
- Faire correspondre les artefacts release et REA au même commit et inclure les symboles uniquement dans le paquet d’analyse.
- Conserver ROM, BIOS, sauvegardes et données locales hors de GitHub et des artefacts.

## 5. Tests et sorties à exiger par tranche

Une tranche est intégrée seulement si :
- `cargo fmt --all -- --check`, Clippy et `cargo test --workspace --all-features --locked` passent dans la CI épinglée ;
- les tests indiquent explicitement la couverture et les limites de la famille d’instructions ;
- une instruction ou un flux inconnu ne se transforme jamais en NOP/retour arbitraire ;
- les rapports mentionnent adresses et octets bruts pour reproduire chaque échec ;
- aucune ROM/BIOS ou sortie dérivée interdite n’a été ajoutée ;
- `README.md`, `audit.md`, `docs/repository-audit.md` et `SESSION_HANDOFF.md` reflètent l’état réel et les résultats CI ;
- la validation différentielle est marquée « non exécutée » tant qu’elle n’a pas réellement été lancée.

## 6. Prochaine tranche de travail concrète

Sur la branche `phase-1/rust-bevy-static-only`, le parseur d’en-tête et le scaffold Bevy existent déjà. Les prochaines modifications devraient donc **étendre ces modules**, non les dupliquer :

1. ajouter le chargement ROM complet dans `src/rom/` ;
2. ajouter des décodeurs ARM/Thumb et une IR sous `src/recompiler/`, en réutilisant le type `InstructionSet` et les erreurs `RecompileError` existants ;
3. calculer et traverser le branchement d’entrée et les cibles de branche directes ;
4. ajouter un générateur de code source Rust borné qui ne peut émettre que des instructions dont la sémantique est implémentée ;
5. refuser explicitement le codegen en cas de diagnostic non résolu ; aucune exécution interprétée/JIT de secours.

L’état courant est un scaffold avec parseur d’en-tête, pas encore un recompilateur de jeux GBA complet.
