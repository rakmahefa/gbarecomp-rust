# Session handoff

Dernière mise à jour : 2026-10-24
État : PR #4 fusionnée, PR #5 fermée sans fusion, PR #6 fusionnée ; refonte des artefacts ROM en proposition via roms#1 et gbarecomp-rust#7
Branche de reprise technique : `main`
HEAD public ayant servi de base à la proposition : `100d45784ad93a49de31d86c85cbdad260c73545`
CI (Linux) de cette base : [run 40058586979](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/40058586979) — succès
Build (Linux) de cette base : [run 40058729974](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/40058729974) — succès
Ancien audit direct depuis le dépôt public : [run 39762725872](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39762725872) — échec de chargement de clé SSH, sans analyse ROM

## Nouvelle architecture proposée — ROM privée → artefact assaini (2026-10-24)

Deux PRs ouvertes proposent de déplacer l'exécution dans le dépôt privé puis de faire importer au dépôt public un seul artefact assaini : [rakmahefa/roms#1](https://github.com/rakmahefa/roms/pull/1) et [gbarecomp-rust#7](https://github.com/rakmahefa/gbarecomp-rust/pull/7). Aucune nouvelle chaîne d'analyse n'a encore été exécutée de bout en bout.

- Dans `roms`, `.github/workflows/rom-analysis.yml` checkout la ROM privée et le code public `gbarecomp-rust@main`, construit `gbarecomp-recomp`, exécute `inspect`/`analyze`, puis REA/Ghidra sur la ROM elle-même comme image ARM brute à `0x08000000` et séparément sur le binaire natif public. `rom-analysis-private` contient les rapports détaillés de la ROM et le JSON REA de la ROM ; `rom-analysis-public` n'exporte que les compteurs agrégés, le statut/configuration de REA-ROM et le JSON REA du binaire natif public.
- Dans `gbarecomp-rust`, `.github/workflows/private-rom-audit.yml` est retiré. Le nouveau `.github/workflows/import-private-rom-analysis.yml` accepte un Run ID, vérifie le run source, l'allowlist, le schéma et les checksums, puis importe seulement `rom-analysis-public`.
- Le jeton requis dans le dépôt public est un fine-grained token / GitHub App token limité au seul dépôt privé `roms` avec **Actions: read**. Il est enregistré dans l'environnement public `roms-artifacts-read`, restreint à `main`, sous le secret `ROMS_ARTIFACT_READ_TOKEN`. Le workflow privé n'a pas besoin de clé SSH ou de secret pour cloner le dépôt public.
- L'artefact public exclut les octets ROM, le nom et le SHA-256 ROM, le JSON REA-ROM détaillé, le désassemblage/pseudocode et le code généré depuis la ROM. REA/Ghidra utilise un langage ARM générique avec base `0x08000000`, pas un modèle exact ARM7TDMI/GBA ; pas d'émulation de MMIO/banking et aucune preuve de compilation complète du jeu.
- Après fusion des PR : configurer le token, lancer `Private ROM analysis (manual)` sur `main` dans `roms`, attendre le succès, copier le Run ID et lancer `Import private ROM analysis result (manual)` depuis `main` dans `gbarecomp-rust`. Seulement après validation, supprimer l'ancien secret `roms-audit / ROMS_DEPLOY_KEY` et la deploy key correspondante si elle ne sert plus.

### CI / validation de la proposition

- La CI de la base publique, [run 40058586979](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/40058586979), et son build, [run 40058729974](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/40058729974), sont réussis avant cette proposition.
- La CI publique la plus récente vérifiée sur la branche de PR #7, [run 40060200483](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/40060200483), est verte pour le HEAD `d22503a`; ses vérifications Rust (format, Clippy, tests) et hygiène passent. Des commits documentaires/workflow plus récents peuvent avoir déclenché un run supplémentaire ; à re-vérifier avant fusion.
- Le workflow privé PR #1 n'a pas encore été fusionné ni exécuté. La passe REA-ROM, Ghidra/REA et le schéma d'artefact restent donc à vérifier au premier run réel.
- L'ancien run [39762725872](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39762725872) n'a jamais commencé l'inspection : `Load key ... error in libcrypto` puis `Permission denied (publickey)`.

### Prochaine séquence recommandée

1. Revoir et fusionner `roms#1`; revoir et fusionner `gbarecomp-rust#7` ensuite.
2. Configurer `ROMS_ARTIFACT_READ_TOKEN` dans l'environnement `roms-artifacts-read` du dépôt public, avec permission Actions:read uniquement sur `roms`.
3. Exécuter l'analyse privée, puis l'import public avec son Run ID. Inspecter l'artefact public et confirmer l'absence de fichiers, hash ou détails ROM privés.
4. Reprendre ensuite le défaut ARM `condition == 0x0F` décrit plus bas dans une branche neuve depuis le `main` courant; PR #5 reste fermée sans fusion.

## Consignes immédiates pour la prochaine session

