# Test e2e #3 — Tunnel adb + ouverture des sockets

Couvre la fonctionnalité #3 de [FEATURES.md](../../docs/FEATURES.md) et sert de
**test de clôture de la Phase 1** (handshake client ↔ serveur).

## Tests unitaires (automatisés)

```bash
cd client && cargo test
```

Attendu : les tests de `device::tunnel` passent — arguments `adb reverse`/`forward`
(+ `--remove`) et capacité à écouter sur un port local. ✅ 18 tests verts au total.

## Procédure manuelle (handshake, appareil réel)

### Préconditions
- Serveur embarqué (build `GNINOU_BUILD_SERVER=1`), appareil branché (#1, #2 OK).

### Étapes
1. Lancer le client : `cargo run` (ou l'exécutable).
2. Observer la séquence de logs.
3. Vérifier la redirection : `adb reverse --list` doit montrer `localabstract:gninou`.

### Résultat attendu (logs, dans l'ordre)
1. « serveur poussé sur <SERIAL> »
2. « tunnel Reverse ouvert (localabstract:gninou ↔ tcp:27183) »
3. « serveur lancé sur <SERIAL> »
4. « connexion établie avec le serveur (handshake) »
5. Côté serveur : `[gninou-server] connecté au tunnel (gninou)`
6. `adb reverse --list` liste bien `localabstract:gninou`.
7. À l'arrêt du client, la redirection est retirée (Drop → `adb reverse --remove`).

### Résultat observé
- Date : _à remplir_
- Appareil(s) : _à remplir_
- Statut : ⬜ en attente (nécessite le jar réel + matériel) — **clôture Phase 1**
