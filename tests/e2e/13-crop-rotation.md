# Test e2e #13 — Rognage (crop) et rotation

Couvre la fonctionnalité #13 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Parsing/validation du crop et de la rotation, côté client :

```bash
cd client && cargo test
```

Attendu : tests de `geometry` verts (crop valide/invalide, `fits`, aller-retour,
rotations 0/90/180/270). ✅ 56 tests au total.

## Procédure manuelle (appareil réel)

### Préconditions
- Chaîne vidéo opérationnelle (Phase 2), SDK configuré.

### Étapes
1. `gninou --crop 1080:1080:0:0` → seule la zone définie est affichée.
2. `gninou --rotation 90` → l'image est pivotée d'un quart de tour.
3. `gninou --crop 9999:9999:0:0` → crop hors écran, ignoré (log serveur) + plein écran.
4. `gninou --rotation 45` → erreur côté client (valeur non autorisée).

### Résultat attendu
1. Capture limitée au rectangle demandé.
2. Rotation appliquée à la projection.
3. Crop hors limites ignoré proprement (avertissement), capture plein écran.
4. Erreur claire pour une rotation non multiple de 90.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ parsing/validation testés (unit) ; projection (SurfaceControl) à valider sur matériel
