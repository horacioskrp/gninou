# Test e2e #4 — Capture écran (SurfaceControl)

Couvre la fonctionnalité #4 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (logique pure)

Dimensionnement (ratio + alignement 8 px), côté serveur :

```bash
cd server && ./gradlew test
```

Attendu : `DisplayInfoTest` vert. ⚠️ Nécessite l'environnement Gradle + `android.jar`
configuré (`server/gradle.properties`) ; non exécutable dans l'environnement de dev actuel.

## Procédure manuelle (appareil réel)

> Cette fonctionnalité produit une image sur une `Surface` ; sans l'encodeur (#5),
> on la valide en branchant la capture sur une surface de test / un `ImageReader`.

### Préconditions
- Serveur compilé contre un `android.jar` de plateforme (APIs masquées de `SurfaceControl`).
- Appareil branché, débogage USB autorisé.

### Étapes
1. Démarrer le serveur avec une surface de test (ou via #5 une fois dispo).
2. Lire les logs serveur.
3. Vérifier qu'au moins une image/frame arrive sur la surface.

### Résultat attendu
1. Log « capture écran <W>x<H> -> <w>x<h> (rot r) ».
2. Des frames arrivent (contenu de l'écran réel).
3. `release()` détruit le display virtuel sans laisser de trace.

### Résultat observé
- Date : _à remplir_
- Appareil(s) / version Android : _à remplir_
- Statut : ⬜ en attente (APIs masquées — dépend de la version d'Android, cf. TODO DisplayInfo.primary)
