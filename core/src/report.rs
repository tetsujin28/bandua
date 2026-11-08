//! Écriture des rapports dans chaque dossier de destination :
//! <dossier>.mhl, <dossier>_log.txt, <dossier>_rapport.pdf, <dossier>.json

use crate::pdf::{fit, Font, Pdf, H, W};
use crate::i18n::{date_fmt, fill, tr};
use crate::{hb, Summary, APP_AUTHOR, APP_NAME, APP_VERSION};
use chrono::{DateTime, Local, TimeZone};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

pub fn write_all(s: &Summary, dir: &Path, mtimes: &[Option<filetime::FileTime>]) -> Result<Vec<PathBuf>, String> {
    let base = &s.folder_name;
    let mut out = vec![];
    let w = |name: String, data: Vec<u8>, out: &mut Vec<PathBuf>| -> Result<(), String> {
        let p = dir.join(name);
        fs::write(&p, data).map_err(|e| format!("{} : {e}", p.display()))?;
        out.push(p);
        Ok(())
    };
    let pdf_suffix = match s.job.lang.as_str() {
        "en" => "report",
        "es" => "informe",
        _ => "rapport",
    };
    w(format!("{base}_{pdf_suffix}.pdf"), pdf_report(s), &mut out)?;
    w(format!("{base}_log.txt"), log_text(s).into_bytes(), &mut out)?;
    w(format!("{base}.mhl"), mhl(s, mtimes).into_bytes(), &mut out)?;
    w(
        format!("{base}.json"),
        serde_json::to_vec_pretty(s).map_err(|e| e.to_string())?,
        &mut out,
    )?;
    Ok(out)
}

fn user_host() -> (String, String) {
    let user = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_default();
    let host = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_default();
    (user, host)
}

fn duration(secs: f64) -> String {
    let s = secs.round() as u64;
    if s >= 3600 {
        format!("{} h {:02} min {:02} s", s / 3600, (s % 3600) / 60, s % 60)
    } else if s >= 60 {
        format!("{} min {:02} s", s / 60, s % 60)
    } else {
        format!("{s} s")
    }
}

/// Contrôles effectués, en une ligne : « copies relues sur le disque · carte lue deux fois · … »
fn checks_line(s: &Summary) -> String {
    let l = s.job.lang.as_str();
    [
        tr(l, if s.uncached { "chk_uncached" } else { "chk_cached" }),
        tr(l, if s.job.source_reread { "chk_reread" } else { "chk_single" }),
        tr(l, "chk_watch"),
    ]
    .join(" · ")
}

fn status_word(s: &Summary) -> &'static str {
    let l = s.job.lang.as_str();
    tr(l, if s.cancelled { "cancelled" } else if s.success() { "verified" } else { "errors" })
}

// ---------------------------------------------------------------- LOG

fn log_text(s: &Summary) -> String {
    let (user, host) = user_host();
    let l = s.job.lang.as_str();
    let j = &s.job;
    let df = date_fmt(l);
    let mut t = String::new();
    let _ = writeln!(t, "{APP_NAME} {APP_VERSION} - {} - by {APP_AUTHOR}", tr(l, "log_title"));
    let _ = writeln!(t, "{}", "=".repeat(72));
    let row = |t: &mut String, k: &str, v: String| {
        let _ = writeln!(t, "{:<18}: {v}", k);
    };
    row(&mut t, tr(l, "status"), status_word(s).into());
    row(&mut t, tr(l, "folder"), s.folder_name.clone());
    row(&mut t, tr(l, "project"), j.project.clone());
    row(&mut t, tr(l, "day"), j.day.clone());
    row(&mut t, tr(l, "camera"), format!("{} ({})", j.camera, s.brand));
    row(&mut t, tr(l, "card"), j.card.clone());
    row(&mut t, tr(l, "operator"), j.operator.clone());
    row(&mut t, tr(l, "source"), j.source.display().to_string());
    for (i, d) in s.dest_dirs.iter().enumerate() {
        row(&mut t, &format!("{} {}", tr(l, "destination"), i + 1), d.display().to_string());
    }
    row(&mut t, tr(l, "algorithm"), j.algo.label().into());
    row(&mut t, tr(l, "checks"), checks_line(s));
    row(&mut t, tr(l, "start"), s.started.format(df).to_string());
    row(&mut t, tr(l, "end"), s.finished.format(df).to_string());
    row(&mut t, tr(l, "duration"), format!("{} ({:.0} MB/s)", duration(s.duration_s), s.avg_mb_s));
    row(
        &mut t,
        tr(l, "volume"),
        format!("{}, {} {}, {} clips", hb(l, s.total_bytes), s.total_files, tr(l, "files"), s.clips.len()),
    );
    row(
        &mut t,
        tr(l, "result"),
        fill(tr(l, "ok_err"), &[("ok", s.files_ok.to_string()), ("err", s.files_failed.to_string())]),
    );
    if !user.is_empty() || !host.is_empty() {
        row(&mut t, tr(l, "machine"), format!("{user}@{host}"));
    }
    if !j.notes.trim().is_empty() {
        row(&mut t, tr(l, "notes"), j.notes.trim().into());
    }
    let _ = writeln!(t, "{}", "-".repeat(72));
    let _ = writeln!(
        t,
        "{:<7} {:<16}  {:>15}  {}",
        tr(l, "status_col"),
        j.algo.label().to_uppercase(),
        tr(l, "bytes_col"),
        tr(l, "file_col")
    );
    for f in &s.files {
        let _ = writeln!(
            t,
            "{:<7} {:<16}  {:>15}  {}{}",
            if f.ok { tr(l, "ok") } else { tr(l, "error") },
            f.source_hash,
            f.size,
            f.rel,
            f.error.as_ref().map(|e| format!("  <- {e}")).unwrap_or_default()
        );
        if !f.ok {
            for (i, d) in f.dests.iter().enumerate() {
                let _ = writeln!(t, "          dest {}: {} {}", i + 1, d.hash, if d.ok { "ok" } else { tr(l, "different") });
            }
        }
    }
    if !s.source_changes.is_empty() {
        let _ = writeln!(t, "{}", "-".repeat(72));
        let _ = writeln!(t, "{} ({})", tr(l, "src_changes"), s.source_changes.len());
        for c in &s.source_changes {
            let _ = writeln!(t, "  {c}");
        }
    }
    let skipped = s.total_files.saturating_sub(s.files.len());
    if skipped > 0 {
        let _ = writeln!(t, "{}", fill(tr(l, "not_processed"), &[("n", skipped.to_string())]));
    }
    let _ = writeln!(t, "{}", "-".repeat(72));
    let _ = writeln!(t, "{}", tr(l, "disclaimer_short"));
    t
}

