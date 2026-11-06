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
