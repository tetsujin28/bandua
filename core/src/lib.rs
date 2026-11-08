//! bandua-core : copie d'une carte vers N destinations en une seule lecture,
//! hash à la volée, relecture de chaque copie pour vérification,
//! puis log texte, MHL, JSON et rapport PDF dans chaque destination.

pub mod clips;
pub mod hash;
pub mod i18n;
pub mod pdf;
pub mod rawio;
mod report;

pub use clips::{Brand, Clip};
pub use hash::Algo;
use i18n::{fill, tr};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use walkdir::WalkDir;

pub const APP_NAME: &str = "Bandua";
pub const APP_AUTHOR: &str = "Domingo Olmo Martin";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const BUF: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub source: PathBuf,
    pub destinations: Vec<PathBuf>,
    #[serde(default)]
    pub project: String,
    #[serde(default)]
    pub day: String,
    #[serde(default)]
    pub camera: String,
    #[serde(default)]
    pub card: String,
    #[serde(default)]
    pub operator: String,
    #[serde(default)]
    pub notes: String,
    pub algo: Algo,
    /// Langue des rapports : fr, en, es
    #[serde(default = "default_lang")]
    pub lang: String,
    /// Relire la carte une seconde fois pour repérer un lecteur ou un câble défaillant
    #[serde(default = "default_true")]
    pub source_reread: bool,
}

