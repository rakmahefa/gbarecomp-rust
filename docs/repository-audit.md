# Audit d'inventaire des fichiers du dépôt

- **Dernière revue :** 2026-10-22
- **Portée :** inventaire des fichiers suivis sur `phase-1/rust-bevy-static-only` pour la tranche CFG/IR/émission Rust statique.
- **Cadence :** actualiser l'inventaire à chaque groupe de travail qui ajoute, supprime, renomme ou change le rôle d'un fichier ; faire une revue complète au minimum avant chaque release.
- **Complément :** [`audit.md`](../audit.md) suit les risques techniques, licences et distribution ; le présent document suit les fichiers eux-mêmes.

## Inventaire

| Chemin | Rôle et contrat | État | Revue requise quand… |
| --- | --- | --- | --- |
| `.gitignore` | Exclut `target/`, sorties locales, analyses REA et ROM ; garde seulement `rom/README.md`. | En place | Un outil ou une sortie générée crée de nouveaux fichiers locaux. |
| `Cargo.toml` | Manifeste du crate ; Bevy exactement `=0.20.0`, feature set 2D initial. | Scaffold compilé par CI | Dépendances, features ou structure du workspace changent. |
| `Cargo.lock` | Verrouille les dépendances ; utilisé avec `--locked`. | Bevy 0.20 / feature 2D validé sous `--locked` par CI | Dépendances changent ; générer avec Cargo, ne jamais l'éditer manuellement. |
| `rust-toolchain.toml` | Épingle Rust `1.98.1` et les composants rustfmt/Clippy. | En place | MSRV, Bevy ou image de runner change. |
| `src/main.rs` | Point d'entrée qui démarre le runtime Bevy. | Squelette ; pas de jeu recompilé | Lancement ou initialisation globale change. |
| `src/lib.rs` | Expose les modules `recompiler`, `rom` et `runtime`. | Mis à jour avec le parseur d'en-tête | La structure des couches/API change. |
| `src/rom/mod.rs` | Expose `GbaHeader`, `GbaRom`, les erreurs de parsing/mapping, `RomWindow` et les constantes de taille. | Mis à jour pour l'abstraction d'image ROM ; CI de cette nouvelle tranche en attente. | Ajout de sous-modules ROM ou changement de l'API publique. |
| `src/rom/header.rs` | Parse le header GBA fixe de 0xC0 octets : opcode de branchement initial brut, titre, codes, octets de contrôle, version et complement checksum. | Trois tests synthétiques et le champ `entry_branch_opcode` validés par CI 39749349856 au commit e811b898. Ne décode/exécute aucune instruction. | Champs, checksum, politique de rejet ou validation ROM changent. |
| `src/rom/image.rs` | Possède les octets de la ROM, impose le maximum de 32 MiB et mappe les fenêtres Game Pak wait-state 0/1/2 vers les offsets de fichier ; lecture d'octet seulement, sans décodage invité. | Code ajouté avec quatre tests synthétiques ; CI du commit courant en attente. Les images puissance de deux utilisent un masque de miroir ; les offsets hors fichier d'une image non puissance de deux sont des erreurs. | Cartographie, mirroring, tailles acceptées ou types de cartouche changent. |
| `src/recompiler/mod.rs` | Expose les modules de découverte, décodage, IR et émission ; contient `InstructionSet` et `RecompileError`. | Contrat statique étendu pour une tranche limitée ; CI de la nouvelle implémentation à confirmer. | Ajout de diagnostics, état CPU ou sous-système. |
| `src/recompiler/arm.rs` | Décodeur conservateur ARM : B/BL, NOP exact, certaines opérations data-processing à immédiat, BX/SWI identifiés ; les autres encodages restent inconnus. | Tests synthétiques pour branche, carry après rotate et inconnus ; CI à confirmer. | Toute famille/opcode ou sémantique ajoutée. |
| `src/recompiler/thumb.rs` | Décodeur conservateur Thumb : B, branche conditionnelle, MOVS immédiat, BX/SWI et marqueurs BL. | Tests synthétiques ; les paires BL et encodages non supportés sont rejetés par le passeur. | Instructions, états ou calculs de cibles ajoutés. |
| `src/recompiler/ir.rs` | IR typée pour opérations ARM immédiates, MOVS Thumb, instructions, terminators et rapport de diagnostics. | En place ; invariant `is_complete_for_codegen` rejette les chemins incomplets. | Nouveau contrat d’instruction, contrôle de flux ou état matériel. |
| `src/recompiler/analysis.rs` | Parcours déterministe du graphe accessible depuis le branchement d’entrée ; gère branches directes, budget, cibles indirectes et détections d’overlap simples. | Découverte limitée, échecs explicites ; n'identifie pas encore toutes fonctions ni les régions code/données. CI à confirmer. | Algorithme de worklist, splitting de blocs, résolution de cibles ou hypothèses code/données. |
| `src/recompiler/codegen.rs` | Générateur AOT de fonctions Rust et table de dispatch pour IR validée ; contient un test qui compile puis exécute la sortie via `rustc --test`. | Source émis uniquement sans diagnostics ; le test utilise une fixture synthétique, sans matériel GBA. CI à confirmer. | ABI, opérations supportées, état CPU, intégration runtime ou génération de source. |
| `src/bin/gbarecomp-recomp.rs` | CLI offline `inspect`, `analyze`, `recompile --emit`, distincte du binaire Bevy. | En place ; charge une ROM locale ; l'option `--emit` n'écrase pas le fichier existant. CI à confirmer. | Syntaxe CLI ou flux analyse/emission. |
| `src/runtime.rs` | Application Bevy et scène visuelle provisoire. | Compilation/Clippy/tests du scaffold validés ; fenêtre interactive pas encore testée localement. Ne charge pas de ROM. | Présentation ou initialisation du matériel change. |
| `README.md` | Objectifs, état réel, références, commandes, exclusions et workflows. | Mis à jour pour la première tranche statique ; CI du commit final à confirmer. | État fonctionnel, architecture, commandes ou workflows changent. |
| `SESSION_HANDOFF.md` | Passation chronologique, commits, validations et prochaine action. | À mettre à jour avant la clôture de cette tranche ; note l'état réel de CI. | Toujours à chaque groupe de travail. |
| `audit.md` | Risques de ROM, licence, exécution statique, validations, couverture de l'analyse et distribution. | Mis à jour avec le périmètre du décodeur et de l'émetteur ; CI du commit final à confirmer. | Nouveau risque, résultat de CI, réutilisation de code ou livraison. |
| `docs/repository-audit.md` | Présent inventaire des chemins, rôles, états et cadences de revue. | Mis à jour pour ROM, décodeurs, IR, découverte, émetteur et CLI ; CI finale en attente. | Chaque changement de fichiers/rôle ; revue complète avant release. |
| `reference-audit.md` | Audit fonctionnel de la référence C++ (491 fichiers), contraintes PolyForm Noncommercial, éléments adaptables/exclus et roadmap Rust. | Ajouté dans le groupe courant ; aucune conclusion de licence au-delà du texte inspecté. | Changement de référence, décision de portage ou extension des imports source. |
| `docs/architecture.md` | Couches header, analyse ARM/Thumb, IR, émission Rust, matériel et Bevy. | Distingue explicitement le sous-ensemble implémenté des éléments hardware/runtime manquants. | Décision architecturale ou nouveau sous-système. |
| `docs/development.md` | Environnement Linux, dépendances natives, CLI d'analyse et commandes de validation. | Exemples `inspect/analyze/recompile` ajoutés ; résultat CI du groupe courant à noter. | Toolchain, dépendances système ou commandes changent. |
| `docs/workflows.md` | Déclencheurs, concurrence, réutilisation des artefacts et packaging. | CI scaffold antérieur passée ; Build release non encore validé après fusion de code Rust. | Workflow ou résultat observé diverge de la stratégie. |
| `docs/rea.md` | Contrat et procédure du bundle REA. | Manuel ; pas une exécution d'agent autonome. | Version REA/Node ou format de bundle change. |
| `docs/reference-projects.md` | Références GBARecomp et REA, différences et licence. | Lie désormais à `reference-audit.md` pour la cartographie et la stratégie de portage. | Références, licences ou stratégie de portage changent. |
| `docs/decisions/ADR-0001-static-rust-recompilation.md` | Décision : générer du Rust compilé avant lancement, sans interpréteur/JIT. | Accepté | Un changement toucherait au contrat statique ; il nécessite une décision explicite. |
| `.github/workflows/ci.yml` | Vérifie le patch, rejette ROM binaires suivies, installe dépendances Linux et exécute fmt/Clippy/tests. | Le run `39749349856` a validé le parseur et son champ renommé ; les nouveaux tests de mapping attendent la CI du commit courant. | Dépendance native, règle d'exclusion ou test change. |
| `.github/workflows/build.yml` | Build release Linux après succès CI sur push `main` ; distribution + entrée REA d'une même compilation. | PRs ignorées intentionnellement ; validation release/build attend le push de main après fusion. | Packaging, cible, profil Cargo ou artefacts change. |
| `.github/workflows/release.yml` | Publie manuellement un artefact existant après validation tag/commit/checksum. | Manuel ; pas de compilation. | Politique de tag, droits ou format de release change. |
| `.github/workflows/rea-analysis.yml` | Prépare un bundle REA depuis l'artefact existant sans recompiler. | Manuel ; n'affirme pas effectuer une investigation agent automatique. | Node/REA ou format du bundle change. |
| `rom/README.md` | Explique comment placer une ROM légalement obtenue uniquement en local. | Seul fichier suivi dans `rom/` ; rom binaire ignorée. | Chemin ou politique de ROM change. |

## Règle de mise à jour obligatoire

À chaque groupe de travail, avant le commit de clôture :

1. Actualiser le présent inventaire si les chemins, responsabilités ou états changent.
2. Relire et mettre à jour `audit.md` si les risques, licences, résultats de validation ou règles de distribution changent.
3. Relire et mettre à jour `README.md` pour qu'il décrive l'état réellement livré, les tests et limitations du moment.
4. Mettre à jour `SESSION_HANDOFF.md` avec groupe/date, fichiers revus, changements, résultats et échecs réels, SHA du commit et prochaine étape.

Une vérification non exécutée doit être marquée **non vérifiée**, pas comme un succès. Avant une release, examiner la liste exacte de fichiers empaquetés et vérifier qu'aucune ROM/BIOS/sauvegarde/code ROM dérivé n'est présent.
