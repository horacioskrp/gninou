# Test e2e #19 — Enregistrement (mp4/mkv)

Couvre la fonctionnalité #19 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Choix du conteneur par extension, compatibilité codec ↔ conteneur, compteurs :

```bash
cd client && cargo test
```

Attendu : tests `media::recorder` verts (extensions mp4/mkv/m4v, refus `.avi`,
PCM refusé en mp4 mais accepté en mkv, compat, compteurs). ✅ 85 tests au total.

## Procédure manuelle (appareil réel)

> Le multiplexage réel dépend de la feature `ffmpeg` (à implémenter). Ces étapes
> valident la sélection/validation côté client dès maintenant, et le fichier
> produit une fois `ffmpeg` branché.

### Étapes
1. `gninou --record capture.mp4`
   → enregistrement activé, conteneur MP4 détecté.
2. `gninou --record capture.mkv --audio-codec raw`
   → accepté (MKV gère le PCM brut).
3. `gninou --record capture.mp4 --audio-codec raw`
   → erreur : MP4 ne gère pas le PCM (suggère .mkv ou --no-audio).
4. `gninou --record capture.avi`
   → erreur : extension non gérée.

### Résultat attendu
1. Déploiement normal + fichier `capture.mp4` lisible (une fois `ffmpeg` branché).
2. Fichier `capture.mkv` lisible avec piste audio PCM.
3. Erreur de compatibilité **avant** tout déploiement.
4. Erreur d'extension **avant** tout déploiement.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ sélection conteneur + compat testées (unit) ; écriture réelle du
  fichier à valider une fois la feature `ffmpeg` implémentée
