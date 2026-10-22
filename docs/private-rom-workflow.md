# Accès contrôlé à la ROM privée depuis GitHub Actions

## Architecture retenue

- Code et workflows : rakmahefa/gbarecomp-rust, dépôt public.
- ROM de test : rakmahefa/roms, dépôt privé.
- Authentification : clé SSH de déploiement dédiée, lecture seule, enregistrée comme secret d'environnement ROMS_DEPLOY_KEY dans l'environnement roms-audit.
- Workflow consommateur : .github/workflows/private-rom-audit.yml, déclenché manuellement uniquement depuis main.
- Le workflow ne publie ni la ROM, ni la désassemblage détaillé, ni les sources générées. Il expose uniquement un résumé agrégé, car les journaux et résumés d'un dépôt public ne doivent pas être traités comme des espaces privés.

Le GITHUB_TOKEN de gbarecomp-rust n'accorde pas automatiquement l'accès à un autre dépôt privé. La clé de déploiement limite l'accès à un seul dépôt. GitHub indique que les deploy keys sont en lecture seule par défaut et sont attachées à un dépôt spécifique : https://docs.github.com/en/authentication/connecting-to-github-with-ssh/managing-deploy-keys.

## Configuration manuelle requise

La connexion GitHub utilisée pour cet audit ne peut pas créer/écrire les secrets Actions ou les deploy keys. Configure ces éléments dans l'interface GitHub ; ne colle jamais une clé privée dans un ticket, un commit ou une conversation.

### 1. Générer une clé dédiée sur ta machine Linux

    install -d -m 700 ~/.ssh
    ssh-keygen -t ed25519 \
      -C "gbarecomp-rust-readonly-roms" \
      -f ~/.ssh/gbarecomp-rust-roms-actions \
      -N ""

Cette paire de clés ne doit servir qu'à cet accès. La clé publique se termine par .pub ; la clé privée est le fichier sans extension.

### 2. Ajouter la clé publique au dépôt privé roms

Ouvre https://github.com/rakmahefa/roms/settings/keys, choisis **Add deploy key**, puis :

- Titre : **gbarecomp-rust Actions (read-only)**
- Key : colle le contenu de ~/.ssh/gbarecomp-rust-roms-actions.pub
- **Ne coche pas** Allow write access.

Enregistre la clé. Elle doit être ajoutée au dépôt roms, pas au dépôt public gbarecomp-rust.

### 3. Créer l'environnement et le secret dans gbarecomp-rust

Ouvre https://github.com/rakmahefa/gbarecomp-rust/settings/environments :

1. Crée l'environnement nommé exactement **roms-audit**.
2. Dans les règles de déploiement, autorise uniquement la branche **main** (selected branches/tags).
3. Ajoute un **environment secret** nommé exactement **ROMS_DEPLOY_KEY**.
4. Comme valeur du secret, colle le contenu complet de ~/.ssh/gbarecomp-rust-roms-actions (la clé privée, sans l'exposer dans un commit ou une conversation).
5. Sauvegarde.

L'environnement limité à main est important : le workflow avec accès au secret ne doit pas être lancé depuis une branche de test qui pourrait contenir un workflow modifié.

Après avoir enregistré le secret, retire la copie locale si tu n'en as plus besoin. Ne publie ni la clé privée ni son contenu dans les logs.

## 4. Tester le workflow

Ouvre https://github.com/rakmahefa/gbarecomp-rust/actions/workflows/private-rom-audit.yml, sélectionne **Private ROM audit (manual)**, choisis la branche main, conserve le budget par défaut (250000) et clique sur **Run workflow**.

Le workflow exige exactement un fichier .gba à la racine de rakmahefa/roms. Le dépôt actuel satisfait ce contrat. Il construit gbarecomp-recomp, exécute inspect puis analyze sur le runner et affiche uniquement des compteurs agrégés. Aucun artefact n'est téléversé. Le runner supprime le checkout privé et les rapports bruts après l'exécution.

### Résultats et limites

- Une erreur du checkout privé indique en général une deploy key absente/incorrecte ou enregistrée au mauvais dépôt, ou un secret non disponible dans l'environnement roms-audit.
- Un job vert signifie que le téléchargement, l'inspection et l'analyse bornée se sont terminés ; ce n'est **pas** une compilation complète du jeu.
- L'analyse actuelle ne supporte qu'un sous-ensemble ARM/Thumb et ne démontre pas qu'Emerald puisse être recompilé ou lancé. Les diagnostics sont attendus et doivent être interprétés comme des limites de couverture.
- Le workflow actuel ne lance pas REA sur des données dérivées de la ROM. Le bundle REA existant inspecte le binaire natif hôte ; les audits détaillés de code invité doivent rester dans un environnement privé tant que leurs sorties ne sont pas protégées.
- Les workflows de Build (Linux), Release et REA existants restent indépendants de la ROM. Aucun paquet de distribution ne dépend de ce secret.

## Incident / rotation de la clé

Si la clé privée est exposée, supprime le secret ROMS_DEPLOY_KEY dans gbarecomp-rust, puis retire le deploy key correspondant dans roms et génère une nouvelle paire. Ne réutilise pas cette clé pour un autre dépôt.
