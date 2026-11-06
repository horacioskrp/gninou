# Workflow Git & politique de test

gninou suit le modèle **Git Flow**, avec un test obligatoire par fonctionnalité.

## Branches

| Branche | Rôle | Dure |
|---------|------|------|
| `main` | code stable, versions publiées (tags) | permanente |
| `develop` | intégration continue des fonctionnalités | permanente |
| `feature/<nom>` | une fonctionnalité (voir FEATURES.md) | éphémère |
| `release/<version>` | préparation d'une version | éphémère |
| `hotfix/<nom>` | correctif urgent sur `main` | éphémère |

Rien n'est commité directement sur `main` ni sur `develop` : tout passe par une branche.

## Cycle d'une fonctionnalité

```bash
# 1. Partir de develop, à jour
git switch develop && git pull

# 2. Créer la branche de la fonctionnalité
git switch -c feature/01-detection-adb     # cf. numéro dans FEATURES.md

# 3. Développer + écrire le(s) test(s)

# 4. TESTER — la fonctionnalité doit être verte (voir ci-dessous)

# 5. Fusionner dans develop (pas de fast-forward, pour garder l'historique)
git switch develop
git merge --no-ff feature/01-detection-adb

# 6. Mettre à jour FEATURES.md (état ✅ + colonne Testé) et ROADMAP.md
# 7. Supprimer la branche
git branch -d feature/01-detection-adb
```

## Versions (release)

```bash
git switch -c release/0.1.0 develop
# ... ajustements, bump de version, changelog ...
git switch main && git merge --no-ff release/0.1.0
git tag -a v0.1.0 -m "gninou 0.1.0"
git switch develop && git merge --no-ff release/0.1.0
```

## Politique de test — « tester chaque fonctionnalité au fur et à mesure »

Aucune `feature/*` n'est fusionnée dans `develop` tant que son test n'est pas vert.
Le type de test dépend du composant :

| Composant | Tests | Commande |
|-----------|-------|----------|
| client (Rust) | tests unitaires + intégration | `cargo test` |
| serveur (Kotlin) | tests unitaires | `./gradlew test` |
| bout en bout | script manuel ou automatisé sur appareil réel/émulateur | voir `tests/` |

Règles :
1. Chaque fonctionnalité ajoute **au moins un test** qui la couvre.
2. Les fonctionnalités nécessitant un appareil (capture, injection…) sont validées
   par un **test de bout en bout** documenté (étapes + résultat attendu) dans `tests/`.
3. Chaque **phase** de la roadmap se clôt par un **test de phase** (ligne « Test de phase »
   dans [ROADMAP.md](ROADMAP.md)) avant de passer à la suivante.
4. On ne coche « Testé » dans [FEATURES.md](FEATURES.md) qu'après un test vert.

## Convention de messages de commit

Format `type(scope): description` :

- `feat(client): détection des appareils via adb devices`
- `fix(server): keyframe manquante au démarrage`
- `test(client): couverture du parsing des arguments`
- `docs: mise à jour de la roadmap`
- `chore`, `refactor`, `build`, `ci` selon le cas.
