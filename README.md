# Bandua

**by Domingo Olmo Martin**

Déchargement de rushes avec vérification par hash, pour Windows, macOS et Linux.
Interface, log et rapport PDF en français, anglais et espagnol.

Bandua est une divinité protectrice de l'ouest de la péninsule Ibérique : elle veille ici sur vos rushes.

## Ce que fait la V1

- **Une seule lecture de la carte**, écrite en parallèle vers 1 à N destinations.
- **Hash à la volée** (xxHash64 par défaut, XXH3, MD5 ou SHA-1).
- **Vérification réelle** : chaque copie est relue depuis le disque et comparée au hash source.
  Les fichiers sont écrits en `.bandua-part` et ne prennent leur vrai nom qu'une fois vérifiés.
  En cas d'écart, le fichier est renommé `.HASH_ERREUR`.
- **Dossier daté** dans chaque destination : `2026-11-06_1633_PROJET_J03_A_A003`.
- **Copie à l'identique** de toute la carte (structure RED .RDM/.RDC, dossiers ARRI, BRAW + sidecar…),
  sans les fichiers système (`.DS_Store`, `.Spotlight-V100`, `System Volume Information`…).
- **Date de modification d'origine** conservée sur les copies.
- **Contrôles préalables** : espace libre, destination à l'intérieur de la source.
- **Rapports** dans chaque dossier daté :
  - `…_rapport.pdf` : une ligne par clip (un .RDC RED, un dossier ARRIRAW, un .braw…)
  - `…_log.txt` : chaque fichier avec son statut, son hash et sa taille
  - `….mhl` : MHL 1.1, lisible par Silverstack, ShotPut, Hedge, YoYotta
  - `….json` : tout le détail, pour des outils futurs
- **Re-vérification** d'un dossier existant à partir de son MHL.
- Signal sonore et bandeau vert ou rouge à la fin.
- Interface, rapports et log en **français, anglais ou espagnol** (sélecteur FR / EN / ES).
- Avertissement de non-responsabilité à accepter au premier lancement, rappelé dans « À propos » et en pied de chaque rapport.

## Structure

```
core/        moteur Rust (copie, hash, clips, rapports), sans interface
  src/bin/cli.rs   outil en ligne de commande pour tester le moteur
src-tauri/   application Tauri (fenêtre, sélection de dossiers, événements)
ui/          interface HTML/CSS/JS, sans npm ni framework
```

`ui/index.html` s'ouvre aussi directement dans un navigateur : il passe alors en **mode démo**
avec une carte RED simulée.

## Compiler sur Windows (sans npm)

1. Installer Rust : https://rustup.rs (choisir l'option par défaut, avec les Build Tools Visual Studio).
2. WebView2 est déjà présent sur Windows 11.
3. Installer la CLI Tauri (une seule fois) :
   ```
   cargo install tauri-cli --version "^2" --locked
   ```
4. Dans le dossier du projet :
   ```
   cd src-tauri
   cargo tauri dev        # lancer en mode développement
   cargo tauri build      # produire l'installeur (.msi et .exe dans target/release/bundle)
   ```

## macOS et Linux

Même procédure. Sur macOS, il faut les Xcode Command Line Tools (`xcode-select --install`).
Sur Linux (Debian/Ubuntu) : `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev`.

Sans Mac sous la main, le workflow GitHub Actions `.github/workflows/build.yml` compile les trois
plateformes à chaque tag `v*` et dépose les installeurs dans les artefacts.

## Tester le moteur seul

```
cargo run -p bandua-core --release --bin bandua-cli -- E:\ D:\Rushes F:\Backup --project "Due Volti" --day J03 --camera A --card A003
cargo run -p bandua-core --release --bin bandua-cli -- --verify D:\Rushes\2026-11-06_1633_...\2026-11-06_1633_....mhl
cargo test -p bandua-core
```

## Limites connues de la V1

- La relecture de vérification peut être servie en partie par le cache disque du système.
  V2 : ouverture sans cache (`FILE_FLAG_NO_BUFFERING` sous Windows, `F_NOCACHE` sous macOS).
- Pas encore de vignettes ni de métadonnées caméra dans le PDF (V3 : REDline, ARRI Reference Tool,
  Blackmagic RAW SDK, ffmpeg).
- Pas encore de file d'attente de cartes, de glisser-déposer, ni d'éjection automatique.

## Feuille de route

- **V2** : file d'attente, presets par production, détection de carte déjà déchargée,
  lecture sans cache, ASC MHL v2, glisser-déposer, éjection, notifications (ntfy/Telegram),
  
- **V3** : vignettes et métadonnées dans le PDF (TC, ISO, WB, objectif), rapport de journée.

## Licence et avertissement

© 2026 Domingo Olmo Martin. Distribué sous licence MIT (voir `LICENSE`).
Le logiciel est fourni « en l'état », sans garantie. L'auteur n'est pas responsable
des pertes de données ou dommages liés à son utilisation. Ne formatez jamais une
carte avant d'avoir vérifié vos copies.
