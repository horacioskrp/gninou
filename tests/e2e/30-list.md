# Test e2e #30 — Découverte enrichie `--list`

Couvre la fonctionnalité #30 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Logique pure d'agrégation et de formatage de la vue unifiée : classement
USB/réseau d'une série, fusion appareils adb + services mDNS, déduplication
d'un appareil déjà connecté, et rendu du tableau.

```bash
cd client && cargo test
```

Attendu : `device::listing::*` verts (réseau vs USB, classe USB/réseau, ajout
des services annoncés, déduplication, tableau vide lisible, en-tête + lignes).
✅ 136 tests client.

## Procédure manuelle (appareil réel)

### Préconditions
- `adb` dans le PATH. Au moins un appareil branché en USB **et/ou** un appareil
  en « Débogage sans fil » (Android 11+) sur le même réseau.

### Étapes
1. `gninou --list`
   → affiche un tableau `Type | Appareil | État` regroupant :
     - les appareils USB (série, « prêt » / « non autorisé » / « hors ligne ») ;
     - les appareils déjà connectés en TCP/IP (adresse `ip:port`, Wi-Fi) ;
     - les appareils seulement **annoncés** en mDNS (« annoncé (connexion/appairage, mDNS) »).
2. Aucun appareil : message clair « Aucun appareil détecté ».
3. `gninou --list` **n'effectue aucun déploiement** (pas de push serveur, pas de
   tunnel) et quitte immédiatement.

### Résultat attendu
1. Vue unifiée correcte ; un appareil déjà connecté n'apparaît pas en double
   avec son annonce mDNS.
2. Message lisible quand rien n'est détecté.
3. Sortie immédiate, sans effet de bord.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ agrégation/formatage testés (unit) ; collecte réelle
  (`adb devices` + `adb mdns services`) à valider sur matériel
