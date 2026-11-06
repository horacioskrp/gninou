# Test e2e #14 — Transfert audio

Couvre la fonctionnalité #14 de [FEATURES.md](../../docs/FEATURES.md). Audio Android 11+.

## Tests unitaires (automatisés)

- Client : codec audio + stats du pipeline (`media::audio`).
  ```bash
  cd client && cargo test
  ```
  ✅ 59 tests au total.
- Serveur : résolution du codec (`AudioCodec`).
  ```bash
  cd server && ./gradlew test
  ```

## Procédure manuelle (appareil réel)

### Préconditions
- Android 11+ (capture de lecture), chaîne vidéo opérationnelle, SDK configuré.
- Son en cours de lecture sur l'appareil.

### Étapes
1. `gninou` (audio activé par défaut) → écouter le son du PC.
2. `gninou --audio-codec aac` → vérifier l'encodage AAC.
3. `gninou --no-audio` → aucun flux audio.
4. `gninou --audio-codec mp3` → erreur côté client.

### Résultat attendu
1. L'audio de l'appareil sort sur les haut-parleurs du PC, synchronisé.
2. Codec respecté (ou repli documenté).
3. Pas d'audio quand désactivé.
4. Erreur claire « codec audio inconnu ».

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ codec + pipeline testés (unit) ; capture AudioRecord + lecture réelle à valider sur matériel