1. **Commencer sur `main`** et lire ce fichier avant toute modification. Ne pas reprendre directement `phase-1/rust-bevy-static-only` ni la branche de livraison `reference-audit-static-port` comme base de travail; elles ont servi aux PR précédentes et sont désormais historiques.
2. La PR [#4](https://github.com/rakmahefa/gbarecomp-rust/pull/4) est fusionnée. Elle a intégré `reference-audit.md`, l'audit et la première tranche de recompilation statique ARM/Thumb décrite ci-dessous.
3. La PR [#5](https://github.com/rakmahefa/gbarecomp-rust/pull/5) a été fermée **sans fusion**. Ses changements ne doivent pas être supposés présents. Si l'on reprend son correctif, créer une nouvelle branche depuis le `main` courant et ouvrir une nouvelle PR plutôt que rouvrir/recycler PR #5.
4. Priorité technique : corriger le décodeur ARM au sujet de la condition `0xF` et ajouter les tests de non-régression décrits dans « Prochaine action ».

## Invariant non négociable

Le code invité GBA ARM7TDMI est analysé à l'avance et les blocs pris en charge sont émis sous forme de source Rust natif, compilé avant exécution avec Cargo/rustc. Bevy reste la couche hôte/rendu. Aucun interpréteur d'instructions invité, aucun JIT et aucune traduction de code invité à l'exécution. Instruction inconnue, cible indirecte non résolue, code dynamique ou ambiguïté code/données doivent bloquer la génération plutôt que devenir un NOP ou un chemin de secours silencieux.

## État livré par PR #4

- Audit complet des principales zones de la référence `mstan/gbarecomp` et contrainte de licence PolyForm Noncommercial 1.0.0 dans `reference-audit.md`.
- Parsing de l'en-tête GBA et abstraction `GbaRom` avec mapping borné des fenêtres ROM Game Pak.
- Décodeur ARM conservateur : B/BL, NOP exact, sous-ensemble des opérations data-processing immédiates, dont RSB/CMN. BX/SWI sont repérés mais pas gérés par le codegen actuel.
- Décodeur Thumb conservateur : B, branche conditionnelle, MOVS/CMP/ADDS/SUBS immédiats, BX/SWI et marqueurs de préfixe/suffixe BL; les chemins non pris en charge restent en diagnostic.
- IR typée et découverte de blocs depuis le branchement d'entrée, avec budget et diagnostics pour instruction non supportée, cible statique non résolue, flux indirect et chevauchement de blocs.
- Émetteur AOT de source Rust pour le sous-ensemble validé; le codegen utilise `instruction_address + 8` pour le PC visible ARM lorsqu'une opération immédiate lit R15.
- CLI hors ligne distincte de Bevy : `gbarecomp-recomp inspect`, `analyze` et `recompile --emit`.
- Pas de bus/MMIO avec effets, pas de load/store dans l'IR/codegen, pas d'IRQ/DMA, de PPU/audio, de résolution générale des pointeurs indirects, ni d'intégration de l'exécutable généré à un jeu Bevy.

## État des PR et de la CI

- PR #4 : [fusionnée](https://github.com/rakmahefa/gbarecomp-rust/pull/4) dans `main` au merge commit `fe9e1df5b74ccb686723adeea495419249e08d2f`.
- PR #5 : [fermée sans fusion](https://github.com/rakmahefa/gbarecomp-rust/pull/5). Ses changements ne sont pas présents dans `main` et ne doivent pas être supposés intégrés.
- PR #6 : [fusionnée](https://github.com/rakmahefa/gbarecomp-rust/pull/6) ; elle ajoute le workflow manuel d'audit de ROM privée et sa documentation.
- Sur le HEAD `fcb951f327d81f9f96d3acdf335c767fe29c2c31`, [CI (Linux), run 39762607498](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39762607498) et [Build (Linux), run 39762711875](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39762711875) ont réussi.
- [Private ROM audit (manual), run 39762725872](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39762725872) a échoué lors du chargement de la clé SSH destinée au checkout du dépôt privé. L'audit du binaire/ROM n'a donc pas commencé ; voir la section « État courant — accès à la ROM privée » et `docs/private-rom-workflow.md`.
- La validation du code est réalisée par GitHub Actions ; aucun succès de compilation locale n'est revendiqué.
- Aucun test différentiel mGBA/oracle, démarrage de jeu complet ou test matériel de compatibilité n'a été exécuté.

## Défaut connu et prochaine action

Dans `src/recompiler/arm.rs`, `decode` teste actuellement les motifs BX puis SWI **avant** de vérifier `condition == 0x0F`. Des encodages de l'espace inconditionnel peuvent donc être interprétés selon ces masques avant d'être rejetés. Le correctif proposé dans PR #5 déplaçait le rejet plus tôt, mais la PR est fermée et ce changement n'est pas dans `main`.

Dans une nouvelle branche issue de `main` :
1. déplacer le rejet de `condition == 0x0F` avant les masques conditionnels BX/SWI (tout en gardant l'encodage exact NOP géré);
2. tester qu'un BX conditionnel valide est décodé, qu'un SWI valide ARM7TDMI est reconnu, et que les encodages condition `0xF` concernés retournent `Unsupported` plutôt que BX/SWI;
3. porter le test généré additionnel `SUB r0, pc, #8` pour vérifier le PC ARM visible +8, déjà implémenté mais pas couvert par ce cas;
4. lancer `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` et `cargo test --workspace --all-features --locked` dans CI, puis actualiser les audits.

Après ce correctif, aborder la scission de blocs/classification code-données, puis load/store avec une interface mémoire explicite. Ne pas inventer un comportement pour une instruction non prise en charge.

## Licence et provenance

La référence C++ utilise PolyForm Noncommercial 1.0.0. Le code livré par PR #4 a été écrit indépendamment et ne copie ni extraits C++, ni ROM, ni BIOS, ni données générées depuis ceux-ci. Tout port source direct/dérivé futur exige une vérification de licence et éventuellement une autorisation distincte. Le dépôt cible doit toujours choisir une licence explicite avant une distribution publique étendue. Le blob ROM suivi historiquement peut demeurer dans l'historique Git antérieur.
