# Test e2e #28 — Découverte mDNS (Android 11+)

Couvre la fonctionnalité #28 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Analyse de `adb mdns services` et filtrage des services connectables :

```bash
cd client && cargo test
```

Attendu : `device::mdns::*` verts (parsing 3 services, exclusion du service
d'appairage, port requis, entrée vide). ✅ 123 tests client.

## Procédure manuelle (appareil réel, Android 11+)

### Préconditions
- Appareil et PC sur le même réseau ; **Débogage sans fil** activé sur l'appareil
  (il s'annonce alors en mDNS). Première association éventuelle via `--pair` (#17).

### Étapes
1. `gninou --wifi-discover`
   → liste mDNS, choisit le 1er service `_adb-tls-connect._tcp` / `_adb._tcp`
   et s'y connecte, **sans câble ni IP**.
2. `gninou --wifi-discover --wifi` → erreur clap (modes incompatibles).

### Résultat attendu
1. Connexion réseau établie automatiquement à l'appareil découvert ; le service
   d'appairage (`_adb-tls-pairing._tcp`) est ignoré pour la connexion.
2. Erreur claire (options mutuellement exclusives).

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ parsing/filtrage mDNS testés (unit) ; découverte réseau réelle
  (`adb mdns services`) à valider sur matériel Android 11+
