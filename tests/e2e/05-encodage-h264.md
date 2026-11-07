# Test e2e #5 — Encodage vidéo (MediaCodec)

Couvre la fonctionnalité #5 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (logique pure)

Sélection de codec et calcul de débit :

```bash
cd server && ./gradlew test
```

Attendu : `VideoCodecTest` vert (mime h264/h265/av1, codec inconnu rejeté, débit borné).
⚠️ Nécessite Gradle + `android.jar` configuré ; non exécutable dans l'env de dev actuel.

## Procédure manuelle (appareil réel)

### Préconditions
- Serveur compilé (SDK Android), appareil branché, tunnel OK (#3), capture OK (#4).

### Étapes
1. Démarrer le serveur ; côté client, écrire le flux vidéo reçu dans un fichier brut
   (ex. `out.h264`) sans décoder (le décodage arrive en #6).
2. Laisser tourner ~5 s puis arrêter.
3. Vérifier le flux : `ffprobe out.h264` (ou lecture `ffplay out.h264`).

### Résultat attendu
1. Log serveur « encodage h264 <w>x<h> @ <fps>fps ».
2. `out.h264` non vide, reconnu par ffprobe comme un flux H.264 valide.
3. Débit cohérent avec `VideoCodec.defaultBitrate`.
4. `stop()` arrête proprement encodeur + capture.

### Résultat observé
- Date : _à remplir_
- Appareil(s) / version Android : _à remplir_
- Statut : ⬜ en attente (SDK + matériel)
