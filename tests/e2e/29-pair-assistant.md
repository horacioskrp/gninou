# Test e2e #29 — Assistant d'appairage interactif (Android 11+)

Couvre la fonctionnalité #29 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Logique pure de l'assistant : évaluation d'une saisie de code (valide / à
refaire / abandon) et résolution d'un point d'accès (découverte vs saisie
manuelle normalisée), plus le filtrage mDNS du service d'appairage.

```bash
cd client && cargo test
```

Attendu : `device::assistant::*` verts (code valide accepté, code invalide avec
essais restants → `Retry`, dernier essai → `GiveUp`, préférence découverte,
repli manuel normalisé, absence d'adresse → erreur) et
`device::mdns::filtre_appairage_isole_le_service_pairing`. ✅ 130 tests client.

## Procédure manuelle (appareil réel, Android 11+)

### Préconditions
- Appareil et PC sur le même réseau ; **Débogage sans fil** activé sur
  l'appareil (il s'annonce alors en mDNS).
- Sur l'appareil : « Associer l'appareil à l'aide d'un code d'association »
  (affiche un code à 6 chiffres + une adresse `ip:port`).

### Étapes
1. `gninou --pair-assistant`
   → l'assistant affiche les instructions, détecte le point d'appairage en
     mDNS (`_adb-tls-pairing._tcp`), demande le **code à 6 chiffres**, appaire
     (`adb pair`), détecte le point de connexion (`_adb-tls-connect._tcp`) puis
     s'y connecte.
2. Saisir un code erroné deux fois puis le bon
   → messages « il reste N essai(s) », appairage réussi au 3ᵉ.
3. Saisir un code erroné 3 fois
   → erreur claire « trop de codes invalides » et sortie.
4. Sans mDNS (ou Android < 11) : l'assistant bascule en **saisie manuelle** des
   adresses `ip:port` d'appairage puis de connexion.
5. `gninou --pair-assistant --wifi` → erreur clap (modes incompatibles).

### Résultat attendu
1. Appairage + connexion réseau établis sans saisir d'adresse (code seul).
2. Réessais tolérés jusqu'à 3 ; le bon code aboutit.
3. Abandon propre après 3 échecs.
4. Repli manuel fonctionnel quand la découverte échoue.
5. Erreur claire (options mutuellement exclusives).

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ logique pure (code + résolution d'adresse) et filtrage mDNS testés
  (unit) ; conversation terminal + `adb pair`/`adb connect` réels à valider sur
  matériel Android 11+
