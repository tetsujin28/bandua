<p align="center">
  <img src="src-tauri/icons/128x128.png" width="96" alt="Logo de Bandua">
</p>

<h1 align="center">Bandua</h1>
<p align="center"><b>by Domingo Olmo Martin</b></p>
<p align="center">
  <a href="README.md">English</a> · <a href="README.fr.md">Français</a> · <b>Español</b>
</p>

Bandua vuelca tus tarjetas de cámara a varios discos a la vez, verifica cada copia frente al origen mediante un hash y genera un log, un MHL y un informe PDF para entregar al montador o al laboratorio.

Gratuito, para Windows y macOS. Interfaz e informes en español, inglés y francés.

> Bandua es una divinidad protectora del oeste de la península ibérica. Aquí, vela por tu material.

<p align="center"><img src="docs/screenshots/offload_es.png" width="860" alt="Volcado en curso"></p>

## Descargar

Ve a la **[página de Releases](https://github.com/tetsujin28/bandua/releases)** y elige el archivo para tu ordenador:

| Sistema | Archivo |
|---|---|
| Windows (sin instalación) | `Bandua_x.y.z_portable.exe` |
| Windows (instalador) | `Bandua_x.y.z_x64-setup.exe` o `.msi` |
| macOS (Intel y Apple Silicon) | `Bandua_x.y.z_universal.dmg` |

**Primer arranque.** La aplicación todavía no está firmada, así que tu sistema mostrará un aviso una sola vez:
- **Windows**: haz clic en «Más información» y después en «Ejecutar de todas formas».
- **macOS**: haz clic derecho en la aplicación, elige «Abrir» y confirma.

Tu navegador también puede marcar la descarga como inusual o peligrosa: es habitual con una aplicación reciente y sin firmar. Conserva el archivo desde el panel de descargas y, si tienes dudas, analízalo en [VirusTotal](https://www.virustotal.com).

## Qué hace

- **Copia en una sola pasada** a 1, 2 o más destinos al mismo tiempo.
- **Verifica cada copia** releyéndola desde el disco de destino y comparando los hash (xxHash64 por defecto, XXH3, MD5 o SHA-1). La relectura evita la caché del sistema: se verifica el disco de verdad, no una copia en memoria. Un archivo solo recibe su nombre definitivo cuando está verificado.
- **Relee la tarjeta una segunda vez** (opción activada por defecto) para detectar un lector de tarjetas, un cable o una tarjeta defectuosos.
- **Vigila la tarjeta durante el volcado**: un archivo que cambia, aparece o desaparece hace fallar el volcado, para no formatear nunca una tarjeta que todavía se estaba escribiendo.
- **Crea una carpeta con fecha** en cada destino, por ejemplo `2026-11-06_1633_PROYECTO_D03_A_A003`.
- **Copia la tarjeta tal cual**: RED (.RDM / .RDC), ARRI (ARRIRAW, MXF, ProRes), Blackmagic (BRAW), Sony, Canon, GoPro, DJI… sin los archivos de sistema innecesarios.
- **Reconoce los clips** para listarlos correctamente en el informe (una carpeta .RDC de RED = un clip).
- **Escribe 4 informes** en cada carpeta con fecha: un informe PDF, un log de texto, un archivo MHL (legible por Silverstack, ShotPut, Hedge…) y un archivo JSON.
- **Reverifica** una carpeta existente a partir de su MHL, antes de formatear una tarjeta o entregar un disco: progreso, estado de cada clip (verificado, dañado, falta), archivos añadidos a la carpeta señalados, informe PDF y log escritos junto al MHL.
- **Comprueba el espacio libre** y te avisa antes de empezar.
- **Botón «Nuevo volcado»** para encadenar tarjetas: se conservan los destinos y los datos del proyecto.

## Uso

1. Haz clic en **Origen** y selecciona la tarjeta.
2. Añade uno o varios **destinos**.
3. Rellena el proyecto, el día de rodaje, la cámara y la tarjeta (el número de tarjeta se rellena automáticamente a partir de los nombres de los clips).
4. Haz clic en **Iniciar el volcado**. El círculo indica la parte de la tarjeta copiada, y luego «verificando / espere» durante los últimos controles.
5. Espera a que el círculo se ponga verde y al aviso **TARJETA VERIFICADA** antes de formatear la tarjeta. Un clic en un destino abre la carpeta copiada.

Para comprobar un disco más tarde (antes de formatear una tarjeta o al entregar un disco), haz clic en **Reverificar un MHL** y elige el archivo `.mhl` de la carpeta con fecha.

## Capturas de pantalla

<table>
  <tr>
    <td width="62%"><img src="docs/screenshots/verified_es.png" alt="Tarjeta verificada"><br><sub>Tarjeta verificada</sub></td>
    <td width="38%"><img src="docs/screenshots/report_es.png" alt="Informe PDF entregado al montador o al laboratorio"><br><sub>Informe PDF entregado al montador o al laboratorio</sub></td>
  </tr>
</table>

## Compilar desde el código fuente

Requiere [Rust](https://rustup.rs). No hace falta npm.

```
cargo install tauri-cli --version "^2" --locked
cd src-tauri
cargo tauri dev      # ejecutar
cargo tauri build    # generar los instaladores
```

Cada Release publicada genera automáticamente los archivos para Windows y macOS con GitHub Actions.

Estructura del proyecto: `core/` (motor de copia, hash e informes, en Rust), `src-tauri/` (aplicación de escritorio), `ui/` (interfaz, HTML/CSS/JS sin framework).

## Licencia y aviso

© 2026 Domingo Olmo Martin. Distribuido bajo [licencia MIT](LICENSE).

Bandua se proporciona «tal cual», sin ninguna garantía. Te ayuda a copiar y verificar tus medios, pero no sustituye tus propios controles ni tus procedimientos de copia de seguridad. El autor no se hace responsable de ninguna pérdida o corrupción de datos, ni de ningún daño derivado del uso del software. **No formatees nunca una tarjeta sin haber comprobado que cada copia está completa y es legible.**
