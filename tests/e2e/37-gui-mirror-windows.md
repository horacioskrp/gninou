# Test e2e #37 — GUI : fenêtre de miroir dédiée + liste d'appareils enrichie

Couvre la fonctionnalité #37 de [FEATURES.md](../../docs/FEATURES.md).

## Principe

Depuis la **liste des appareils** (onglet Appareils), un bouton **« 🖥 Miroir »**
ouvre l'appareil dans **sa propre fenêtre** (miroir + emplacement du contrôle), à
la manière d'un gestionnaire d'appareils type Android Studio. Plusieurs appareils
peuvent être ouverts en parallèle, chacun dans sa fenêtre avec son propre zoom,
sa capture et son enregistrement. La liste est enrichie : **icônes** de type
(🔌 USB / 📶 Wi-Fi) et d'état (🟢 prêt / 🟠 non autorisé / 🔴 hors ligne / 🔵
annoncé), identifiant en police à chasse fixe, compteur d'appareils et astuce.

Le view-model des fenêtres (`gui::windows`) et les aides d'icônes
(`device::listing`) sont purs et testés ; l'ouverture réelle des fenêtres
(viewports egui immédiats), le rendu et les écritures PNG vivent dans `gui::app`.

## Tests unitaires (automatisés)

- `device::listing` : icônes de type et d'état, `Entry::is_ready` (un appareil
  seulement « annoncé » en mDNS n'est pas prêt).
- `gui::windows` : ouverture dédupliquée, fermeture (y compris d'un inconnu),
  accès mutable avec état propre par fenêtre, ordre des identifiants ; plus la
  navigation inchangée dans `gui::state`.

```bash
cd client && cargo test
```

Attendu : `device::listing::*` et `gui::windows::*` verts. ✅ 213 tests client
(view-models testés **sans** la feature `gui`).

## Build

```bash
cd client && cargo build --features gui
```

Attendu : compilation sans erreur.

## Procédure manuelle (écran réel)

### Préconditions
- Binaire compilé avec `--features gui`. Pour ouvrir une fenêtre de miroir, il
  faut **au moins un appareil « prêt »** (USB autorisé, ou déjà connecté en
  Wi-Fi) ; sans appareil, le bouton « 🖥 Miroir » reste désactivé (c'est voulu).

### Étapes
1. `gninou --gui`, onglet **Appareils**. Vérifier les **icônes** de type/état,
   l'identifiant en police fixe et le compteur d'appareils.
2. Sur une ligne d'appareil **prêt**, cliquer **« 🖥 Miroir »** → une **nouvelle
   fenêtre** s'ouvre, intitulée « gninou — miroir : \<id\> », avec l'image et la
   barre d'outils (zoom/ajustement, pause, capture, enregistrement, plein écran).
   Le bouton de la ligne devient « 🖥 Ouvert ».
3. Ouvrir un **second** appareil prêt → une deuxième fenêtre indépendante.
4. Dans une fenêtre : zoomer, capturer (PNG dans `gninou-captures/`), enregistrer
   (dossier `rec-…/`), basculer en plein écran — chaque fenêtre a son propre état.
5. Sous l'image, la section **Contrôle** rappelle que clavier/souris/HID
   (#8–#10) ne sont pas encore branchés.
6. Fermer une fenêtre (croix) → elle disparaît, et le bouton de sa ligne
   redevient « 🖥 Miroir ».

### Résultat attendu
1–6. La liste est lisible et conviviale ; ouvrir un appareil l'affiche dans sa
propre fenêtre ; plusieurs fenêtres cohabitent ; capture/enregistrement/zoom
fonctionnent par fenêtre ; la fermeture est propre.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ view-models (fenêtres, icônes, état) testés (unit) et build GUI
  vérifié ; ouverture réelle des fenêtres (viewports), rendu et écritures PNG à
  valider à l'écran (nécessite un appareil prêt). L'image reste la **mire de
  test** tant que le décodage vidéo réel (#6) n'alimente pas le rendu ; le
  contrôle (#8–#10) est un emplacement réservé.
