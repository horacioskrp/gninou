# Test e2e #33 — GUI : écran d'appairage Wi-Fi (assistant)

Couvre la fonctionnalité #33 de [FEATURES.md](../../docs/FEATURES.md).

## Principe

Version graphique de l'assistant d'appairage (#29) : un écran dédié dans la
fenêtre (feature `gui`), avec adresse d'appairage auto-découverte en mDNS ou
saisie, code à 6 chiffres validé en direct, puis adresse de connexion. Le
**view-model** (`gui::pairing`) est pur et toujours compilé — donc testé dans le
build par défaut ; seul le rendu egui est derrière la feature.

## Tests unitaires (automatisés)

Transitions pures du formulaire : état initial, validation du code en direct,
conditions `can_pair`/`can_connect`, remplissage par découverte mDNS, flux
appairage → connexion, refus d'un code invalide, réessai après échec, reset ;
plus la navigation entre écrans dans `gui::state`.

```bash
cd client && cargo test
```

Attendu : `gui::pairing::*` et `gui::state::navigation_*` verts. ✅ 160 tests
client (view-models GUI testés **sans** la feature `gui`).

## Build

```bash
cd client && cargo build --features gui
```

Attendu : compilation sans erreur.

## Procédure manuelle (écran réel, Android 11+)

### Préconditions
- Binaire compilé avec `--features gui`. `adb` dans le PATH. Appareil avec
  « Débogage sans fil » activé, « Associer l'appareil à l'aide d'un code ».

### Étapes
1. `gninou --gui`, puis onglet **Appairage Wi-Fi** (en-tête).
2. Cliquer **Détecter (mDNS)** → l'adresse d'appairage (et de connexion) se
   remplit si l'appareil est annoncé ; sinon, saisir l'adresse affichée.
3. Saisir le **code à 6 chiffres** → retour immédiat « Code valide » / « 6
   chiffres requis ». Le bouton **Appairer** s'active quand adresse + code sont
   corrects.
4. Cliquer **Appairer** → `adb pair` ; message « Appairage réussi » puis l'état
   passe à « appairé ».
5. Vérifier/saisir l'**adresse de connexion**, cliquer **Connecter** →
   `adb connect` ; message « Connecté : ip:port » (vert).
6. **Réinitialiser** vide le formulaire ; un code refusé permet de réessayer.

### Résultat attendu
1–6. Appairage puis connexion réalisés depuis la fenêtre, sans ligne de
commande ; validations et messages clairs à chaque étape.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ view-model (validation, flux, navigation) testé (unit) et build
  GUI vérifié ; rendu egui + `adb pair`/`adb connect` réels à valider à l'écran
  sur Android 11+. Opérations synchrones pour cet incrément.
