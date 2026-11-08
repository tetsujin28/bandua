//! Traductions des rapports et messages du moteur (fr, en, es).
//! Les clés manquantes retombent sur le français.

pub fn tr(lang: &str, key: &str) -> &'static str {
    let l = match lang {
        "en" => 1,
        "es" => 2,
        _ => 0,
    };
    let row: [&'static str; 3] = match key {
        "report_title" => ["RAPPORT DE DÉCHARGEMENT", "OFFLOAD REPORT", "INFORME DE VOLCADO"],
        "log_title" => ["rapport de déchargement", "offload report", "informe de volcado"],
        "verified" => ["VÉRIFIÉ", "VERIFIED", "VERIFICADO"],
        "errors" => ["ERREURS", "ERRORS", "ERRORES"],
        "cancelled" => ["ANNULÉ", "CANCELLED", "CANCELADO"],
        "status" => ["Statut", "Status", "Estado"],
        "folder" => ["Dossier", "Folder", "Carpeta"],
        "project" => ["Projet", "Project", "Proyecto"],
        "day" => ["Jour de tournage", "Shoot day", "Día de rodaje"],
        "camera" => ["Caméra", "Camera", "Cámara"],
        "card" => ["Carte", "Card", "Tarjeta"],
        "operator" => ["Opérateur", "Operator", "Operador"],
        "notes" => ["Notes", "Notes", "Notas"],
        "source" => ["Source", "Source", "Origen"],
        "destination" => ["Destination", "Destination", "Destino"],
        "algorithm" => ["Algorithme", "Algorithm", "Algoritmo"],
        "start" => ["Début", "Start", "Inicio"],
        "end" => ["Fin", "End", "Fin"],
        "duration" => ["Durée", "Duration", "Duración"],
        "volume" => ["Volume", "Volume", "Volumen"],
        "clips" => ["Clips", "Clips", "Clips"],
        "verification" => ["Vérification", "Verification", "Verificación"],
        "result" => ["Résultat", "Result", "Resultado"],
        "machine" => ["Machine", "Machine", "Equipo"],
        "files" => ["fichiers", "files", "archivos"],
        "files_col" => ["FICHIERS", "FILES", "ARCHIVOS"],
        "file_col" => ["FICHIER", "FILE", "ARCHIVO"],
        "bytes_col" => ["OCTETS", "BYTES", "BYTES"],
        "clip_col" => ["CLIP", "CLIP", "CLIP"],
        "type_col" => ["TYPE", "TYPE", "TIPO"],
        "size_col" => ["TAILLE", "SIZE", "TAMAÑO"],
        "status_col" => ["STATUT", "STATUS", "ESTADO"],
        "kind_folder" => ["Dossier", "Folder", "Carpeta"],
        "kind_file" => ["Fichier", "File", "Archivo"],
        "ok" => ["OK", "OK", "OK"],
        "error" => ["ERREUR", "ERROR", "ERROR"],
        "different" => ["DIFFÉRENT", "MISMATCH", "DIFERENTE"],
        "ok_err" => ["{ok} OK, {err} en erreur", "{ok} OK, {err} failed", "{ok} OK, {err} con error"],
        "ok_err_short" => ["{ok} OK / {err} erreur(s)", "{ok} OK / {err} error(s)", "{ok} OK / {err} error(es)"],
        "files_in_error" => ["FICHIERS EN ERREUR", "FAILED FILES", "ARCHIVOS CON ERROR"],
        "no_clips" => [
            "Aucun clip vidéo reconnu : voir le log pour la liste des fichiers.",
            "No video clip recognised: see the log for the file list.",
            "Ningún clip de vídeo reconocido: ver el log para la lista de archivos.",
        ],
        "not_processed" => [
            "{n} fichier(s) non traités (annulation)",
            "{n} file(s) not processed (cancelled)",
            "{n} archivo(s) sin procesar (cancelado)",
        ],
        "footer" => [
            "MHL et log complets dans le dossier de destination",
            "Full MHL and log in the destination folder",
            "MHL y log completos en la carpeta de destino",
        ],
        "disclaimer_short" => [
            "Logiciel fourni « en l'état », sans garantie. Vérifiez vos copies avant de formater un support.",
            "Software provided \"as is\", without warranty. Check your copies before formatting any media.",
            "Software proporcionado \"tal cual\", sin garantía. Compruebe sus copias antes de formatear un soporte.",
        ],
        "page" => ["Page", "Page", "Página"],
        // messages du moteur
        "msg_summary" => [
            "{card} : {n} fichiers, {size} ({brand}), {d} destination(s)",
            "{card}: {n} files, {size} ({brand}), {d} destination(s)",
            "{card}: {n} archivos, {size} ({brand}), {d} destino(s)",
        ],
        "msg_done" => [
            "Terminé : {n} fichiers vérifiés en {s} s ({r} Mo/s)",
            "Done: {n} files verified in {s} s ({r} MB/s)",
            "Terminado: {n} archivos verificados en {s} s ({r} MB/s)",
        ],
        "msg_cancelled" => ["Déchargement annulé", "Offload cancelled", "Volcado cancelado"],
        "msg_done_errors" => [
            "Terminé avec {n} erreur(s)",
            "Finished with {n} error(s)",
            "Terminado con {n} error(es)",
        ],
        "msg_fail" => ["ÉCHEC", "FAILED", "FALLO"],
        "err_no_dest" => ["Aucune destination choisie", "No destination selected", "Ningún destino seleccionado"],
        "err_src" => ["Source introuvable : {p}", "Source not found: {p}", "Origen no encontrado: {p}"],
        "err_dest" => ["Destination introuvable : {p}", "Destination not found: {p}", "Destino no encontrado: {p}"],
        "err_dest_in_src" => [
            "La destination est à l'intérieur de la source : {p}",
            "The destination is inside the source: {p}",
            "El destino está dentro del origen: {p}",
        ],
        "err_space" => [
            "Espace insuffisant sur {p} : {free} libres pour {need} à copier",
            "Not enough space on {p}: {free} free for {need} to copy",
            "Espacio insuficiente en {p}: {free} libres para {need} a copiar",
        ],
        "err_create" => ["Création de {p} impossible : {e}", "Cannot create {p}: {e}", "No se puede crear {p}: {e}"],
        "err_report" => ["Rapport non écrit dans {p} : {e}", "Report not written in {p}: {e}", "Informe no escrito en {p}: {e}"],
        "err_hash" => [
            "hash différent sur au moins une destination",
            "hash mismatch on at least one destination",
            "hash diferente en al menos un destino",
        ],
        // contrôles de fiabilité (V0.2)
        "checks" => ["Contrôles", "Checks", "Controles"],
        "chk_uncached" => [
            "copies relues sur le disque (cache système contourné)",
            "copies re-read from disk (system cache bypassed)",
            "copias releídas del disco (caché del sistema evitada)",
        ],
        "chk_cached" => [
            "copies relues via le cache système (contournement impossible sur ce volume)",
            "copies re-read through the system cache (bypass unavailable on this volume)",
            "copias releídas a través de la caché del sistema (no se pudo evitar en este volumen)",
        ],
        "chk_reread" => ["carte lue deux fois", "card read twice", "tarjeta leída dos veces"],
        "chk_single" => ["carte lue une fois", "card read once", "tarjeta leída una vez"],
        "chk_watch" => [
            "source surveillée pendant la copie",
            "source monitored during the copy",
            "origen vigilado durante la copia",
        ],
        "src_changes" => [
            "SOURCE MODIFIÉE PENDANT LE DÉCHARGEMENT",
            "SOURCE CHANGED DURING THE OFFLOAD",
            "ORIGEN MODIFICADO DURANTE EL VOLCADO",
        ],
        "chg_added" => [
            "fichier apparu sur la carte pendant la copie, non copié : {p}",
            "file appeared on the card during the copy, not copied: {p}",
            "archivo aparecido en la tarjeta durante la copia, no copiado: {p}",
        ],
        "chg_removed" => [
            "fichier disparu de la carte après sa copie : {p}",
            "file disappeared from the card after its copy: {p}",
            "archivo desaparecido de la tarjeta tras su copia: {p}",
        ],
        "chg_modified" => [
            "fichier modifié sur la carte après sa copie : {p}",
            "file modified on the card after its copy: {p}",
            "archivo modificado en la tarjeta tras su copia: {p}",
        ],
        "chg_unreadable" => [
            "carte illisible en fin de déchargement : {e}",
            "card unreadable at the end of the offload: {e}",
            "tarjeta ilegible al final del volcado: {e}",
        ],
        "err_src_changed" => [
            "la source a changé pendant la copie (taille ou date)",
            "the source changed during the copy (size or date)",
            "el origen cambió durante la copia (tamaño o fecha)",
        ],
        "err_src_unstable" => [
            "les deux lectures de la carte diffèrent : lecteur, câble ou carte défaillant",
            "the two reads of the card differ: faulty reader, cable or card",
            "las dos lecturas de la tarjeta difieren: lector, cable o tarjeta defectuoso",
        ],
        "err_src_reread" => [
            "seconde lecture de la carte impossible",
            "second read of the card failed",
            "segunda lectura de la tarjeta imposible",
        ],
        "err_short" => [
            "fichier plus court que prévu ({got} octets sur {exp})",
            "file shorter than expected ({got} of {exp} bytes)",
            "archivo más corto de lo previsto ({got} de {exp} bytes)",
        ],
        "err_long" => [
            "fichier plus long que prévu ({exp} octets attendus)",
            "file longer than expected ({exp} bytes expected)",
            "archivo más largo de lo previsto ({exp} bytes esperados)",
        ],
        "msg_src_changed" => [
            "La carte a changé pendant le déchargement ({n} changement(s)) : ne la formatez pas, relancez-la",
            "The card changed during the offload ({n} change(s)): do not format it, run it again",
            "La tarjeta cambió durante el volcado ({n} cambio(s)): no la formatee, vuelva a lanzarla",
        ],
        "msg_cached_warn" => [
            "Attention : sur au moins une destination, la relecture n'a pas pu contourner le cache système",
            "Warning: on at least one destination, the re-read could not bypass the system cache",
            "Atención: en al menos un destino, la relectura no pudo evitar la caché del sistema",
        ],
        _ => ["?", "?", "?"],
    };
    row[l]
}

/// Remplace des {clés} dans une chaîne traduite.
pub fn fill(s: &str, vars: &[(&str, String)]) -> String {
    let mut o = s.to_string();
    for (k, v) in vars {
        o = o.replace(&format!("{{{k}}}"), v);
    }
    o
}

/// Format de date selon la langue.
pub fn date_fmt(lang: &str) -> &'static str {
    match lang {
        "en" => "%Y-%m-%d %H:%M:%S",
        _ => "%d/%m/%Y %H:%M:%S",
    }
}
