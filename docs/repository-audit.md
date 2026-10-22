# Audit d'inventaire des fichiers du dépôt

- **Dernière revue :** 2026-10-22
- **Portée :** inventaire des fichiers suivis sur `phase-1/rust-bevy-static-only` au moment de la revue.
- **Cadence :** mettre à jour cet inventaire à chaque groupe de travail qui ajoute, supprime, renomme ou change le rôle d'un fichier ; faire une revue complète au minimum avant chaque release.
- **Complément :** [`audit.md`](../audit.md) porte sur les risques techniques, de licence et de distribution ; ce document suit les fichiers eux-mêmes.

## Inventaire

| Chemin | Rôle et contrat | État | Revue requise quand… |
| --- | --- | --- | --- |
| `.gitignore` | Exclut `target/`, sorties locales, analyses REA et ROM ; permet de garder uniquement `rom/README.md`. | En place | Un nouvel outil ou une sortie générée crée des fichiers locaux. |
| `Cargo.toml` | Manifeste du crate hôte. Bevy exactement `=0.20.0`, feature set initial 2D. | Squelette compilable attendu par CI | Dépendances/features ou structure du workspace changent. |
| `Cargo.lock` | Verrouille le graphe de dépendances, utilisé avec `--locked`. | Remplacé par une résolution Bevy 0.20 feature `2d` ; la CI doit confirmer sa cohérence exacte. | Cargo.toml ou les dépendances changent. Générer avec Cargo, jamais l'éditer manuellement. |
| `rust-toolchain.toml` | Épingle Rust `1.98.1` et les composants rustfmt/Clippy. | En place | MSRV, Bevy ou images de runners changent. |
| `src/main.rs` | Point d'entrée : démarre le runtime Bevy. | Squelette | Le lancement ou l'initialisation globale change. |
| `src/lib.rs` | Expose les modules de runtime et de recompilation. | Squelette | Les couches du crate ou les API publiques changent. |
| `src/runtime.rs` | Construction de l'application Bevy et scène visuelle provisoire. | Placeholder ; ne charge pas de ROM | Fenêtre, présentation, périphériques ou initialisation du matériel changent. |
| `src/recompiler/mod.rs` | Types de diagnostic du contrat de recompilation statique, sans fallback runtime. | Types/tests initiaux ; analyseur absent | Instructions, flux de contrôle, IR ou stratégies de rejet sont ajoutés. |
| `README.md` | Objectif, état réel, références, contrainte Rust statique et procédures. | Revu dans cette PR | Chaque groupe de travail modifie le statut, l'architecture, les commandes ou les workflows. |
| `SESSION_HANDOFF.md` | Passation chronologique, commits, validations et prochaines actions. | Revu dans chaque commit de clôture | Toujours : mettre à jour avant le commit qui clôt un groupe. |
| `audit.md` | Risques techniques, ROM, licences et distribution. | Risques ouverts explicités | Nouvelle surface de sécurité, réutilisation de code, livraison ou résultat de CI. |
| `docs/repository-audit.md` | Présent document ; inventaire et cadence d'entretien. | En place | Tout changement de fichiers ou de responsabilité. |
| `docs/architecture.md` | Couches analyse ARM/Thumb, IR, émission Rust, matériel GBA et runtime Bevy ; aucun interpréteur/JIT. | Contrat cible ; composants non implémentés | Décision d'architecture ou nouveau sous-système. |
| `docs/development.md` | Environnement Linux, dépendances natives et commandes de validation. | Guide initial | Toolchain, dépendances système ou commandes changent. |
| `docs/workflows.md` | Déclencheurs, concurrence, réutilisation des artefacts et sécurité du packaging. | Stratégie documentée ; CI en cours de validation | Un workflow change ou son vrai résultat diverge du plan. |
| `docs/rea.md` | Bundle REA, procédure et limite entre packaging et analyse interactive. | En place | Version REA, dépendances ou format de bundle changent. |
| `docs/reference-projects.md` | Références GBARecomp et REA, différences techniques et limites de licence. | En place | Les dépôts, licences ou méthodes pertinentes changent. |
| `docs/decisions/ADR-0001-static-rust-recompilation.md` | Décision contraignante : émettre du Rust et le compiler avant lancement ; pas d'interpréteur/JIT. | Accepté | Un changement proposé toucherait le contrat statique ; toute modification doit être une décision explicite. |
| `.github/workflows/ci.yml` | Vérifie le patch, bloque les ROM suivies, installe les bibliothèques Linux puis exécute fmt, Clippy et tests. | Workflow parsé ; filtre ROM et fmt passent au run 39745465091 ; Clippy a révélé le lockfile à corriger. | Un test, une dépendance native ou une règle d'exclusion change. |
| `.github/workflows/build.yml` | Build Linux après la CI réussie de `main`, fabrique distribution et entrée REA depuis une compilation. | À valider sur un vrai push `main` | Packaging, cible, profils Cargo ou format des artefacts changent. |
| `.github/workflows/release.yml` | Publication manuelle d'un artefact existant après validation du tag/commit et checksum. | Manuel ; pas de compilation | Politique de tag, permissions ou contenu de release change. |
| `.github/workflows/rea-analysis.yml` | Produit un bundle REA depuis l'artefact d'analyse, sans compilation. | Manuel ; pas une analyse d'agent automatique | Version Node/REA, contexte REA ou contrat du bundle change. |
| `rom/README.md` | Instructions pour placer une ROM légalement obtenue uniquement en local. | Document suivi ; ROM elle-même ignorée | Chemin ou politique ROM change. |

## Règle de mise à jour obligatoire

À chaque groupe de travail, avant son commit de clôture :

1. Mettre à jour ce fichier si l'inventaire, le rôle ou l'état d'un fichier change.
2. Mettre à jour `audit.md` si le risque, les licences, l'état de validation ou la politique de distribution changent.
3. Relire et mettre à jour `README.md` pour refléter l'état réellement livré, les commandes qui fonctionnent et les limitations actuelles.
4. Mettre à jour `SESSION_HANDOFF.md` en dernier avec le groupe, la date, les chemins touchés, les validations exécutées (et celles qui ont échoué), le SHA du commit et la prochaine étape.

Une vérification non exécutée doit être marquée **non vérifiée**, pas comme un succès. Avant release, faire une revue complète des fichiers inclus dans les artefacts et vérifier qu'aucune ROM/BIOS/sauvegarde/code ROM dérivé ne s'y trouve.