// ---------------------------------------------------------------- MHL

pub(crate) fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
pub(crate) fn xml_unescape(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&amp;", "&")
}

fn iso<Tz: TimeZone>(d: &DateTime<Tz>) -> String
where
    Tz::Offset: std::fmt::Display,
{
    d.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

/// MHL 1.1 (format « hashlist » lu par Silverstack, ShotPut, Hedge, YoYotta, mhl-tool…).
fn mhl(s: &Summary, mtimes: &[Option<filetime::FileTime>]) -> String {
    let (user, host) = user_host();
    let tag = s.job.algo.mhl_tag();
    let mut x = String::new();
    let _ = writeln!(x, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    let _ = writeln!(x, "<hashlist version=\"1.1\">");
    let _ = writeln!(x, "  <creatorinfo>");
    let _ = writeln!(x, "    <name>{}</name>", xml_escape(&s.job.operator));
    let _ = writeln!(x, "    <username>{}</username>", xml_escape(&user));
    let _ = writeln!(x, "    <hostname>{}</hostname>", xml_escape(&host));
    let _ = writeln!(x, "    <tool>{APP_NAME} {APP_VERSION} ({APP_AUTHOR})</tool>");
    let _ = writeln!(x, "    <startdate>{}</startdate>", iso(&s.started));
    let _ = writeln!(x, "    <finishdate>{}</finishdate>", iso(&s.finished));
    let _ = writeln!(x, "  </creatorinfo>");
    for (i, f) in s.files.iter().enumerate().filter(|(_, f)| f.ok) {
        let mdate = mtimes
            .get(i)
            .copied()
            .flatten()
            .and_then(|t| Local.timestamp_opt(t.unix_seconds(), 0).single())
            .map(|d| iso(&d))
            .unwrap_or_default();
        let _ = writeln!(x, "  <hash>");
        let _ = writeln!(x, "    <file>{}</file>", xml_escape(&f.rel));
        let _ = writeln!(x, "    <size>{}</size>", f.size);
        let _ = writeln!(x, "    <lastmodificationdate>{mdate}</lastmodificationdate>");
        let _ = writeln!(x, "    <{tag}>{}</{tag}>", f.source_hash);
        let _ = writeln!(x, "    <hashdate>{}</hashdate>", iso(&s.finished));
        let _ = writeln!(x, "  </hash>");
    }
    let _ = writeln!(x, "</hashlist>");
    x
}

// ---------------------------------------------------------------- PDF

const INK: (f32, f32, f32) = (0.10, 0.11, 0.12);
const MUTED: (f32, f32, f32) = (0.45, 0.47, 0.50);
const RULE: (f32, f32, f32) = (0.86, 0.87, 0.88);
const BAND: (f32, f32, f32) = (0.07, 0.075, 0.085);
const ACCENT: (f32, f32, f32) = (0.91, 0.64, 0.24);
const OK: (f32, f32, f32) = (0.13, 0.62, 0.40);
const BAD: (f32, f32, f32) = (0.85, 0.24, 0.22);
const WHITE: (f32, f32, f32) = (1.0, 1.0, 1.0);

fn pdf_report(s: &Summary) -> Vec<u8> {
    let mut p = Pdf::new();
    let m = 40.0; // marge
    let j = &s.job;
    let l = j.lang.as_str();
    let ok = s.success();
    let up = |k: &str| tr(l, k).to_uppercase();

    // En-tête
    p.new_page();
    p.rect(0.0, 0.0, W, 92.0, BAND);
    p.rect(0.0, 92.0, W, 3.0, ACCENT);
    p.text(m, 38.0, 9.0, Font::Bold, ACCENT, &format!("{} · {}", APP_NAME.to_uppercase(), tr(l, "report_title")));
    p.text(m, 66.0, 18.0, Font::Bold, WHITE, &fit(&s.folder_name, 380.0, 18.0, Font::Bold));
    let st = status_word(s);
    let col = if ok { OK } else { BAD };
    let bw = 104.0;
    p.rect(W - m - bw, 46.0, bw, 26.0, col);
    p.text(W - m - bw + (bw - st.chars().count() as f32 * 7.0) / 2.0, 63.5, 11.0, Font::Bold, WHITE, st);

    // Bloc d'informations sur deux colonnes
    let mut y = 124.0;
    let df = date_fmt(l);
    let left: Vec<(String, String)> = vec![
        (up("project"), j.project.clone()),
        (up("day"), j.day.clone()),
        (up("camera"), format!("{} ({})", j.camera, s.brand)),
        (up("card"), j.card.clone()),
        (up("operator"), j.operator.clone()),
    ];
    let right: Vec<(String, String)> = vec![
        (up("start"), s.started.format(df).to_string()),
        (up("duration"), format!("{}  ·  {:.0} MB/s", duration(s.duration_s), s.avg_mb_s)),
        (up("volume"), format!("{}  ·  {} {}", hb(l, s.total_bytes), s.total_files, tr(l, "files"))),
        (up("clips"), format!("{}", s.clips.len())),
        (
            up("verification"),
            format!(
                "{}  ·  {}",
                j.algo.label(),
                fill(tr(l, "ok_err_short"), &[("ok", s.files_ok.to_string()), ("err", s.files_failed.to_string())])
            ),
        ),
    ];
    for (i, ((lk, lv), (rk, rv))) in left.iter().zip(right.iter()).enumerate() {
        let yy = y + i as f32 * 17.0;
        p.text(m, yy, 7.5, Font::Bold, MUTED, lk);
        p.text(m + 95.0, yy, 9.5, Font::Regular, INK, &fit(if lv.is_empty() { "-" } else { lv }, 160.0, 9.5, Font::Regular));
        p.text(310.0, yy, 7.5, Font::Bold, MUTED, rk);
        p.text(385.0, yy, 9.5, Font::Regular, INK, &fit(rv, 170.0, 9.5, Font::Regular));
    }
    y += 5.0 * 17.0 + 6.0;
    p.text(m, y, 7.5, Font::Bold, MUTED, &up("source"));
    p.text(m + 95.0, y, 9.0, Font::Mono, INK, &fit(&j.source.display().to_string(), 420.0, 9.0, Font::Mono));
    for (i, d) in s.dest_dirs.iter().enumerate() {
        y += 15.0;
        p.text(m, y, 7.5, Font::Bold, MUTED, &format!("{} {}", up("destination"), i + 1));
        p.text(m + 95.0, y, 9.0, Font::Mono, INK, &fit(&d.display().to_string(), 420.0, 9.0, Font::Mono));
    }
    y += 15.0;
    p.text(m, y, 7.5, Font::Bold, MUTED, &up("checks"));
    for (i, line) in wrap(&checks_line(s), 88).iter().enumerate() {
        if i > 0 {
            y += 11.0;
        }
        p.text(m + 95.0, y, 8.5, Font::Regular, if s.uncached { INK } else { BAD }, line);
    }
    if !j.notes.trim().is_empty() {
        y += 15.0;
        p.text(m, y, 7.5, Font::Bold, MUTED, &up("notes"));
        for line in wrap(j.notes.trim(), 88) {
            p.text(m + 95.0, y, 9.0, Font::Regular, INK, &line);
            y += 12.0;
        }
        y -= 12.0;
    }

    // Tableau des clips
    y += 30.0;
    let cols = [m, m + 26.0, m + 300.0, m + 352.0, m + 410.0, m + 470.0];
    let header = |p: &mut Pdf, y: f32| {
        p.text(cols[0], y, 7.5, Font::Bold, MUTED, "#");
        p.text(cols[1], y, 7.5, Font::Bold, MUTED, tr(l, "clip_col"));
        p.text(cols[2], y, 7.5, Font::Bold, MUTED, tr(l, "type_col"));
        p.text(cols[3], y, 7.5, Font::Bold, MUTED, tr(l, "files_col"));
        p.text(cols[4], y, 7.5, Font::Bold, MUTED, tr(l, "size_col"));
        p.text(cols[5], y, 7.5, Font::Bold, MUTED, tr(l, "status_col"));
        p.line(m, y + 6.0, W - m, y + 6.0, INK, 0.8);
    };
    header(&mut p, y);
    y += 20.0;
    let mut page = 1;
    for (i, c) in s.clips.iter().enumerate() {
        let row_h = if c.hash.is_some() { 24.0 } else { 17.0 };
        if y + row_h > H - 60.0 {
            footer(&mut p, page, s);
            p.new_page();
            page += 1;
            y = 50.0;
            header(&mut p, y);
            y += 20.0;
        }
        p.text(cols[0], y, 8.5, Font::Regular, MUTED, &format!("{}", i + 1));
        p.text(cols[1], y, 9.5, Font::Bold, INK, &fit(&c.name, 265.0, 9.5, Font::Bold));
        if let Some(h) = &c.hash {
            p.text(cols[1], y + 10.0, 7.0, Font::Mono, MUTED, h);
        }
        p.text(cols[2], y, 8.5, Font::Regular, INK, tr(l, if c.is_folder { "kind_folder" } else { "kind_file" }));
        p.text(cols[3], y, 8.5, Font::Regular, INK, &format!("{}", c.file_count));
        p.text(cols[4], y, 8.5, Font::Regular, INK, &hb(l, c.bytes));
        p.text(cols[5], y, 8.5, Font::Bold, if c.ok { OK } else { BAD }, tr(l, if c.ok { "ok" } else { "error" }));
        p.line(m, y + row_h - 11.0, W - m, y + row_h - 11.0, RULE, 0.5);
        y += row_h;
    }
    if s.clips.is_empty() {
        p.text(m, y, 9.0, Font::Regular, MUTED, tr(l, "no_clips"));
    }
    let bad: Vec<_> = s.files.iter().filter(|f| !f.ok).collect();
    if !bad.is_empty() {
        y += 16.0;
        p.text(m, y, 9.0, Font::Bold, BAD, &format!("{} ({})", tr(l, "files_in_error"), bad.len()));
        y += 14.0;
        for f in bad {
            if y > H - 70.0 {
                footer(&mut p, page, s);
                p.new_page();
                page += 1;
                y = 50.0;
            }
            p.text(m, y, 8.0, Font::Mono, INK, &fit(&f.rel, 330.0, 8.0, Font::Mono));
            p.text(m + 340.0, y, 8.0, Font::Regular, BAD, &fit(f.error.as_deref().unwrap_or(""), 175.0, 8.0, Font::Regular));
            y += 12.0;
        }
    }
    if !s.source_changes.is_empty() {
        y += 16.0;
        if y > H - 90.0 {
            footer(&mut p, page, s);
            p.new_page();
            page += 1;
            y = 50.0;
        }
        p.text(m, y, 9.0, Font::Bold, BAD, &format!("{} ({})", tr(l, "src_changes"), s.source_changes.len()));
        y += 14.0;
        for c in &s.source_changes {
            if y > H - 70.0 {
                footer(&mut p, page, s);
                p.new_page();
                page += 1;
                y = 50.0;
            }
            p.text(m, y, 8.0, Font::Regular, INK, &fit(c, W - 2.0 * m, 8.0, Font::Regular));
            y += 12.0;
        }
    }
    footer(&mut p, page, s);
    p.render()
}

fn footer(p: &mut Pdf, page: usize, s: &Summary) {
    let l = s.job.lang.as_str();
    p.line(40.0, H - 46.0, W - 40.0, H - 46.0, RULE, 0.5);
    p.text(40.0, H - 34.0, 7.5, Font::Bold, MUTED, &format!(
        "{APP_NAME} {APP_VERSION}  ·  by {APP_AUTHOR}  ·  {}  ·  {}",
        s.finished.format("%d/%m/%Y %H:%M"),
        tr(l, "footer")
    ));
    p.text(40.0, H - 22.0, 6.8, Font::Regular, MUTED, tr(l, "disclaimer_short"));
    p.text(W - 75.0, H - 34.0, 7.5, Font::Regular, MUTED, &format!("{} {page}", tr(l, "page")));
}

fn wrap(s: &str, width: usize) -> Vec<String> {
    let mut lines = vec![];
    let mut cur = String::new();
    for w in s.split_whitespace() {
        if cur.chars().count() + w.chars().count() + 1 > width && !cur.is_empty() {
            lines.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(w);
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}
