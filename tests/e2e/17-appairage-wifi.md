# Test e2e #17 — Appairage sans fil (Wi-Fi, Android 11+)

Couvre la fonctionnalité #17 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Validation du code (6 chiffres) et construction des arguments `adb pair` :

```bash
cd client && cargo test
```

Attendu : tests de `device::connection` verts (code valide/invalide, `pair_args`
complets, rejet adresse/code). ✅ 72 tests au total.

## Procédure manuelle (appareil réel)

### Préconditions
- Android 11 ou plus récent, PC et appareil sur le même réseau.
- Sur l'appareil : Options développeur → **Débogage sans fil** → activer, puis
  **Associer l'appareil à l'aide d'un code**. Un dialogue affiche une adresse
  `ip:port` d'appairage **et** un code à 6 chiffres.

### Étapes
1. `gninou --pair 192.168.1.42:37123 --pair-code 123456 --tcpip 192.168.1.42:5555`
   - `37123` = port d'appairage affiché (aléatoire), `5555` (ou le port affiché
     sur l'écran principal « Débogage sans fil ») = port de connexion.
2. `gninou --pair 192.168.1.42:37123` (sans `--pair-code`) → erreur clap :
   l'option exige `--pair-code`.
3. `gninou --pair 192.168.1.42:37123 --pair-code abc --tcpip 192.168.1.42:5555`
   → erreur « code d'appairage invalide » avant toute commande adb.

### Résultat attendu
1. `adb pair` réussit, puis `adb connect` réussit ; l'appareil est piloté sans fil
   sans câble USB préalable.
2. Message d'erreur clair de la CLI (les deux options vont de pair).
3. Erreur de validation du code (6 chiffres) avant l'appel adb.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ validation code + arguments testés (unit) ; appairage réseau réel à
  valider sur matériel Android 11+
