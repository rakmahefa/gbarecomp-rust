# Audit technique — 2026-10-22

## Résumé

La PR #4 a été fusionnée dans `main` au merge commit `fe9e1df5b74ccb686723adeea495419249e08d2f`. Elle apporte des décodeurs ARM/Thumb limités, une IR typée, une découverte conservatrice de branches statiques et un émetteur Rust AOT dont la fixture synthétique est compilée puis exécutée par rustc. Le run post-fusion [39756038257](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39756038257) est vert pour le code fusionné. Cela reste une tranche limitée, pas un recompilateur de jeu complet. Elle n'est pas un recompilateur de jeu complet : le bus matériel, les instructions restantes, la découverte exhaustive des fonctions, les exceptions et le runtime d'un jeu recompilé restent à construire.

## Constatations et actions

### A-01 — ROM GBA précédemment suivie dans un dépôt public

**Sévérité : élevée — action requise si les droits sont incertains.**

Le fichier `rom/testrom.gba` était suivi dans l'historique initial de `main` et faisait 16 777 216 octets (16 MiB). La Phase 0 l'a retiré de l'arbre courant et a ajouté des règles d'ignorance. Il n'est donc plus présent dans l'arbre courant, mais son blob reste accessible dans les anciens commits publics tant que l'historique n'est pas réécrit.

