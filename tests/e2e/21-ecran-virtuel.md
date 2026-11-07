# Test e2e #21 — Écran virtuel / nouvel affichage (Android 10+)

Couvre la fonctionnalité #21 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Parsing de `--new-display` (client) et des options `display_id` / `new_display`
(serveur) :

```bash
cd client && cargo test
```

Attendu : tests `geometry::new_display_*` verts (vide/défauts, `LxH`, `LxH/dpi`,
casse du `x`, rejets). ✅ 90 tests client.

> Tests serveur (`OptionsTest`) : `display_id`, `new_display` avec/sans taille.
> Exécution Gradle à faire une fois le wrapper généré (Phase 0).

## Procédure manuelle (appareil réel)

> La création réelle du display virtuel (DisplayManager / SurfaceControl) est une
> API dépendante d'Android (non testable hors appareil) et s'appuie sur le
> pipeline de capture (#4).

### Étapes
1. `gninou --display 0` → reflète l'écran principal (comportement par défaut).
2. `gninou --new-display` → crée un affichage virtuel aux dimensions de l'appareil.
3. `gninou --new-display 1920x1080/420` → affichage virtuel 1080p à 420 dpi.
4. `gninou --new-display 1920x1080 --display 0` → erreur clap (options incompatibles).
5. `gninou --new-display 0x1080` → erreur « dimensions non nulles » avant déploiement.

### Résultat attendu
1. Miroir de l'écran principal.
2–3. Un second écran (vide / lanceur) apparaît dans la fenêtre gninou, sur lequel
   on peut déplacer des applications ; l'écran physique reste indépendant.
4. Message d'erreur clair (incompatibilité).
5. Erreur de validation avant tout déploiement.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ parsing options testé (client + serveur) ; création réelle du display
  virtuel à valider sur matériel Android 10+
