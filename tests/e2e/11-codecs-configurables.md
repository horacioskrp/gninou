# Test e2e #11 — Codecs vidéo configurables (négociation)

Couvre la fonctionnalité #11 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

- Client : validation/normalisation de `--video-codec` (`media::codec`).
  ```bash
  cd client && cargo test
  ```
  ✅ 46 tests au total.
- Serveur : négociation avec repli (`VideoCodec.negotiate`).
  ```bash
  cd server && ./gradlew test
  ```
  (nécessite Gradle + android.jar).

## Procédure manuelle (appareil réel)

### Préconditions
- Chaîne vidéo opérationnelle (Phase 2), SDK configuré.

### Étapes
1. `gninou --video-codec h265` → vérifier l'encodage HEVC côté logs.
2. `gninou --video-codec av1` sur un appareil sans encodeur AV1 → repli attendu.
3. `gninou --video-codec vp9` → erreur côté client avant tout déploiement.

### Résultat attendu
1. HEVC utilisé si disponible.
2. Repli sur H.264 (ou premier codec dispo) signalé dans les logs.
3. Erreur claire « codec vidéo inconnu » pour un codec non géré.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ validation (client) + négociation (serveur) testées en logique ; requête
  réelle `MediaCodecList` + mesure à valider sur matériel
