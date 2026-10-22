# Audit d'inventaire des fichiers du dépôt

- **Dernière revue :** 2026-10-22
- **Portée :** inventaire des fichiers suivis sur `phase-1/rust-bevy-static-only` pour le groupe du parseur d'en-tête GBA.
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
| `src/rom/mod.rs` | Expose `GbaHeader`, `RomHeaderError` et la taille du header. | Ajouté dans ce groupe ; CI du commit courant en attente | Ajout de sous-modules ROM ou changement de l'API publique. |
| `src/rom/header.rs` | Parse le header GBA fixe de 0xC0 octets : point d'entrée, titre, codes, octets de contrôle, version et complement checksum. | Parseur initial et trois tests synthétiques ajoutés ; résultat CI courant en attente. Ne lit/exécute pas d'instructions et ne dépend pas d'une vraie ROM. | Champs, checksum, politique de rejet ou validation ROM changent. |
| `src/recompiler/mod.rs` | Diagnostics du contrat statique ; pas de fallback runtime. | Erreurs/tests initiaux, aucun décodeur encore | Instructions, flux de contrôle, IR et rejet statique changent. |
| `src/runtime.rs` | Application Bevy et scène visuelle provisoire. | Compilation/Clippy/tests du scaffold validés ; fenêtre interactive pas encore testée localement. Ne charge pas de ROM. | Présentation ou initialisation du matériel change. |
| `README.md` | Objectifs, état réel, références, règles de recompilation et workflows. | Mis à jour pour le parseur d'en-tête ; validation CI du nouveau commit en attente. | État fonctionnel, architecture, commandes ou workflows changent. |
| `SESSION_HANDOFF.md` | Passation chronologique, commits, validations et prochaine action. | Mis à jour pour le groupe du parseur ; à actualiser avant chaque commit de clôture. | Toujours à chaque groupe de travail. |
| `audit.md` | Risques de ROM, licence, exécution statique, validations et distribution. | Mis à jour ; le parseur est identifié comme limité et en attente de CI. | Nouveau risque, résultat de CI, réutilisation de code ou livraison. |
| `docs/repository-audit.md` | Présent inventaire des chemins, rôles, états et cadences de revue. | Mis à jour pour les nouveaux fichiers ROM. | Chaque changement de fichiers/rôle ; revue complète avant release. |
| `docs/architecture.md` | Couches header, analyse ARM/Thumb, IR, émission Rust, matériel et Bevy. | Précise que le header est le premier sous-système ; les couches restantes restent à faire. | Décision architecturale ou nouveau sous-système. |
| `docs/development.md` | Environnement Linux, dépendances natives et commandes de validation. | Mis à jour sur le parseur et les limites de tests synthétiques. | Toolchain, dépendances système ou commandes changent. |
| `docs/workflows.md` | Déclencheurs, concurrence, réutilisation des artefacts et packaging. | CI scaffold antérieur passée ; Build release non encore validé après fusion de code Rust. | Workflow ou résultat observé diverge de la stratégie. |
| `docs/rea.md` | Contrat et procédure du bundle REA. | Manuel ; pas une exécution d'agent autonome. | Version REA/Node ou format de bundle change. |
| `docs/reference-projects.md` | Références GBARecomp et REA, différences et licence. | En place | Références, licences ou stratégie de portage changent. |
| `docs/decisions/ADR-0001-static-rust-recompilation.md` | Décision : générer du Rust compilé avant lancement, sans interpréteur/JIT. | Accepté | Un changement toucherait au contrat statique ; il nécessite une décision explicite. |
| `.github/workflows/ci.yml` | Vérifie le patch, rejette ROM binaires suivies, installe dépendances Linux et exécute fmt/Clippy/tests. | Run `39746990377` vert sur le scaffold précédent ; le commit du parseur doit être validé. | Dépendance native, règle d'exclusion ou test change. |
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
