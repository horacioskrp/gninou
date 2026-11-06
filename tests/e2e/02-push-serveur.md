# Test e2e #2 — Push + lancement du serveur (app_process)

Couvre la fonctionnalité #2 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Logique pure, sans appareil :

```bash
cd client && cargo test
```

Attendu : les tests de `device::server_bin` passent — construction de la commande
distante `app_process` (avec et sans arguments), nommage du jar temporaire, refus
d'agir quand le serveur embarqué est un placeholder vide. ✅ 13 tests verts au total.

## Procédure manuelle (appareil réel)

### Préconditions
- Serveur compilé et embarqué : build avec `GNINOU_BUILD_SERVER=1`
  (SDK Android requis), sinon `is_placeholder()` est vrai et le déploiement est sauté.
- Appareil branché, débogage USB autorisé (fonctionnalité #1 validée).

### Étapes
1. `scripts/build.sh` (ou `.ps1`) pour embarquer le vrai jar.
2. Lancer le binaire : `cargo run` (ou l'exécutable compilé).
3. Sur l'appareil, vérifier le fichier poussé :
   `adb shell ls -l /data/local/tmp/gninou-server.jar`
4. Vérifier que le process serveur tourne (logs du client : « serveur déployé et lancé »).

### Résultat attendu
1. Build OK, jar non vide dans `client/assets/`.
2. Log client « serveur déployé et lancé sur <SERIAL> ».
3. Le jar est présent sur l'appareil.
4. La sortie du serveur (`[gninou-server] ...`) remonte dans la console.
5. À l'arrêt du client, rien ne subsiste (le process serveur s'arrête).

### Résultat observé
- Date : _à remplir_
- Appareil(s) : _à remplir_
- Statut : ⬜ en attente (nécessite le jar réel + matériel)
