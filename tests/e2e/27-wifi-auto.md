# Test e2e #27 — Wi-Fi automatique (USB → sans-fil)

Couvre la fonctionnalité #27 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Extraction de l'IPv4 Wi-Fi depuis la sortie de `ip addr` :

```bash
cd client && cargo test
```

Attendu : `device::connection::wlan_ipv4_*` verts (wlan0 présent, loopback
ignoré, absence → erreur). ✅ 119 tests client.

## Procédure manuelle (appareil réel)

### Préconditions
- Appareil branché en **USB**, débogage activé, connecté au même Wi-Fi que le PC.

### Étapes
1. `gninou --wifi`
   → lit l'IP Wi-Fi de l'appareil, `adb tcpip 5555`, `adb connect ip:5555`.
2. Débrancher le câble USB → le miroir continue en sans-fil.
3. `gninou --wifi --wifi-port 5556` → même chose sur un autre port.
4. `gninou --wifi --tcpip 1.2.3.4` → erreur clap (options incompatibles).

### Résultat attendu
1. Connexion réseau établie automatiquement, **sans saisir d'IP**.
2. Miroir maintenu une fois l'USB retiré.
3. Bascule sur le port indiqué.
4. Erreur claire (modes de connexion mutuellement exclusifs).

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ extraction d'IP testée (unit) ; bascule réseau réelle (adb tcpip +
  connect) à valider sur matériel
