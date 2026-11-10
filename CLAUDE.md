# Bandua

Application gratuite de déchargement de rushes de Domingo Olmo Martin (Dom), directeur de la photographie. Elle copie une carte caméra vers une ou plusieurs destinations en une seule lecture, vérifie chaque copie par hash en la relisant depuis le disque, et écrit dans chaque dossier daté un rapport PDF, un log, un MHL 1.1 et un JSON.

Windows et macOS (binaire universel). Linux retiré le 8 novembre 2026 faute de pouvoir le tester, mais le code doit rester compatible Linux. Licence MIT. Version publiée : v0.2.0 (pré-version bêta).

**Lire `docs/ROADMAP.md`** avant tout travail de fond : état du projet, priorités, choix et pistes écartées.

## Règles de travail

- **Ne jamais lancer de fabrication GitHub Actions sans que Dom le demande explicitement.** Cela inclut « Run workflow » (`workflow_dispatch`) et la publication d'une Release, qui déclenche aussi `build.yml`.
- **Toujours passer par une branche et une pull request**, fusionnée sur GitHub (commits « Verified »). Ne jamais pousser directement sur `main`.
- **Aucun tiret cadratin (caractère U+2014)** dans les textes écrits pour Dom : interface, rapports, README, notes de version, messages de commit. Utiliser virgules, points ou parenthèses.
- Échanges avec Dom en français. Commentaires et messages de commit en français, comme le code existant.
- Interface et rapports **toujours en trois langues** (français, anglais, espagnol) : toute nouvelle chaîne s'ajoute dans `core/src/i18n.rs` (moteur, rapports) ou `ui/i18n.js` (interface), dans les trois langues.
- README : `README.md` (anglais, langue principale), `README.fr.md`, `README.es.md`, à tenir synchronisés. Notes de version en anglais, avec FR et ES repliés.
- Interface sombre. Le vert et le rouge sont réservés à « vérifié » et « erreur ».
- Crédits « Bandua by Domingo Olmo Martin » partout (app, rapports, MHL, installeur), avertissement de non-responsabilité conservé.
- Les vignettes et métadonnées de clips dans le PDF ont été **abandonnées** : ne pas les réintroduire.

## La fiabilité d'abord

Une appli de déchargement qui n'est pas irréprochable ne sert à rien. Toute modification du chemin de copie doit préserver :
- l'écriture en `.bandua-part` et le renommage final **seulement** si le hash correspond, sinon `.HASH_ERREUR` ;
- la relecture des copies sans le cache système (`core/src/rawio.rs` : FILE_FLAG_NO_BUFFERING, F_NOCACHE + F_FULLFSYNC, posix_fadvise) ;
- la seconde lecture de la carte (option activée par défaut) et la surveillance de la source (taille et date avant et après chaque fichier, inventaire final) ;
- les tests de pannes simulées. Ajouter un test pour tout nouveau cas.

## Organisation

```
core/                 bandua-core (moteur, sans interface)
  src/lib.rs          run() (déchargement), scan(), reverify(), verify_mhl(), folder_name()
  src/clips.rs        regroupement des fichiers en clips par marque (RED .RDC, ARRIRAW, BRAW…)
  src/hash.rs         xxHash64 (par défaut), XXH3, MD5, SHA-1
  src/rawio.rs        lectures sans cache, tampon aligné, lectures bornées
  src/report.rs       PDF, log, MHL 1.1, JSON
  src/pdf.rs          générateur PDF maison, sans dépendance
  src/i18n.rs         textes FR / EN / ES du moteur et des rapports
  src/bin/cli.rs      bandua-cli <source> <destinations…> --lang fr|en|es
src-tauri/            application Tauri 2 (src/main.rs : start_offload, start_reverify, cancel_offload…)
ui/                   index.html, app.js, i18n.js : HTML/CSS/JS sans npm ni framework
docs/                 ROADMAP.md et captures d'écran
.github/workflows/build.yml   fabrication Windows + macOS universel
```

L'interface parle au moteur par `invoke` et reçoit la progression par l'événement `offload`. Sans `window.__TAURI__`, `ui/` tourne en **mode démo** dans un navigateur.

## Commandes

```
cargo test -p bandua-core             # à lancer avant chaque PR
cargo run -p bandua-core --bin bandua-cli -- <source> <dest1> [dest2…] --lang fr
cargo install tauri-cli --version "^2" --locked
cd src-tauri && cargo tauri dev       # lancer l'app
```

## Publier une version

Changer le numéro dans `core/Cargo.toml`, `src-tauri/Cargo.toml` et `src-tauri/tauri.conf.json` (l'interface lit la version de Tauri au démarrage, la valeur par défaut de `ui/app.js` peut suivre). Mettre à jour `docs/ROADMAP.md`. Dom crée ensuite la Release `vX.Y.Z` lui-même. Ne pas supprimer la Release v0.1.0.

## Apps sœurs

- [Bandua MHL Companion](https://github.com/tetsujin28/bandua-mhl-companion) partage `hash.rs`, `rawio.rs` et `clips.rs` (versions dérivées). Une correction de fiabilité faite ici est probablement à reporter là-bas, et inversement : le signaler à Dom.
- [Ataecina](https://github.com/tetsujin28/ataecina) (récupération et réparation de rushes).
- Site commun : https://tetsujin28.github.io (dépôt `tetsujin28.github.io`).
