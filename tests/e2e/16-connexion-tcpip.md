# Test e2e #16 — Connexion TCP/IP

Couvre la fonctionnalité #16 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Validation/normalisation de l'adresse `ip:port` :

```bash
cd client && cargo test
```

Attendu : tests de `device::connection` verts (port par défaut, port invalide,
hôte vide, normalisation, serial TCP/IP). ✅ 68 tests au total.

## Procédure manuelle (appareil réel)

### Préconditions
- Appareil et PC sur le même réseau ; débogage sans fil / `adb tcpip 5555` activé
  (une première connexion USB peut être nécessaire pour l'activer).

### Étapes
1. `gninou --tcpip 192.168.1.42` (port 5555 par défaut).
2. `gninou --tcpip 192.168.1.42:5555` (port explicite).
3. `gninou --tcpip 192.168.1.42:abc` → erreur de format côté client.

### Résultat attendu
1. `adb connect 192.168.1.42:5555` réussit ; l'appareil est détecté et piloté sans fil.
2. Même résultat avec le port explicite.
3. Erreur claire « port invalide » avant toute tentative de connexion.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ validation d'adresse testée (unit) ; connexion réseau réelle à valider sur matériel
