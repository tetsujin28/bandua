//! bandua-core : copie d'une carte vers N destinations en une seule lecture,
//! hash à la volée, relecture de chaque copie pour vérification,
//! puis log texte, MHL, JSON et rapport PDF dans chaque destination.

pub mod clips;
pub mod hash;
pub mod i18n;
pub mod pdf;
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
}

fn default_lang() -> String {
    "fr".into()
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
}

impl Summary {
    pub fn success(&self) -> bool {
        !self.cancelled && self.files_failed == 0 && self.files_ok == self.total_files
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
    let work = info.total_bytes * (1 + n);
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
    let mut buf = vec![0u8; BUF];
    let mut th = Throttle { last: Instant::now() };
    let mut cancelled = false;

    for (i, f) in files.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            cancelled = true;
            break;
        }
        emit(Event::File { index: i, rel: f.rel.clone(), size: f.size });
        let r = copy_and_verify(job.algo, lg, f, &dest_dirs, &mut buf, cancel, &mut |phase, delta| {
            done += delta;
            if th.ready() {
                emit(Event::Progress { done, total: work, phase, file_index: i });
            }
        });
        // recaler le compteur si le fichier a échoué en cours de route
        let expected: u64 = results.iter().map(|r| r.size).sum::<u64>() * (1 + n) + f.size * (1 + n);
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
        files: results,
        reports: vec![],
    };

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

fn copy_and_verify(
    algo: Algo,
    lg: &str,
    f: &FileEntry,
    dest_dirs: &[PathBuf],
    buf: &mut [u8],
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(Phase, u64),
) -> Result<FileResult, String> {
    let finals: Vec<PathBuf> = dest_dirs.iter().map(|d| d.join(&f.rel)).collect();
    let parts: Vec<PathBuf> = finals.iter().map(|p| part_path(p)).collect();

    // 1. Copie : une lecture source, écriture vers toutes les destinations
    let mut src = File::open(&f.abs).map_err(|e| format!("ouverture source : {e}"))?;
    let mut outs: Vec<File> = vec![];
    for p in &parts {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("dossier {} : {e}", parent.display()))?;
        }
        outs.push(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(p)
                .map_err(|e| format!("création {} : {e}", p.display()))?,
        );
    }
    let mut h = hash::Hasher::new(algo);
    let mut read_total = 0u64;
    loop {
        if cancel.load(Ordering::Relaxed) {
            cleanup(&parts);
            return Err("annulé".into());
        }
        let n = src.read(buf).map_err(|e| format!("lecture source : {e}"))?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
        for (o, p) in outs.iter_mut().zip(&parts) {
            o.write_all(&buf[..n]).map_err(|e| format!("écriture {} : {e}", p.display()))?;
        }
        read_total += n as u64;
        progress(Phase::Copy, n as u64);
    }
    if read_total != f.size {
        cleanup(&parts);
        return Err(format!("taille lue {read_total} ≠ taille attendue {}", f.size));
    }
    for o in &outs {
        o.sync_all().map_err(|e| format!("synchronisation disque : {e}"))?;
    }
    drop(outs);
    let source_hash = h.finish_hex();

    // 2. Vérification : relecture de chaque copie depuis le disque
    let mut dests = vec![];
    for p in &parts {
        let mut rf = File::open(p).map_err(|e| format!("relecture {} : {e}", p.display()))?;
        let mut hh = hash::Hasher::new(algo);
        loop {
            if cancel.load(Ordering::Relaxed) {
                cleanup(&parts);
                return Err("annulé".into());
            }
            let n = rf.read(buf).map_err(|e| format!("relecture : {e}"))?;
            if n == 0 {
                break;
            }
            hh.update(&buf[..n]);
            progress(Phase::Verify, n as u64);
        }
        let hx = hh.finish_hex();
        dests.push(DestCheck { ok: hx == source_hash, hash: hx });
    }

    // 3. Finalisation : renommage + date de modification d'origine
    let all_ok = dests.iter().all(|d| d.ok);
    let mut error = None;
    for ((p, fin), d) in parts.iter().zip(&finals).zip(&dests) {
        if d.ok {
            if let Err(e) = fs::rename(p, fin) {
                error = Some(format!("renommage : {e}"));
            } else if let Some(mt) = f.mtime {
                let _ = filetime::set_file_mtime(fin, mt);
            }
        } else {
            let mut bad = fin.as_os_str().to_owned();
            bad.push(".HASH_ERREUR");
            let _ = fs::rename(p, PathBuf::from(bad));
            error = Some(tr(lg, "err_hash").into());
        }
    }
    Ok(FileResult { rel: f.rel.clone(), size: f.size, source_hash, dests, ok: all_ok && error.is_none(), error })
}

fn cleanup(parts: &[PathBuf]) {
    for p in parts {
        let _ = fs::remove_file(p);
    }
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
        match File::open(&p) {
            Ok(mut fh) => {
                let mut h = hash::Hasher::new(algo);
                let mut buf = vec![0u8; BUF];
                loop {
                    let n = fh.read(&mut buf).map_err(|e| e.to_string())?;
                    if n == 0 {
                        break;
                    }
                    h.update(&buf[..n]);
                }
                if h.finish_hex() != expected.to_ascii_lowercase() {
                    errors.push(format!("hash différent : {file}"));
                }
            }
            Err(_) => errors.push(format!("manquant : {file}")),
        }
        checked += 1;
    }
    Ok((checked, errors))
}
