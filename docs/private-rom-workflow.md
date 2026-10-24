# Flux d'analyse : dépôt ROM privé → artefact assaini → dépôt public

## Architecture

- `rakmahefa/roms` est le seul dépôt qui checkout et lit la ROM. Le workflow `.github/workflows/rom-analysis.yml` s'exécute manuellement depuis `main`.
- Ce workflow checkout `rakmahefa/gbarecomp-rust@main` (dépôt public), construit `gbarecomp-recomp`, inspecte la ROM et effectue une analyse ARM/Thumb bornée.
- Le même workflow exécute REA/Ghidra sur deux cibles : la ROM elle-même comme image ARM brute mappée à `0x08000000` (le rapport complet reste privé), puis le binaire Linux natif `gbarecomp-recomp` construit depuis le **code public**. L'analyse ARM brute est approximative : elle n'émule pas le processeur ARM7TDMI exact ni le matériel GBA, les accès MMIO, les wait states ou le banking. Elle ne prouve pas que le jeu soit entièrement recompilé ou exécutable.
- Le dépôt public ne checkout plus le dépôt ROM et n'a plus besoin d'une deploy key SSH.

## Séparation des artefacts

Le workflow privé publie deux artefacts distincts, conservés 14 jours :

- `rom-analysis-private` dans `rakmahefa/roms` : inspection ROM brute, diagnostics détaillés, résultat REA complet de la ROM et résultat REA du binaire natif public. Il reste dans le dépôt privé. Ne pas le recopier dans un dépôt public.
- `rom-analysis-public` dans `rakmahefa/roms` : `summary.json`, `summary.md`, checksums, statut/configuration simplifiée de la passe REA sur ROM et résultat REA du binaire natif issu du code public. Aucun octet ROM, nom de ROM, empreinte SHA-256 ROM, résultat REA détaillé/désassemblage/pseudocode ROM ou source générée depuis la ROM n'est inclus.

Le workflow public `.github/workflows/import-private-rom-analysis.yml` ne télécharge que `rom-analysis-public`. Il vérifie que le run source a réussi sur `main`, valide le schéma, les checksums et une liste autorisée de quatre fichiers, puis publie ce résultat assaini en artefact du run public. Les artefacts/actions d'un dépôt public sont eux-mêmes accessibles publiquement : il est donc intentionnel que **seul** l'artefact assaini y soit importé.

## Configuration requise

### 1. Fusionner les deux propositions de workflow

Fusionner d'abord la PR du dépôt privé `rakmahefa/roms`, qui ajoute `.github/workflows/rom-analysis.yml` et `docs/rom-analysis-workflow.md`. Fusionner ensuite la PR de `rakmahefa/gbarecomp-rust`, qui remplace le workflow d'accès direct à la ROM par l'import d'artefact. Les workflows ne sont exécutables manuellement depuis `main` qu'après leur fusion respective.

### 2. Créer un jeton de lecture d'artefacts limité au dépôt privé

Dans les paramètres développeur GitHub, crée un **fine-grained personal access token** limité au seul dépôt `rakmahefa/roms`, avec uniquement la permission de dépôt **Actions: Read-only** (la permission Metadata requise par GitHub est en lecture seule). Il n'a pas besoin de permission Contents ni d'écriture. Un GitHub App token équivalent limité à Actions: read convient également.

Ne place pas ce jeton dans le dépôt privé, un commit ou des logs. Il sera utilisé seulement pour lire le run et l'artefact du dépôt privé.

### 3. Enregistrer le jeton dans l'environnement du dépôt public

Dans [les environnements de gbarecomp-rust](https://github.com/rakmahefa/gbarecomp-rust/settings/environments) :

1. Crée l'environnement nommé exactement `roms-artifacts-read`.
2. Restreins ses déploiements à la branche `main`.
3. Ajoute un **environment secret** nommé exactement `ROMS_ARTIFACT_READ_TOKEN`, contenant le jeton précédent.

Le workflow d'import refuse de s'exécuter ailleurs que sur `main`. Ne déclenche pas ce workflow depuis une branche non fiable avec le jeton disponible.

### 4. Exécuter puis importer

1. Dans [Actions de roms](https://github.com/rakmahefa/roms/actions/workflows/rom-analysis.yml), lance **Private ROM analysis (manual)** depuis `main`. Garde le budget borné proposé, sauf besoin explicite.
2. Attends que le run privé termine avec succès et copie son **Run ID numérique**.
3. Dans [Actions de gbarecomp-rust](https://github.com/rakmahefa/gbarecomp-rust/actions/workflows/import-private-rom-analysis.yml), lance **Import private ROM analysis result (manual)** depuis `main`, puis fournis ce Run ID.
4. Télécharge le nouvel artefact public `rom-analysis-imported-<run-id>` dans le run d'import. Il ne doit contenir que le résumé agrégé et le résultat REA du binaire public.

Si le run privé échoue, inspecte ses logs et l'artefact privé `rom-analysis-private`. Le workflow public refusera les runs privés échoués ou les fichiers supplémentaires/non autorisés.

## Nettoyage de l'ancien mécanisme

L'ancien workflow `.github/workflows/private-rom-audit.yml` est remplacé par l'import d'artefact. Après validation de la nouvelle chaîne, supprimer le secret d'environnement `roms-audit / ROMS_DEPLOY_KEY` du dépôt public et retirer la deploy key SSH associée de `roms` si elle n'est plus utilisée. Aucune clé SSH d'accès à la ROM n'est requise par la nouvelle chaîne.

## Limites

- L'analyse ARM/Thumb est bornée à un sous-ensemble limité de l'outil actuel, même en cas de réussite.
- REA/Ghidra analyse le binaire natif Linux public ; ce résultat n'est pas un rapport REA de la ROM.
- Le résumé public n'est pas une preuve de compatibilité totale d'Emerald et ne doit pas être interprété comme un build jouable.
- La ROM, son empreinte, ses détails de désassemblage et tout code généré depuis elle demeurent dans le dépôt/artefact privé.