fn default_lang() -> String {
    "fr".into()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub rel: String,
    pub size: u64,
    #[serde(skip)]
    pub abs: PathBuf,
    #[serde(skip)]
    pub mtime: Option<filetime::FileTime>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanInfo {
    pub source: PathBuf,
    pub card_label: String,
    pub total_files: usize,
    pub total_bytes: u64,
    pub brand: Brand,
    pub brand_label: String,
    pub clips: Vec<Clip>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DestCheck {
    pub hash: String,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileResult {
    pub rel: String,
    pub size: u64,
    pub source_hash: String,
    pub dests: Vec<DestCheck>,
    pub ok: bool,
    pub error: Option<String>,
    /// Les copies ont été relues sur le disque, sans passer par le cache du système
    pub uncached: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClipStatus {
    pub name: String,
    pub rel_path: String,
    pub is_folder: bool,
    pub brand: String,
    pub file_count: usize,
    pub bytes: u64,
    pub ok: bool,
    /// Hash du fichier si le clip est un fichier unique
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub job: Job,
    pub folder_name: String,
    pub dest_dirs: Vec<PathBuf>,
    pub started: DateTime<Local>,
    pub finished: DateTime<Local>,
    pub duration_s: f64,
    pub total_files: usize,
    pub total_bytes: u64,
    pub files_ok: usize,
    pub files_failed: usize,
    pub cancelled: bool,
    pub avg_mb_s: f64,
    pub brand: String,
    pub clips: Vec<ClipStatus>,
    pub files: Vec<FileResult>,
    pub reports: Vec<PathBuf>,
    /// Toutes les relectures ont contourné le cache du système
    pub uncached: bool,
    /// Changements constatés sur la source pendant le déchargement
    /// (fichiers apparus, disparus ou modifiés après leur copie)
    pub source_changes: Vec<String>,
    /// Résultat global, repris tel quel par l'interface
    pub ok: bool,
}

impl Summary {
    pub fn success(&self) -> bool {
        !self.cancelled
            && self.files_failed == 0
            && self.files_ok == self.total_files
            && self.source_changes.is_empty()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Started { total_files: usize, total_bytes: u64, work_bytes: u64, folder_name: String },
    File { index: usize, rel: String, size: u64 },
    Progress { done: u64, total: u64, phase: Phase, file_index: usize },
    FileResult { index: usize, result: FileResult },
    Log { level: String, message: String },
    Finished { summary: Box<Summary> },
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Copy,
    Verify,
    /// Seconde lecture de la carte
    Source,
}

/// Fichiers système à ne jamais copier.
fn is_junk(name: &str) -> bool {
    matches!(
        name,
        ".DS_Store" | ".Spotlight-V100" | ".Trashes" | ".fseventsd" | ".TemporaryItems"
            | "System Volume Information" | "$RECYCLE.BIN" | "Thumbs.db" | "desktop.ini"
    ) || name.starts_with("._")
}

pub fn scan(source: &Path) -> Result<(Vec<FileEntry>, ScanInfo), String> {
    if !source.is_dir() {
        return Err(format!("Source introuvable : {}", source.display()));
    }
    let mut files = vec![];
    let walker = WalkDir::new(source)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !is_junk(&e.file_name().to_string_lossy()));
    for entry in walker {
        let entry = entry.map_err(|e| format!("Lecture impossible : {e}"))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let md = entry.metadata().map_err(|e| e.to_string())?;
        let rel = entry
            .path()
            .strip_prefix(source)
            .unwrap()
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        files.push(FileEntry {
            rel,
            size: md.len(),
            abs: entry.path().to_path_buf(),
            mtime: Some(filetime::FileTime::from_last_modification_time(&md)),
        });
    }
    let pairs: Vec<(String, u64)> = files.iter().map(|f| (f.rel.clone(), f.size)).collect();
    let clips = clips::group_clips(&pairs);
    let brand = clips::dominant_brand(&clips);
    let card_label = source
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| source.display().to_string());
    let info = ScanInfo {
        source: source.to_path_buf(),
        card_label,
        total_files: files.len(),
        total_bytes: files.iter().map(|f| f.size).sum(),
        brand,
        brand_label: brand.label().to_string(),
        clips,
    };
    Ok((files, info))
}

/// Espace libre sur le volume qui contient `path`.
pub fn free_space(path: &Path) -> Option<u64> {
    fs2::available_space(path).ok()
}

fn sanitize(s: &str) -> String {
    s.trim()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

/// Nom du dossier daté : 2026-11-06_1633_PROJET_J03_CAMA_A001
pub fn folder_name(job: &Job, at: DateTime<Local>) -> String {
    let mut parts = vec![at.format("%Y-%m-%d_%H%M").to_string()];
    for p in [&job.project, &job.day, &job.camera, &job.card] {
        let s = sanitize(p);
        if !s.is_empty() {
            parts.push(s);
        }
    }
    parts.join("_")
}

fn unique_dir(base: &Path, name: &str) -> PathBuf {
    let mut p = base.join(name);
    let mut n = 2;
    while p.exists() {
        p = base.join(format!("{name}_{n}"));
        n += 1;
    }
    p
}

struct Throttle {
    last: Instant,
}
impl Throttle {
    fn ready(&mut self) -> bool {
        if self.last.elapsed() >= Duration::from_millis(80) {
            self.last = Instant::now();
            true
        } else {
            false
        }
    }
}

/// Lance le déchargement complet. Bloquant : à appeler depuis un thread dédié.
pub fn run(job: &Job, cancel: &AtomicBool, emit: &mut dyn FnMut(Event)) -> Result<Summary, String> {
    if job.destinations.is_empty() {
        return Err(tr(&job.lang, "err_no_dest").into());
    }
    let lg = job.lang.as_str();
    let started = Local::now();
    let t0 = Instant::now();
    if !job.source.is_dir() {
        return Err(fill(tr(lg, "err_src"), &[("p", job.source.display().to_string())]));
    }
    let (files, info) = scan(&job.source)?;

    // Vérifications préalables
    for d in &job.destinations {
        if !d.is_dir() {
            return Err(fill(tr(lg, "err_dest"), &[("p", d.display().to_string())]));
        }
        if d.starts_with(&job.source) {
            return Err(fill(tr(lg, "err_dest_in_src"), &[("p", d.display().to_string())]));
        }
        if let Some(free) = free_space(d) {
            if free < info.total_bytes {
                return Err(fill(
                    tr(lg, "err_space"),
                    &[
                        ("p", d.display().to_string()),
                        ("free", hb(lg, free)),
                        ("need", hb(lg, info.total_bytes)),
                    ],
                ));
            }
        }
    }

    let fname = folder_name(job, started);
    let dest_dirs: Vec<PathBuf> = job.destinations.iter().map(|d| unique_dir(d, &fname)).collect();
    for d in &dest_dirs {
        fs::create_dir_all(d)
            .map_err(|e| fill(tr(lg, "err_create"), &[("p", d.display().to_string()), ("e", e.to_string())]))?;
    }

    let n = dest_dirs.len() as u64;
    // passes par fichier : 1 lecture de la carte, n relectures, + 1 relecture de la carte en option
    let passes = 1 + n + job.source_reread as u64;
    let work = info.total_bytes * passes;
    emit(Event::Started {
        total_files: files.len(),
        total_bytes: info.total_bytes,
        work_bytes: work,
        folder_name: fname.clone(),
    });
    emit(Event::Log {
        level: "info".into(),
        message: fill(
            tr(lg, "msg_summary"),
            &[
                ("card", info.card_label.clone()),
                ("n", files.len().to_string()),
                ("size", hb(lg, info.total_bytes)),
                ("brand", info.brand_label.clone()),
                ("d", n.to_string()),
            ],
        ),
    });

    let mut done: u64 = 0;
    let mut results: Vec<FileResult> = Vec::with_capacity(files.len());
    let mut buf = rawio::AlignedBuf::new(BUF);
    let mut th = Throttle { last: Instant::now() };
    let mut cancelled = false;

    for (i, f) in files.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            cancelled = true;
            break;
        }
        emit(Event::File { index: i, rel: f.rel.clone(), size: f.size });
        let r = copy_and_verify(job.algo, lg, f, &dest_dirs, &mut buf, job.source_reread, cancel, &mut |phase, delta| {
            done += delta;
            if th.ready() {
                emit(Event::Progress { done, total: work, phase, file_index: i });
            }
        });
        // recaler le compteur si le fichier a échoué en cours de route
        let expected: u64 = results.iter().map(|r| r.size).sum::<u64>() * passes + f.size * passes;
        done = expected;
        emit(Event::Progress { done, total: work, phase: Phase::Verify, file_index: i });

        let res = match r {
            Ok(res) => res,
            Err(e) => FileResult {
                rel: f.rel.clone(),
                size: f.size,
                source_hash: String::new(),
                dests: vec![],
                ok: false,
                error: Some(e),
                uncached: false,
            },
        };
        if !res.ok {
            emit(Event::Log {
                level: "error".into(),
                message: format!("{} {} : {}", tr(lg, "msg_fail"), res.rel, res.error.clone().unwrap_or_default()),
            });
        }
        emit(Event::FileResult { index: i, result: res.clone() });
        results.push(res);
        if cancel.load(Ordering::Relaxed) {
            cancelled = true;
            break;
        }
    }

    // La carte a-t-elle changé pendant le déchargement ?
    let source_changes = if cancelled { vec![] } else { source_changes(lg, &job.source, &files, &results) };
    for c in &source_changes {
        emit(Event::Log { level: "error".into(), message: c.clone() });
    }

    // Statut par clip
    let clips: Vec<ClipStatus> = info
        .clips
        .iter()
        .map(|c| {
            let ok = c.files.iter().all(|&i| results.get(i).map(|r| r.ok).unwrap_or(false));
            let hash = if c.files.len() == 1 && !c.is_folder {
                results.get(c.files[0]).map(|r| r.source_hash.clone()).filter(|h| !h.is_empty())
            } else {
                None
            };
            ClipStatus {
                name: c.name.clone(),
                rel_path: c.rel_path.clone(),
                is_folder: c.is_folder,
                brand: c.brand.label().to_string(),
                file_count: c.files.len(),
                bytes: c.bytes,
                ok,
                hash,
            }
        })
        .collect();

    let finished = Local::now();
    let secs = t0.elapsed().as_secs_f64().max(0.001);
    let files_ok = results.iter().filter(|r| r.ok).count();
    let mut summary = Summary {
        job: job.clone(),
        folder_name: fname,
        dest_dirs: dest_dirs.clone(),
        started,
        finished,
        duration_s: secs,
        total_files: files.len(),
        total_bytes: info.total_bytes,
        files_ok,
        files_failed: results.len() - files_ok,
        cancelled,
        avg_mb_s: info.total_bytes as f64 / 1_000_000.0 / secs,
        brand: info.brand_label.clone(),
        clips,
        uncached: results.iter().any(|r| r.ok) && results.iter().filter(|r| r.ok).all(|r| r.uncached),
        files: results,
        reports: vec![],
        source_changes,
        ok: false,
    };
    summary.ok = summary.success();
    if summary.files_ok > 0 && !summary.uncached {
        emit(Event::Log { level: "warn".into(), message: tr(lg, "msg_cached_warn").into() });
    }

    let mtimes: Vec<Option<filetime::FileTime>> = files.iter().map(|f| f.mtime).collect();
    for d in &dest_dirs {
        match report::write_all(&summary, d, &mtimes) {
            Ok(mut paths) => summary.reports.append(&mut paths),
            Err(e) => emit(Event::Log {
                level: "error".into(),
                message: fill(tr(lg, "err_report"), &[("p", d.display().to_string()), ("e", e)]),
            }),
        }
    }
    emit(Event::Log {
        level: if summary.success() { "ok" } else { "error" }.into(),
        message: if summary.success() {
            fill(
                tr(lg, "msg_done"),
                &[
                    ("n", summary.files_ok.to_string()),
                    ("s", format!("{secs:.0}")),
                    ("r", format!("{:.0}", summary.avg_mb_s)),
                ],
            )
        } else if cancelled {
            tr(lg, "msg_cancelled").into()
        } else if summary.files_failed == 0 {
            fill(tr(lg, "msg_src_changed"), &[("n", summary.source_changes.len().to_string())])
        } else {
            fill(tr(lg, "msg_done_errors"), &[("n", summary.files_failed.to_string())])
        },
    });
    emit(Event::Finished { summary: Box::new(summary.clone()) });
    Ok(summary)
}

fn part_path(final_path: &Path) -> PathBuf {
    let mut s = final_path.as_os_str().to_owned();
    s.push(".bandua-part");
    PathBuf::from(s)
}

/// Taille et date de modification actuelles d'un fichier source.
fn source_sig(p: &Path) -> std::io::Result<(u64, filetime::FileTime)> {
    let md = fs::metadata(p)?;
    Ok((md.len(), filetime::FileTime::from_last_modification_time(&md)))
}

/// Le fichier source est-il resté tel qu'il était à l'analyse de la carte ?
fn source_unchanged(f: &FileEntry) -> bool {
    match source_sig(&f.abs) {
        Ok((size, mt)) => size == f.size && f.mtime.is_none_or(|m| m == mt),
        Err(_) => false,
    }
}

/// Message d'erreur lisible pour une lecture bornée.
fn read_err(lg: &str, what: &str, e: rawio::ReadError, expected: u64) -> String {
    match e {
        rawio::ReadError::Io(e) => format!("{what} : {e}"),
        rawio::ReadError::Short(got) => format!(
            "{what} : {}",
            fill(tr(lg, "err_short"), &[("got", got.to_string()), ("exp", expected.to_string())])
        ),
        rawio::ReadError::Long => format!("{what} : {}", fill(tr(lg, "err_long"), &[("exp", expected.to_string())])),
        rawio::ReadError::Abort(m) => m,
    }
}

#[cfg(test)]
thread_local! {
    /// Point d'injection des tests : appelé juste avant la seconde lecture de la carte.
    static BEFORE_REREAD: std::cell::Cell<Option<fn(&Path)>> = const { std::cell::Cell::new(None) };
}

#[allow(clippy::too_many_arguments)]
fn copy_and_verify(
    algo: Algo,
    lg: &str,
    f: &FileEntry,
    dest_dirs: &[PathBuf],
    buf: &mut [u8],
    reread: bool,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(Phase, u64),
) -> Result<FileResult, String> {
    let finals: Vec<PathBuf> = dest_dirs.iter().map(|d| d.join(&f.rel)).collect();
    let parts: Vec<PathBuf> = finals.iter().map(|p| part_path(p)).collect();
    // En cas d'échec avant la finalisation, on ne laisse aucun fichier partiel
    copy_and_verify_inner(algo, lg, f, &finals, &parts, buf, reread, cancel, progress).inspect_err(|_| cleanup(&parts))
}

#[allow(clippy::too_many_arguments)]
fn copy_and_verify_inner(
    algo: Algo,
    lg: &str,
    f: &FileEntry,
    finals: &[PathBuf],
    parts: &[PathBuf],
    buf: &mut [u8],
    reread: bool,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(Phase, u64),
) -> Result<FileResult, String> {
    let cancelled = || -> Result<(), String> {
        if cancel.load(Ordering::Relaxed) {
            Err("annulé".into())
        } else {
            Ok(())
        }
    };

    // 0. La source est-elle toujours celle qu'on a analysée ?
    if !source_unchanged(f) {
        return Err(tr(lg, "err_src_changed").into());
    }

    // 1. Copie : une lecture de la carte, écriture vers toutes les destinations
    let (mut src, _) = rawio::open_uncached(&f.abs).map_err(|e| format!("ouverture source : {e}"))?;
    let mut outs: Vec<File> = vec![];
    for p in parts {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("dossier {} : {e}", parent.display()))?;
        }
        let o = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(p)
            .map_err(|e| format!("création {} : {e}", p.display()))?;
        rawio::prepare_write(&o);
        outs.push(o);
    }
    let mut h = hash::Hasher::new(algo);
    rawio::read_sized(&mut src, f.size, buf, |data| {
        cancelled()?;
        h.update(data);
        for (o, p) in outs.iter_mut().zip(parts) {
            o.write_all(data).map_err(|e| format!("écriture {} : {e}", p.display()))?;
        }
        progress(Phase::Copy, data.len() as u64);
        Ok(())
    })
    .map_err(|e| read_err(lg, "lecture source", e, f.size))?;
    drop(src);
    // Écriture physique sur les disques (F_FULLFSYNC sous macOS), puis vidage du cache
    for o in &outs {
        o.sync_all().map_err(|e| format!("synchronisation disque : {e}"))?;
        rawio::drop_cache(o);
    }
    drop(outs);
    let source_hash = h.finish_hex();

    // 2. Vérification : relecture de chaque copie depuis le disque, sans cache
    let mut dests = vec![];
    let mut uncached = true;
    for p in parts {
        let (mut rf, unc) = rawio::open_uncached(p).map_err(|e| format!("relecture {} : {e}", p.display()))?;
        uncached &= unc;
        let mut hh = hash::Hasher::new(algo);
        rawio::read_sized(&mut rf, f.size, buf, |data| {
            cancelled()?;
            hh.update(data);
            progress(Phase::Verify, data.len() as u64);
            Ok(())
        })
        .map_err(|e| read_err(lg, &format!("relecture {}", p.display()), e, f.size))?;
        let hx = hh.finish_hex();
        dests.push(DestCheck { ok: hx == source_hash, hash: hx });
    }

    // 3. Seconde lecture de la carte : un lecteur ou un câble qui flanche
    //    ne renvoie pas deux fois les mêmes données
    let mut source_error: Option<String> = None;
    if reread {
        #[cfg(test)]
        if let Some(hook) = BEFORE_REREAD.with(|c| c.get()) {
            hook(&f.abs);
        }
        let mut h2 = hash::Hasher::new(algo);
        let r = rawio::open_uncached(&f.abs)
            .map_err(rawio::ReadError::Io)
            .and_then(|(mut s2, _)| {
                rawio::read_sized(&mut s2, f.size, buf, |data| {
                    cancelled()?;
                    h2.update(data);
                    progress(Phase::Source, data.len() as u64);
                    Ok(())
                })
            });
        match r {
            Ok(()) if h2.finish_hex() == source_hash => {}
            Ok(()) => source_error = Some(tr(lg, "err_src_unstable").into()),
            Err(rawio::ReadError::Abort(m)) => return Err(m),
            Err(rawio::ReadError::Io(e)) => source_error = Some(format!("{} : {e}", tr(lg, "err_src_reread"))),
            Err(_) => source_error = Some(tr(lg, "err_src_changed").into()),
        }
    }

    // 4. La source a-t-elle changé pendant ce temps ?
    if source_error.is_none() && !source_unchanged(f) {
        source_error = Some(tr(lg, "err_src_changed").into());
    }

    // 5. Finalisation : renommage + date de modification d'origine,
    //    ou marquage .HASH_ERREUR si quoi que ce soit est douteux
    let trusted = source_error.is_none() && dests.iter().all(|d| d.ok);
    let mut error = source_error;
    for ((p, fin), d) in parts.iter().zip(finals).zip(&dests) {
        if trusted {
            if let Err(e) = fs::rename(p, fin) {
                error = Some(format!("renommage : {e}"));
            } else if let Some(mt) = f.mtime {
                let _ = filetime::set_file_mtime(fin, mt);
            }
        } else {
            let mut bad = fin.as_os_str().to_owned();
            bad.push(".HASH_ERREUR");
            let _ = fs::rename(p, PathBuf::from(bad));
            if error.is_none() && !d.ok {
                error = Some(tr(lg, "err_hash").into());
            }
        }
    }
    Ok(FileResult {
        rel: f.rel.clone(),
        size: f.size,
        source_hash,
        dests,
        ok: trusted && error.is_none(),
        error,
        uncached,
    })
}

