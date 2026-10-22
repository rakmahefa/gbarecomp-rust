# Session handoff

Dernière mise à jour : 2026-10-22
Groupe de travail : Phase 1 — audit de référence et première tranche de recompilation statique AOT
Branche de livraison : `reference-audit-static-port`
Branche de base : `phase-1/rust-bevy-static-only`
PR : [#4 — audit de référence et portage Rust statique](https://github.com/rakmahefa/gbarecomp-rust/pull/4)

## Invariant non négociable

Le code invité GBA ARM7TDMI est analysé à l'avance et les blocs pris en charge sont émis sous forme de code source Rust natif, compilé avant exécution avec Cargo/rustc. Bevy reste la couche hôte/rendu. Aucun interpréteur d'instructions invité, aucun JIT et aucune traduction du code invité à l'exécution. Instruction inconnue, destination indirecte non résolue, code dynamique ou ambiguïté code/données doivent bloquer la sortie générée plutôt que devenir un NOP ou un chemin de secours silencieux.

## Audit de référence

- Référence étudiée : [mstan/gbarecomp](https://github.com/mstan/gbarecomp), commit observé `9266573123fb950dfdee2d19040a2daf5e63e39d`, 491 fichiers suivis dans l'arbre inspecté.
- `reference-audit.md` documente les familles de modules, la stratégie de portage, les exclusions de provenance et l'ordre d'implémentation.
- Le dépôt de référence utilise PolyForm Noncommercial 1.0.0. Le bootstrap Rust courant est écrit indépendamment et ne copie ni extraits C++, ni ROM, ni BIOS, ni tables/données générées depuis ceux-ci. Tout futur port dérivé direct exige une vérification de licence et potentiellement une autorisation distincte.
- La licence du dépôt Rust doit encore être choisie avant une distribution publique. Le blob ROM antérieurement suivi peut subsister dans l'historique Git : ce groupe n'a pas réécrit l'historique.

## Implémentation présente dans la PR

- Parsing de l'en-tête ROM et abstraction `GbaRom` avec mapping borné des fenêtres Game Pak.
- Décodeur ARM conservateur : B/BL, NOP exact, opérations data-processing immédiates prises en charge (AND, EOR, SUB, RSB, ADD, TST, TEQ, CMP, CMN, ORR, MOV, BIC, MVN); BX/SWI détectés mais les chemins non gérés sont rejetés.
- Décodeur Thumb conservateur : B, branches conditionnelles, MOVS/CMP/ADDS/SUBS immédiats, BX/SWI et préfixe/suffixe BL détectés; les chemins non pris en charge restent en diagnostic.
- IR typée et découverte depuis le branchement d'entrée; branches directes/conditionnelles, budget de découverte, et diagnostics pour cible inconnue, branchement indirect, instruction inconnue et chevauchement de blocs.
- Émetteur AOT de fonctions Rust et dispatch statique. Le code généré respecte le PC ARM visible (+8) pour les opérations immédiates qui lisent R15; les tests compilent et exécutent le source généré.
- CLI offline distincte du host Bevy : `gbarecomp-recomp inspect`, `analyze`, `recompile --emit`.
- Documentation de référence, d'architecture, de développement, de risques et d'inventaire mise à jour.

## Validation CI connue

- Le run [39753518405](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39753518405) au commit `7505b138a8500fac9c0289376e5887ba5f396881` a passé hygiène/filtre ROM, `cargo fmt --all -- --check`, Clippy strict et `cargo test --workspace --all-features --locked`, y compris le test qui compile et exécute le Rust généré.
- Le run [39754684102](https://github.com/rakmahefa/gbarecomp-rust/actions/runs/39754684102) au commit `77e58405c09560a4e40a41cfc697797adb03e187` a passé le contrôle ROM, `cargo fmt --all -- --check`, Clippy strict et `cargo test --workspace --all-features --locked`, incluant le test d'exécution du source généré avec les opérations Thumb immédiates et le PC ARM (+8). Le commit documentaire suivant n'a pas changé le code Rust.
- Aucun test différentiel mGBA/oracle n'a été exécuté. Aucun démarrage de jeu complet ni intégration du code émis à un runtime de jeu Bevy n'est démontré.
- L'environnement local de préparation n'a pas de toolchain Rust; la compilation est vérifiée par GitHub Actions, pas annoncée comme exécutée localement.

## Limites ouvertes / prochaine tranche

1. Ajouter opérations registre/décalages, ADC/SBC/RSC, BL Thumb, BX/BLX/interworking et résolution des cibles indirectes.
2. Implémenter splitting de blocs et classification code/données; résoudre les tables de saut de manière statique.
3. Ajouter mémoire RAM et sémantique load/store à l'IR/codegen.
4. Définir les effets MMIO, puis IRQ/DMA/timers/ordonnanceur, PPU, audio, sauvegardes, RTC et périphériques.
5. Lier la sortie compilée à un runtime natif/hôte, ajouter validation différentielle déterministe et rapports de couverture.
6. Faire une revue finale des licences des sous-modules et choisir une licence explicite pour le dépôt cible.
