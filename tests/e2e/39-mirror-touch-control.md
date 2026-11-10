# Test e2e #39 — GUI : navigation tactile du miroir + dimensions correctes

Couvre la fonctionnalité #39 de [FEATURES.md](../../docs/FEATURES.md).

## Principe

La fenêtre de miroir devient **interactive** : on **pilote** l'appareil à la
souris. Un **clic** dans l'image devient un *tap*, un **glisser** un *swipe*
(défilement), et trois boutons envoient les touches de navigation Android
(**Retour / Accueil / Récents**). Les positions écran sont converties en
coordonnées **pixel de l'appareil** et injectées via `adb shell input`.

En même temps, l'affichage est corrigé : la fenêtre utilise une **barre d'outils
en haut**, la **navigation + l'état en bas** et l'**image au centre**, si bien que
l'image occupe exactement la place restante et que ses **proportions coïncident**
avec l'écran réel (plus de hauteur « empruntée » par la zone de contrôle).

- Le **view-model** du contrôle (`gui::control`) est pur et testé : conversion
  position → pixel appareil (bornée), classement tap/swipe, construction des
  arguments `adb` (tap/swipe/keyevent).
- L'**injection** (`device::input::inject`) et le fil d'entrées non bloquant
  (`InputSink` dans `gui::app`) sont l'I/O.

## Tests unitaires (automatisés)

- `gui::control` : `mappe_les_coins`, `borne_hors_image`,
  `image_ou_appareil_degenere`, `tap_vs_swipe`, `geste_produit_tap_ou_swipe`,
  `args_adb_tap_swipe_key`, `keycodes_navigation`.

```bash
cd client && cargo test
```

Attendu : `gui::control::*` verts. ✅ 223 tests client (view-model testé **sans**
la feature `gui`).

## Build

```bash
cd client && cargo build --features gui
```

Attendu : compilation sans erreur.

## Procédure manuelle (appareil réel)

### Préconditions
- Binaire compilé avec `--features gui`.
- **Un appareil Android réel branché et prêt** (`adb devices` → `device`). Les
  entrées passent par `adb shell input`, donc adb doit piloter l'appareil.

### Étapes
1. `gninou --gui`, onglet **Appareils** → **« 🖥 Miroir »** sur l'appareil prêt.
2. L'écran réel s'affiche ; vérifier que **les proportions de l'image collent à
   celles du téléphone** (pas d'étirement ni de débordement). La barre de titre
   indique la résolution (ex. `1080×2280`).
3. **Cliquer** sur une icône/bouton à l'écran → le *tap* s'applique sur le
   téléphone (l'app s'ouvre, le bouton réagit).
4. **Glisser** verticalement dans l'image → l'écran **défile** (swipe).
5. Cliquer **◀ Retour**, **⬤ Accueil**, **▢ Récents** → la navigation Android
   répond en conséquence.
6. Redimensionner la fenêtre → l'image se réajuste en gardant les proportions ;
   les clics restent **alignés** sur les bons éléments.
7. Fermer la fenêtre → les fils de capture **et** d'entrées s'arrêtent (join au
   `Drop`).

### Résultat attendu
1–7. On pilote réellement l'appareil depuis la fenêtre (tap/swipe/navigation),
l'image respecte les proportions de l'écran et les clics tombent au bon endroit à
toutes les tailles de fenêtre ; fermeture propre.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ view-model du contrôle testé (unit) et build GUI vérifié ;
  injection `adb input` + alignement des clics à confirmer à l'écran (nécessite
  un appareil prêt). Le débit d'affichage reste celui de l'échantillonnage
  screencap (#38) ; la saisie de texte au clavier et le HID matériel (#8–#10)
  restent à venir.