fn cleanup(parts: &[PathBuf]) {
    for p in parts {
        let _ = fs::remove_file(p);
    }
}

/// Compare la carte en fin de déchargement avec son état initial :
/// fichiers apparus (non copiés), disparus, ou modifiés après une copie réussie.
fn source_changes(lg: &str, source: &Path, before: &[FileEntry], results: &[FileResult]) -> Vec<String> {
    let after = match scan(source) {
        Ok((files, _)) => files,
        Err(e) => return vec![fill(tr(lg, "chg_unreadable"), &[("e", e)])],
    };
    let mut changes = vec![];
    let old: std::collections::HashMap<&str, (usize, &FileEntry)> =
        before.iter().enumerate().map(|(i, f)| (f.rel.as_str(), (i, f))).collect();
    let new: std::collections::HashMap<&str, &FileEntry> = after.iter().map(|f| (f.rel.as_str(), f)).collect();
    for f in &after {
        match old.get(f.rel.as_str()) {
            None => changes.push(fill(tr(lg, "chg_added"), &[("p", f.rel.clone())])),
            Some((i, o)) => {
                // un fichier déjà en erreur a déjà son propre message
                let copied_ok = results.get(*i).is_some_and(|r| r.ok);
                if copied_ok && (o.size != f.size || o.mtime != f.mtime) {
                    changes.push(fill(tr(lg, "chg_modified"), &[("p", f.rel.clone())]));
                }
            }
        }
    }
    for (i, f) in before.iter().enumerate() {
        if !new.contains_key(f.rel.as_str()) && results.get(i).is_some_and(|r| r.ok) {
            changes.push(fill(tr(lg, "chg_removed"), &[("p", f.rel.clone())]));
        }
    }
    changes
}