Si sa redistribution publique n'est pas autorisée, une purge d'historique coordonnée (par exemple avec `git-filter-repo`, suivie d'une mise à jour forcée contrôlée) reste nécessaire. La sauvegarde du développeur doit être restaurée localement après le checkout de la branche appropriée, pas ajoutée à Git.

### A-02 — Le recompilateur n'est pas encore implémenté

**Sévérité : élevée pour la couverture fonctionnelle ; attendue à ce stade.**

L'implémentation actuelle conserve ces fonctions d'en-tête et de cartouche, et ajoute `recompiler::arm`, `recompiler::thumb`, `recompiler::analysis`, `recompiler::ir` et `recompiler::codegen`. Le passeur statique suit le branchement d'entrée, certaines opérations ARM data-processing avec immédiat, MOVS/CMP/ADDS/SUBS immédiats Thumb et des branches directes/conditionnelles. Il émet du Rust pour les blocs uniquement si aucun diagnostic n'est présent. Aucun bus/MMIO à effets, aucune PPU/audio, aucune IRQ/DMA, aucun ensemble de cibles indirectes, et aucun jeu complet relié à Bevy n'est encore implémenté ; la compilation du code généré sur fixture synthétique ne prouve pas la compatibilité d'un titre.

### A-03 — Licence de la référence C++ à vérifier avant tout portage de code

**Sévérité : élevée si du code source est réutilisé.**

Le fichier LICENSE de [mstan/gbarecomp](https://github.com/mstan/gbarecomp/blob/main/LICENSE) indique **PolyForm Noncommercial License 1.0.0**. Cette licence ne doit pas être supposée compatible avec une distribution commerciale. Étudier les comportements, le matériel et la conception comme référence n'accorde pas automatiquement le droit de copier ou d'adapter son code source. Aucun code C++ de ce dépôt n'est copié dans le scaffold.

Le dépôt [morluto/rea](https://github.com/morluto/rea/blob/main/LICENSE) est sous MIT. Respecter les notices si des portions de son code sont un jour réutilisées.

### A-04 — Licence propre au nouveau code non définie

**Sévérité : moyenne.**

La licence du code écrit ici doit être décidée explicitement avant une distribution publique plus large. Ne pas ajouter une licence permissive par défaut sans accord du mainteneur. Documenter également les notices tierces et le traitement des ROM, BIOS, sauvegardes et données générées.

### A-05 — Recompilation Rust statique seulement, sans interpréteur ni JIT

**Sévérité : décision architecturale contraignante.**

Le code généré est du Rust compilé avant distribution avec Cargo/rustc. Aucun interpréteur de code invité ni JIT ne sera utilisé à l'exécution. Les cibles indirectes incomplètes, instructions non supportées, ambiguïtés code/données et régions de code générées à l'exécution doivent produire des erreurs de recompilation explicites. Elles ne doivent jamais devenir des opérations silencieuses ni des chemins de secours runtime. La décision est consignée dans [ADR-0001](docs/decisions/ADR-0001-static-rust-recompilation.md).

### A-06 — Fidélité du modèle matériel à établir

**Sévérité : élevée avant toute revendication de compatibilité.**

La recompilation doit préserver les effets observables du CPU ARM7TDMI et des services matériels GBA. Le rendu Bevy est une couche de présentation et ne constitue pas l'état matériel canonique. Les contrats du bus, registres E/S, interruptions, DMA, timers, PPU, audio et sauvegardes doivent être documentés et testés. Aucun support de jeu ne doit être déclaré à partir du seul fait que le binaire compile.

### A-07 — Cohérence de Cargo.lock sous --locked

**État : résolu et validé.**

Le run CI `39745465091` a révélé que le lockfile initial adapté depuis un projet Bevy plus large ne correspondait pas exactement au manifeste minimal. Il a été remplacé par une résolution Bevy 0.20.0 avec la feature 2d seule. Le run [39746308189](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39746308189) a validé Clippy et les tests avec `--locked`. Ne pas retirer `--locked` pour masquer une divergence future.

### A-08 — Type de retour du builder Bevy

**État : résolu et validé.**

Le run `39745818857` a détecté que la chaîne de configuration Bevy retournait `&mut App`, et non `App`. Le builder configure désormais une variable locale et retourne explicitement l'application. La CI [39746308189](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39746308189) passe depuis ce correctif.

### A-09 — Parseur de l'en-tête GBA : périmètre limité

**État : en-tête et renommage validés par la CI ; le parsing structurel ne valide pas la sémantique d'un jeu.**

Le parseur vérifie qu'au moins 0xC0 octets sont disponibles, expose le mot d'instruction ARM de branchement stocké aux offsets 0..4 et les champs fixes, expose les octets de titre/code/maker sans les normaliser et signale le complement checksum calculé. Le checksum incohérent est une information diagnostique et ne fait pas échouer le parsing structurel. Les tests utilisent uniquement une fixture synthétique, jamais la ROM de développement.

Le premier run CI du parseur (`39747920168`) a échoué uniquement sur `cargo fmt --check`, qui demandait de réordonner la liste `pub use` dans `src/rom/mod.rs`. Le correctif de formatage a été validé par [le run CI `39748310841`](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39748310841). Le renommage `entry_branch_opcode` a ensuite été validé au run [39749349856](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39749349856).

Le parseur de header ne décode ni n'exécute le ROM. L'analyse et la génération sont des étapes distinctes de build ajoutées dans la tranche courante ; leur couverture reste limitée et doit être examinée avec A-11.

### A-11 — Découverte ARM/Thumb et source emitter : couverture initiale seulement

**Sévérité : élevée tant que la couverture et la fidélité du modèle matériel ne sont pas démontrées. État : implémentation ajoutée ; CI du commit final à confirmer.**

Le nouveau décodeur reconnaît un ensemble réduit : branchements ARM B/BL, quelques data-processing à immédiat et NOP ARM exact ; Thumb B, branches conditionnelles, MOVS/CMP/ADDS/SUBS immédiats ; autres encodages retournent `Unsupported`. La découverte démarre à 0x08000000, calcule les cibles des branches immédiates, suit les continuations statiquement connues, applique un budget et rapporte les cibles hors ROM, les branches indirectes et chevauchements d'entrées de blocs.

Le générateur refuse une analyse contenant un diagnostic et émet des fonctions Rust spécialisées avec un état registres/CPSR réduit. La fixture de test compile et exécute ce source comme un binaire de test. **Ce n'est pas l'ABI CPU/matériel final** : mémoire/MMIO, les opérations ARM/Thumb non implémentées, les transitions BX/BLX, exceptions/SWI, le traitement exhaustif de code/données, le retour d'appels intégré au runtime et les événements matériels restent hors portée.

Limites à surveiller avant extension :
- les blocs de branches vers une adresse déjà décodée au milieu d'un bloc provoquent actuellement un diagnostic de chevauchement au lieu de scinder le bloc ;
- les paires BL Thumb et cibles indirectes sont rejetées ;
- une fonction de test qui passe n'établit pas une comparaison sémantique différentielle avec un émulateur oracle.

## Cadence de mise à jour de l'audit

À chaque groupe de travail significatif, revoir le présent audit, le README et [l'inventaire détaillé](docs/repository-audit.md), puis noter dans [SESSION_HANDOFF.md](SESSION_HANDOFF.md) la réalité du dépôt, les résultats CI exécutés, le commit et les risques restant ouverts. Faire une revue exhaustive avant chaque release.

## Références de workflows

- CI automatique sur PR et push Linux ; le workflow échoue si une ROM/BIOS/sauvegarde est suivie dans Git.
- Build après succès de la CI sur un push à `main`, pour éviter la concurrence avec celle-ci.
- Release manuelle : vérifie que le tag existant pointe exactement sur le commit du build et réutilise l'artefact.
- Bundle REA manuel : réutilise l'artefact et ne relance aucune compilation.
- Les artefacts temporaires sont conservés 14 jours ; la release taguée est le canal de distribution durable.


### A-10 — Mapping statique des fenêtres Game Pak : policy explicite

**État : code et tests fusionnés dans PR #4 ; le run post-fusion 39756038257 a passé la CI.**

L'abstraction `GbaRom` associe les plages CPU documentées 0x08000000-0x09FFFFFF, 0x0A000000-0x0BFFFFFF et 0x0C000000-0x0DFFFFFF aux fenêtres wait-state 0, 1 et 2. Les fenêtres exposent le même contenu ROM ; le résultat conserve l'identité de la fenêtre pour ne pas perdre l'information de wait-state. La taille maximale acceptée ici est de 32 MiB.

Pour une image dont la taille est une puissance de deux, le mapping utilise un masque de taille pour représenter l'alias des bits d'adresse hauts. Pour les images de taille non puissance de deux, les offsets au-delà des octets fournis deviennent une erreur explicite plutôt qu'un miroir deviné. Cette règle conservatrice devra être comparée à une ROM commerciale autorisée et à un oracle matériel avant toute affirmation de compatibilité.

Référence de la carte mémoire : [gbadev — GBA memory map](https://gbadev.net/gbadoc/memory.html). Aucun code de cette référence n'est copié.


### A-12 — Extension des opérations arithmétiques immédiates ARM/Thumb

**État : inclus dans PR #4 et dans `main`; la CI de code a passé au run [39754684102](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39754684102), puis au run post-fusion [39756038257](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39756038257).**

La tranche codegen ajoute RSB et CMN au sous-ensemble ARM data-processing immédiat, et étend Thumb du seul MOVS immédiat à MOVS/CMP/ADDS/SUBS. Le source généré calcule les flags propres à chaque opération : RSB calcule Operand2 - Rn et les flags dans cet ordre; CMN actualise les flags comme une addition sans écrire le registre destination; CMP Thumb ne modifie pas le registre comparé; ADDS/SUBS écrivent le registre et actualisent N/Z/C/V.

Un test du code généré compile puis exécute maintenant le source sur des données synthétiques, y compris un accès au PC ARM visible (+8) et des opérations immédiates Thumb. Ce ne sont pas des tests différentiels GBA. Les décalages registre, ADC/SBC/RSC, chargements/stockages, BL Thumb, BX/interworking et le bus matériel restent exclus; l'inconnu bloque toujours la génération.


### A-13 — Décoder la condition ARM `0xF` avant les masques BX/SWI

**Sévérité : élevée pour la validité du décodeur. État : défaut confirmé dans `main`; correctif de PR #5 fermé sans fusion.**

Dans `src/recompiler/arm.rs`, `decode` vérifie actuellement les motifs BX et SWI avant le garde `condition == 0x0F`. Des encodages de l'espace ARM inconditionnel peuvent ainsi correspondre à un masque BX/SWI avant le rejet de la condition `0xF`. Le code GBA ARM7TDMI ne doit pas reconnaître par accident cet espace comme les instructions conditionnelles ordinaires.

**Action de reprise :** dans une nouvelle branche créée depuis `main`, déplacer le rejet de `condition == 0x0F` immédiatement après l'identification exacte du NOP (ou avant tous les masques conditionnels concernés), puis ajouter des tests qui prouvent que les encodages avec condition `0xF` ne sont pas décodés comme BX/SWI, tout en gardant un SWI conditionnel valide reconnu. L'ancien correctif est visible dans la branche de PR #5, mais PR #5 est fermée sans fusion : vérifier chaque changement contre `main` et ne pas supposer qu'il a été intégré.

Le calcul du PC ARM visible (`adresse instruction + 8`) est déjà présent dans le codegen fusionné. Le cas additionnel `SUB r0, pc, #8` proposé dans PR #5 n'est pas dans le test du codegen fusionné et devrait servir de test de non-régression.
