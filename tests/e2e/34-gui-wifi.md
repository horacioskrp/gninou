# Test e2e #34 — GUI : écran « Connexion Wi-Fi »

Couvre la fonctionnalité #34 de [FEATURES.md](../../docs/FEATURES.md).

## Principe

Un écran dédié (feature `gui`) qui regroupe les commodités Wi-Fi déjà en place :
la bascule USB → sans-fil sans IP à saisir (#27), la découverte mDNS d'un
appareil annonçant sa connexion (#28) et la reconnexion automatique à délai
exponentiel (#31). Le **view-model** (`gui::wifi`) est pur et toujours compilé —
donc testé dans le build par défaut ; seul le rendu egui et les I/O
(`adb tcpip`/`adb connect`, mDNS) sont derrière la feature.

## Tests unitaires (automatisés)

Transitions pures : état initial, validation du port (1–65535), nombre de
tentatives tolérant (vide/0/illisible), politique de backoff selon la
reconnexion, conditions `can_switch`/`can_connect_discovered`, refus série/port
manquants, flux bascule → connexion, découverte qui remplit/vide, reset,
normalisation de l'adresse découverte ; plus la navigation vers l'écran dans
`gui::state`.

```bash
cd client && cargo test
```

Attendu : `gui::wifi::*` et `gui::state::navigation_*` verts. ✅ 174 tests
client (view-models GUI testés **sans** la feature `gui`).

## Build

```bash
cd client && cargo build --features gui
```

Attendu : compilation sans erreur.

## Procédure manuelle (écran réel)

### Préconditions
- Binaire compilé avec `--features gui`. `adb` dans le PATH. Un appareil USB
  avec débogage activé (pour la bascule), et/ou un appareil au « Débogage sans
  fil » actif sur le même réseau (pour la découverte mDNS, Android 11+).

### Étapes
1. `gninou --gui`, puis onglet **Connexion Wi-Fi** (en-tête). Si un appareil USB
   est détecté, sa série pré-remplit le champ.
2. **Depuis un appareil USB** : vérifier/saisir la série et le port (défaut
   5555), cliquer **Basculer en Wi-Fi** → lecture de l'IP Wi-Fi, `adb tcpip`
   puis `adb connect` ; message « Basculé en Wi-Fi et connecté : ip:port » (vert).
3. **Sur le réseau local** : cliquer **Détecter (mDNS)** → l'adresse annoncée
   s'affiche si l'appareil est trouvé ; **Connecter** lance `adb connect`.
4. **Fiabilité** : cocher **Reconnexion automatique** et régler le nombre de
   tentatives → la bascule et la connexion sont relancées avec un délai
   croissant jusqu'à ce nombre en cas d'échec.
5. **Réinitialiser** vide l'écran.

### Résultat attendu
1–5. Passage à une connexion sans-fil et connexion à un appareil réseau depuis
la fenêtre, sans ligne de commande ; validations et messages clairs, reprise
automatique optionnelle.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ view-model (validation, backoff, flux, navigation) testé (unit) et
  build GUI vérifié ; rendu egui + `adb tcpip`/`adb connect`/mDNS réels à valider
  à l'écran. Opérations synchrones pour cet incrément.