pub fn human_bytes(b: u64) -> String {
    hb("fr", b)
}

/// Taille lisible selon la langue (o/Ko/Mo en français, B/KB/MB sinon).
pub fn hb(lang: &str, b: u64) -> String {
    let units = if lang == "fr" { ["o", "Ko", "Mo", "Go", "To"] } else { ["B", "KB", "MB", "GB", "TB"] };
    let mut v = b as f64;
    let mut u = 0;
    while v >= 1000.0 && u < units.len() - 1 {
        v /= 1000.0;
        u += 1;
    }
    let txt = if u == 0 { format!("{b} {}", units[0]) } else { format!("{v:.2} {}", units[u]) };
    if lang == "en" { txt } else { txt.replace('.', ",") }
}

/// Re-vérifie un dossier existant à partir de son MHL (mode « re-vérification »).
pub fn verify_mhl(mhl: &Path, cancel: &AtomicBool) -> Result<(usize, Vec<String>), String> {
    let txt = fs::read_to_string(mhl).map_err(|e| e.to_string())?;
    let root = mhl.parent().unwrap_or(Path::new("."));
    let mut checked = 0;
    let mut errors = vec![];
    let mut buf = rawio::AlignedBuf::new(BUF);
    for block in txt.split("<hash>").skip(1) {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let tag = |t: &str| -> Option<String> {
            let a = block.find(&format!("<{t}>"))? + t.len() + 2;
            let b = block[a..].find(&format!("</{t}>"))? + a;
            Some(report::xml_unescape(&block[a..b]))
        };
        let Some(file) = tag("file") else { continue };
        let (algo, expected) = [Algo::Xxh64, Algo::Xxh3, Algo::Md5, Algo::Sha1]
            .into_iter()
            .find_map(|a| tag(a.mhl_tag()).map(|h| (a, h)))
            .ok_or_else(|| format!("aucun hash pour {file}"))?;
        let p = root.join(&file);
        let size = tag("size").and_then(|v| v.trim().parse::<u64>().ok());
        let mut h = hash::Hasher::new(algo);
        let res = match size {
            // Relecture sans cache, bornée à la taille notée dans le MHL
            Some(sz) => match rawio::open_uncached(&p) {
                Ok((mut fh, _)) => rawio::read_sized(&mut fh, sz, &mut buf, |d| {
                    h.update(d);
                    Ok(())
                })
                .map_err(|e| read_err("fr", "lecture", e, sz)),
                Err(_) => Err(format!("manquant : {file}")),
            },
            // Ancien MHL sans taille : lecture classique jusqu'à la fin
            None => match File::open(&p) {
                Ok(mut fh) => loop {
                    match fh.read(&mut buf) {
                        Ok(0) => break Ok(()),
                        Ok(n) => h.update(&buf[..n]),
                        Err(e) => break Err(e.to_string()),
                    }
                },
                Err(_) => Err(format!("manquant : {file}")),
            },
        };
        match res {
            Ok(()) => {
                if h.finish_hex() != expected.trim().to_ascii_lowercase() {
                    errors.push(format!("hash différent : {file}"));
                }
            }
            Err(e) if e.starts_with("manquant") => errors.push(e),
            Err(e) => errors.push(format!("{file} : {e}")),
        }
        checked += 1;
    }
    Ok((checked, errors))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../target/test-tmp")
            .join(format!("engine-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    /// Carte factice : un clip RED et un fichier isolé, tailles non alignées
    fn card(root: &Path) -> PathBuf {
        let src = root.join("A001_RED");
        let clip = src.join("A001_C001_1106AB.RDM/A001_C001_1106AB_001.R3D");
        fs::create_dir_all(clip.parent().unwrap()).unwrap();
        fs::write(&clip, (0..3_000_123u32).map(|i| (i % 251) as u8).collect::<Vec<_>>()).unwrap();
        fs::write(src.join("A001_C001_1106AB.RDM/A001_C001_1106AB.RTN"), b"meta").unwrap();
        fs::write(src.join("notes.txt"), b"").unwrap();
        src
    }

    fn job(src: &Path, dests: Vec<PathBuf>) -> Job {
        Job {
            source: src.to_path_buf(),
            destinations: dests,
            project: "TEST".into(),
            day: String::new(),
            camera: String::new(),
            card: String::new(),
            operator: String::new(),
            notes: String::new(),
            algo: Algo::Xxh64,
            lang: "fr".into(),
            source_reread: true,
        }
    }

    fn dests(root: &Path, n: usize) -> Vec<PathBuf> {
        (0..n)
            .map(|i| {
                let d = root.join(format!("dest{i}"));
                fs::create_dir_all(&d).unwrap();
                d
            })
            .collect()
    }

    fn leftovers(dir: &Path) -> Vec<String> {
        WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".bandua-part") || n.ends_with(".HASH_ERREUR"))
            .collect()
    }

    #[test]
    fn offload_two_destinations() {
        let root = tmp("ok");
        let src = card(&root);
        let d = dests(&root, 2);
        let mut work = 0;
        let mut done = 0;
        let s = run(&job(&src, d.clone()), &AtomicBool::new(false), &mut |e| match e {
            Event::Started { work_bytes, total_bytes, .. } => {
                assert_eq!(work_bytes, total_bytes * 4); // copie + 2 relectures + relecture carte
                work = work_bytes;
            }
            Event::Progress { done: dn, .. } => done = dn,
            _ => {}
        })
        .unwrap();
        assert!(s.success(), "{:?}", s.files.iter().filter(|f| !f.ok).collect::<Vec<_>>());
        assert!(s.ok && s.source_changes.is_empty());
        assert_eq!(done, work);
        #[cfg(target_os = "linux")]
        assert!(s.uncached);
        for dd in &s.dest_dirs {
            for f in &s.files {
                assert_eq!(fs::read(dd.join(&f.rel)).unwrap(), fs::read(src.join(&f.rel)).unwrap());
            }
            assert!(leftovers(dd).is_empty());
        }
        let mhl = s.reports.iter().find(|p| p.extension().is_some_and(|e| e == "mhl")).unwrap();
        let (n, errs) = verify_mhl(mhl, &AtomicBool::new(false)).unwrap();
        assert_eq!((n, errs.len()), (3, 0));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn single_read_option() {
        let root = tmp("single");
        let src = card(&root);
        let mut j = job(&src, dests(&root, 1));
        j.source_reread = false;
        let s = run(&j, &AtomicBool::new(false), &mut |e| {
            if let Event::Started { work_bytes, total_bytes, .. } = e {
                assert_eq!(work_bytes, total_bytes * 2);
            }
        })
        .unwrap();
        assert!(s.success());
        let _ = fs::remove_dir_all(&root);
    }

    /// Le lecteur renvoie d'autres données à la seconde lecture (même taille, même date)
    #[test]
    fn unstable_reader_is_detected() {
        fn flip(p: &Path) {
            if p.extension().is_some_and(|e| e == "R3D") {
                let mt = filetime::FileTime::from_last_modification_time(&fs::metadata(p).unwrap());
                let mut data = fs::read(p).unwrap();
                data[1_234_567] ^= 0x01;
                fs::write(p, data).unwrap();
                filetime::set_file_mtime(p, mt).unwrap();
            }
        }
        let root = tmp("unstable");
        let src = card(&root);
        BEFORE_REREAD.with(|c| c.set(Some(flip)));
        let s = run(&job(&src, dests(&root, 1)), &AtomicBool::new(false), &mut |_| {}).unwrap();
        BEFORE_REREAD.with(|c| c.set(None));
        assert!(!s.success());
        let bad: Vec<_> = s.files.iter().filter(|f| !f.ok).collect();
        assert_eq!(bad.len(), 1);
        assert_eq!(bad[0].error.as_deref(), Some(tr("fr", "err_src_unstable")));
        // la copie douteuse est marquée, pas présentée comme bonne
        assert_eq!(leftovers(&s.dest_dirs[0]).len(), 1);
        assert!(!s.dest_dirs[0].join(&bad[0].rel).exists());
        let _ = fs::remove_dir_all(&root);
    }

    /// La caméra (ou quelqu'un) écrit sur la carte pendant le déchargement
    #[test]
    fn source_changes_are_detected() {
        let root = tmp("changes");
        let src = card(&root);
        let src2 = src.clone();
        let s = run(&job(&src, dests(&root, 1)), &AtomicBool::new(false), &mut |e| {
            if let Event::File { index: 0, .. } = e {
                // un fichier pas encore copié grossit, un nouveau fichier apparaît
                fs::write(src2.join("notes.txt"), b"ajout").unwrap();
                fs::write(src2.join("A001_C002_1106XY.RDM.txt"), b"nouveau").unwrap();
            }
        })
        .unwrap();
        assert!(!s.success() && !s.ok);
        let bad: Vec<_> = s.files.iter().filter(|f| !f.ok).map(|f| f.rel.as_str()).collect();
        assert_eq!(bad, vec!["notes.txt"]);
        assert_eq!(s.source_changes.len(), 1);
        assert!(s.source_changes[0].contains("A001_C002_1106XY.RDM.txt"));
        assert!(leftovers(&s.dest_dirs[0]).is_empty());
        let _ = fs::remove_dir_all(&root);
    }

    /// Un fichier modifié sur la carte après avoir été copié avec succès
    #[test]
    fn change_after_copy_is_detected() {
        let root = tmp("after");
        let src = card(&root);
        let src2 = src.clone();
        let s = run(&job(&src, dests(&root, 1)), &AtomicBool::new(false), &mut |e| {
            if let Event::FileResult { index: 0, result } = e {
                fs::write(src2.join(&result.rel), b"ecrase").unwrap();
            }
        })
        .unwrap();
        assert_eq!(s.files_failed, 0);
        assert!(!s.success());
        assert_eq!(s.source_changes.len(), 1);
        let _ = fs::remove_dir_all(&root);
    }
}
