# Test e2e #18 — Multi-appareils

Couvre la fonctionnalité #18 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Sélection de plusieurs appareils (`--all`, liste explicite, doublons, série
inconnue, repli sur l'appareil unique) :

```bash
cd client && cargo test
```

Attendu : tests `adb::select_devices` verts. ✅ 78 tests au total.

## Procédure manuelle (appareil réel)

### Préconditions
- Au moins deux appareils en ligne (USB et/ou réseau déjà connectés), débogage
  activé et autorisé sur chacun.

### Étapes
1. `gninou --serial APPAREIL_A --serial APPAREIL_B`
   → pilote les deux appareils indiqués.
2. `gninou --all`
   → pilote **tous** les appareils en ligne.
3. `gninou --all --serial APPAREIL_A`
   → erreur clap : `--all` est incompatible avec `--serial`.
4. `gninou --serial INCONNU`
   → erreur « aucun appareil avec le numéro de série … ».

### Résultat attendu
1. Un déploiement serveur + tunnel par appareil, chacun sur un port local
   distinct (27183, 27184, …) ; pas de collision de socket.
2. Tous les appareils en ligne sont ciblés ; les appareils hors ligne sont exclus.
3. Message d'erreur clair (options incompatibles).
4. Erreur de sélection avant tout déploiement.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ logique de sélection testée (unit) ; pilotage parallèle réel à
  valider sur matériel (dépend du pipeline média/contrôle à venir)
