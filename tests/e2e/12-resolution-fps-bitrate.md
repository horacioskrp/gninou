# Test e2e #12 — Résolution / FPS / débit configurables

Couvre la fonctionnalité #12 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Parsing du débit (`--bit-rate`, suffixes K/M), côté client :

```bash
cd client && cargo test
```

Attendu : tests de `cli::parse_bitrate` verts (entier nu, K/M, invalides, hors limites).
✅ 50 tests au total.

## Procédure manuelle (appareil réel)

### Préconditions
- Chaîne vidéo opérationnelle (Phase 2), SDK configuré.

### Étapes
1. `gninou -m 1024 --max-fps 30 --bit-rate 4M` → vérifier définition/FPS/débit appliqués.
2. `gninou --bit-rate 20M` → image de meilleure qualité, débit plus élevé.
3. Sans `--bit-rate` → débit automatique (selon la définition).
4. `gninou --bit-rate 1.5M` → erreur côté client (format invalide) avant déploiement.

### Résultat attendu
1. Résolution ramenée à ≤1024px (bord long), ~30 fps, ~4 Mbps.
2. Débit effectif cohérent avec la valeur demandée.
3. Débit auto borné (2–40 Mbps) selon la définition.
4. Erreur claire pour un débit mal formé.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ parsing testé (unit) ; application réelle (MediaFormat) à valider sur matériel
