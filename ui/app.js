// Bandua by Domingo Olmo Martin : interface.
// Fonctionne dans l'app Tauri (window.__TAURI__) ou en mode démo dans un navigateur.
"use strict";

let VERSION = "0.2.0"; // remplacé au démarrage par la version de l'application (tauri.conf.json)
const $ = (id) => document.getElementById(id);
const store = {
  get(k, d) { try { const v = localStorage.getItem("bandua." + k); return v == null ? d : JSON.parse(v); } catch { return d; } },
  set(k, v) { try { localStorage.setItem("bandua." + k, JSON.stringify(v)); } catch {} },
};

// ------------------------------------------------------------------ Langue

let LANG = store.get("lang", null) || ((navigator.language || "fr").slice(0, 2));
if (!I18N[LANG]) LANG = "fr";
const t = (k, vars) => {
  let s = (I18N[LANG] && I18N[LANG][k]) ?? I18N.fr[k] ?? k;
  if (vars) for (const [a, b] of Object.entries(vars)) s = s.split("{" + a + "}").join(b);
  return s;
};
const num = (x) => String(x).replace(".", t("dec"));
const fmtBytes = (b) => {
  if (b == null) return "-";
  const u = t("units"); let i = 0, v = b;
  while (v >= 1000 && i < u.length - 1) { v /= 1000; i++; }
  return num(i ? v.toFixed(v >= 100 ? 0 : v >= 10 ? 1 : 2) : v) + " " + u[i];
};
const fmtTime = (s) => {
  if (!isFinite(s) || s < 0) return "-";
  s = Math.round(s);
  const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), r = s % 60;
  return h ? `${h}h${String(m).padStart(2, "0")}` : `${m}:${String(r).padStart(2, "0")}`;
};
const esc = (s) => String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));

function applyLang() {
  document.documentElement.lang = LANG;
  document.querySelectorAll("[data-i18n]").forEach((el) => (el.textContent = t(el.dataset.i18n)));
  document.querySelectorAll("[data-i18n-ph]").forEach((el) => (el.placeholder = t(el.dataset.i18nPh)));
  document.querySelectorAll("[data-i18n-title]").forEach((el) => (el.title = t(el.dataset.i18nTitle)));
  document.querySelectorAll("#lang button").forEach((b) => b.classList.toggle("on", b.dataset.l === LANG));
}
function setLang(l) {
  LANG = l; store.set("lang", l); applyLang();
  // ré-afficher ce qui dépend de la langue
  renderSource(); renderDests(); if (S.info) rebuildClipStates(); else renderClipsPlaceholder();
  if (!S.running && !S.summary) resetDashboard();
  if (S.summary) finishTexts(S.summary);
  setChip(S.chipCls, S.chipKey);
  refresh();
  if ($("modal").innerHTML) S.modalFn && S.modalFn();
}

// ------------------------------------------------------------------ Backends

function tauriBackend() {
  const { invoke } = window.__TAURI__.core;
  const { listen } = window.__TAURI__.event;
  return {
    demo: false, clockScale: 1,
    pickFolder: () => invoke("pick_folder"),
    pickMhl: () => invoke("pick_mhl"),
    scan: (path) => invoke("scan_source", { path }),
    disk: (path) => invoke("disk_info", { path }),
    start: (job) => invoke("start_offload", { job }),
    cancel: () => invoke("cancel_offload"),
    reverify: (path) => invoke("reverify", { path }),
    open: (path) => invoke("open_path", { path }),
    reveal: (path) => invoke("reveal_path", { path }),
    onEvent: (cb) => listen("offload", (e) => cb(e.payload)),
  };
}

function demoBackend() {
  // Cartes RED simulées : 12 clips segmentés par carte
  const redCard = (path) => {
    const id = (path.match(/A\d{3}/) || ["A003"])[0];
    const files = [], clips = [];
    for (let c = 1; c <= 12; c++) {
      const name = `${id}_C${String(c).padStart(3, "0")}_1106${["AB", "K2", "Q7", "ZX"][c % 4]}`;
      const segs = 1 + (c * 7) % 4, idx = [];
      let bytes = 0;
      for (let s = 1; s <= segs; s++) {
        const size = s < segs ? 4_294_967_296 : 600_000_000 + ((c * 911) % 2900) * 1_000_000;
        idx.push(files.length); files.push({ rel: `${id}_1106AB.RDM/${name}.RDC/${name}_00${s}.R3D`, size }); bytes += size;
      }
      idx.push(files.length); files.push({ rel: `${id}_1106AB.RDM/${name}.RDC/${name}.RMD`, size: 48_000 }); bytes += 48_000;
      clips.push({ name, rel_path: `${id}_1106AB.RDM/${name}.RDC`, is_folder: true, brand: "Red", files: idx, bytes });
    }
    return { card_label: path.split("/").pop(), brand_label: "RED", files, clips,
      total_bytes: files.reduce((a, f) => a + f.size, 0) };
  };
  let handler = () => {}, timer = null, cancelled = false, srcPicks = 0, destPicks = 0;
  const destPool = ["/Volumes/SHUTTLE_01", "/Volumes/RAID_PROD", "/Volumes/BACKUP_LTO"];
  const freeMap = { "/Volumes/SHUTTLE_01": 1_840_000_000_000, "/Volumes/RAID_PROD": 23_600_000_000_000, "/Volumes/BACKUP_LTO": 2_900_000_000_000 };
  const hex = () => Array.from({ length: 16 }, () => "0123456789abcdef"[Math.random() * 16 | 0]).join("");
  const SCALE = 14; // accélération de la démo
  return {
    demo: true, clockScale: SCALE,
    pickFolder: async (kind) => (kind === "source" ? `/Volumes/A00${3 + srcPicks++}_RED` : destPool[destPicks++ % destPool.length]),
    pickMhl: async () => "/Volumes/RAID_PROD/2026-11-05_1802_DUE_VOLTI_J02_A_A002/2026-11-05_1802_DUE_VOLTI_J02_A_A002.mhl",
    scan: async (path) => { const c = redCard(path); return { source: path, total_files: c.files.length, ...c }; },
    disk: async (path) => ({ path, free: freeMap[path] ?? 900_000_000_000 }),
    reverify: async () => { await new Promise((r) => setTimeout(r, 900)); return [37, []]; },
    open: async () => log("info", t("demo_open")),
    reveal: async () => log("info", t("demo_reveal")),
    onEvent: (cb) => { handler = cb; },
    cancel: async () => { cancelled = true; },
    start: async (job) => {
      const info = redCard(job.source), files = info.files;
      const n = job.destinations.length + (job.source_reread ? 1 : 0), work = info.total_bytes * (1 + n);
      const now = new Date(), p = (x) => String(x).padStart(2, "0");
      const san = (s) => s.trim().replace(/[^\p{L}\p{N}_-]/gu, "_").replace(/^_+|_+$/g, "");
      const folder = [`${now.getFullYear()}-${p(now.getMonth() + 1)}-${p(now.getDate())}_${p(now.getHours())}${p(now.getMinutes())}`,
        ...[job.project, job.day, job.camera, job.card].map(san).filter(Boolean)].join("_");
      cancelled = false;
      handler({ type: "started", total_files: files.length, total_bytes: info.total_bytes, work_bytes: work, folder_name: folder });
      handler({ type: "log", level: "info", message: t("log_summary", { card: info.card_label, n: files.length, size: fmtBytes(info.total_bytes), brand: "RED", d: job.destinations.length }) });
      let i = 0, inFile = 0, done = 0;
      const t0 = performance.now(), results = [], RATE = 1_050_000_000;
      handler({ type: "file", index: 0, rel: files[0].rel, size: files[0].size });
      timer = setInterval(() => {
        if (cancelled) {
          clearInterval(timer);
          handler({ type: "log", level: "error", message: t("log_cancelled") });
          handler({ type: "finished", summary: { cancelled: true, files_ok: results.length, files_failed: 0, total_files: files.length, reports: [], dest_dirs: [], avg_mb_s: 0, duration_s: 0 } });
          return;
        }
        let budget = RATE * 0.08 * SCALE;
        while (budget > 0 && i < files.length) {
          const f = files[i], need = f.size * (1 + n) - inFile, step = Math.min(need, budget);
          inFile += step; done += step; budget -= step;
          if (inFile >= f.size * (1 + n)) {
            const h = hex();
            const r = { rel: f.rel, size: f.size, source_hash: h, dests: job.destinations.map(() => ({ hash: h, ok: true })), ok: true, error: null, uncached: true };
            results.push(r);
            handler({ type: "file_result", index: i, result: r });
            i++; inFile = 0;
            if (i < files.length) handler({ type: "file", index: i, rel: files[i].rel, size: files[i].size });
          }
        }
        handler({ type: "progress", done, total: work, phase: inFile > (files[i]?.size ?? 0) ? "verify" : "copy", file_index: Math.min(i, files.length - 1) });
        if (i >= files.length) {
          clearInterval(timer);
          const secs = (performance.now() - t0) / 1000 * SCALE;
          handler({ type: "log", level: "ok", message: t("log_done", { n: files.length, s: Math.round(secs), r: Math.round(info.total_bytes / 1e6 / secs) }) });
          handler({ type: "finished", summary: { cancelled: false, files_ok: files.length, files_failed: 0, total_files: files.length,
            folder_name: folder, avg_mb_s: info.total_bytes / 1e6 / secs, duration_s: secs,
            dest_dirs: job.destinations.map((d) => d + "/" + folder),
            reports: job.destinations.map((d) => `${d}/${folder}/${folder}_rapport.pdf`) } });
        }
      }, 80);
    },
  };
}

const api = window.__TAURI__ ? tauriBackend() : demoBackend();
const now = () => performance.now() * api.clockScale / 1000;

// ------------------------------------------------------------------ État

const S = {
  source: null, info: null, dests: [], running: false, summary: null,
  fileToClip: [], clipDone: [], clipState: [], fileSizes: [],
  work: 0, totalBytes: 0, n: 1, completedBytes: 0, curIndex: -1, filesDone: 0, totalFiles: 0,
  t0: 0, samples: [], chipCls: "", chipKey: "ready", srcState: "empty", srcError: "", modalFn: null,
};
const FIELDS = ["project", "day", "camera", "card", "operator", "notes"];

function init() {
  // Version réelle de l'application (Tauri), pour l'affichage dans « À propos »
  window.__TAURI__?.app?.getVersion?.().then((v) => { VERSION = v; S.modalFn?.(); }).catch(() => {});
  applyLang();
  if (api.demo) $("chip").insertAdjacentHTML("afterend", '<span class="demo" data-i18n="demo"></span>');
  FIELDS.forEach((f) => {
    const el = $(f);
    if (f !== "card" && f !== "notes") el.value = store.get(f, "");
    el.addEventListener("input", () => { if (f !== "card" && f !== "notes") store.set(f, el.value); updateName(); });
  });
  $("algo").value = store.get("algo", "xxh64");
  $("algo").onchange = () => { store.set("algo", $("algo").value); refresh(); };
  $("reread").checked = store.get("reread", true);
  $("reread").onchange = () => store.set("reread", $("reread").checked);
  $("src").onclick = chooseSource;
  $("addDest").onclick = addDest;
  $("go").onclick = () => (S.running ? cancel() : S.summary && lastOk() ? newOffload() : start());
  $("openPdf").onclick = () => { const r = S.summary?.reports?.find((x) => x.endsWith(".pdf")); r && api.open(r); };
  $("reveal").onclick = () => S.summary?.dest_dirs?.[0] && api.reveal(S.summary.dest_dirs[0]);
  $("reverifyBtn").onclick = reverify;
  $("aboutBtn").onclick = () => showAbout(false);
  document.querySelectorAll("#lang button").forEach((b) => (b.onclick = () => setLang(b.dataset.l)));
  api.onEvent(onEvent);
  setInterval(updateName, 15000);
  applyLang();
  renderSource(); renderDests(); renderClipsPlaceholder(); resetDashboard(); updateName(); refresh();
  if (!store.get("accepted.v1", false)) showAbout(true);
}

// ------------------------------------------------------------------ À propos / avertissement

const LOGO = (s) => `<svg width="${s}" height="${s}" viewBox="0 0 64 64"><defs><linearGradient id="lg2" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#f0a640"/><stop offset="1" stop-color="#e0663a"/></linearGradient></defs><rect x="2" y="2" width="60" height="60" rx="14" fill="url(#lg2)"/><circle cx="32" cy="32" r="15.5" fill="none" stroke="#111" stroke-width="5"/><path d="M24.5 32.5l5.5 5.5 9.5-11" fill="none" stroke="#111" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/></svg>`;

function showAbout(firstRun) {
  S.modalFn = () => showAbout(firstRun);
  $("modal").innerHTML = `<div class="modal"><div class="box">
    <h2>${LOGO(42)}<div>${firstRun ? esc(t("welcome")) : "Bandua"}<span class="vtag">v${esc(VERSION)}</span><div class="by">by Domingo Olmo Martin</div></div></h2>
    <p style="margin-top:14px">${esc(t("about_desc"))}</p>
    <p style="color:var(--muted)">${esc(t("about_name"))}</p>
    <h4>${esc(t("disclaimer_title"))}</h4>
    <div class="warn"><p>${esc(t("disclaimer_text"))}</p><p style="margin:0;color:var(--text);font-weight:600">${esc(t("disclaimer_rule"))}</p></div>
    <h4>${esc(t("license_title"))}</h4>
    <p>${esc(t("license_text"))}</p>
    <p class="ver">Bandua ${VERSION} · © 2026 Domingo Olmo Martin · MIT License</p>
    <div class="acts">
      <div class="lang" id="mlang">${["fr", "en", "es"].map((l) => `<button data-l="${l}" class="${l === LANG ? "on" : ""}">${l.toUpperCase()}</button>`).join("")}</div>
      <button class="${firstRun ? "go" : "sec"}" id="mclose">${esc(t(firstRun ? "accept" : "close"))}</button>
    </div></div></div>`;
  $("mlang").querySelectorAll("button").forEach((b) => (b.onclick = () => setLang(b.dataset.l)));
  $("mclose").onclick = () => {
    if (firstRun) store.set("accepted.v1", true);
    $("modal").innerHTML = ""; S.modalFn = null;
  };
}

// ------------------------------------------------------------------ Source / destinations

function renderSource() {
  const el = $("src");
  if (S.srcState === "loading") { el.innerHTML = `<div class="empty">${esc(t("analysing"))}</div>`; return; }
  if (S.srcState === "error") { el.innerHTML = `<div class="empty"><b>${esc(t("read_fail"))}</b>${esc(S.srcError)}</div>`; return; }
  if (!S.info) {
    el.classList.remove("has");
    el.innerHTML = `<div class="empty"><b>${esc(t("pick_card"))}</b>${esc(t("pick_card_sub"))}</div>`;
    return;
  }
  const i = S.info;
  el.classList.add("has");
  el.innerHTML = `
    <div><span class="brand b-${esc(i.brand_label)}">${esc(i.brand_label.toUpperCase())}</span><span class="path">${esc(S.source)}</span></div>
    <div class="stats"><span><b>${fmtBytes(i.total_bytes)}</b></span><span><b>${i.clips.length}</b> ${esc(t("clips_word"))}</span><span><b>${i.total_files}</b> ${esc(t("files_word"))}</span></div>`;
}

async function chooseSource() {
  if (S.running) return;
  const p = await api.pickFolder("source");
  if (!p) return;
  if (S.summary) resetView();
  if (S.dests.some((d) => d.path === p)) return log("error", t("already_dest"));
  S.source = p; S.info = null; S.srcState = "loading"; renderSource();
  try {
    S.info = await api.scan(p);
    S.srcState = "ok";
    if (!S.running) $("filesDone").textContent = `0/${S.info.total_files}`;
  } catch (e) {
    S.info = null; S.source = null; S.srcState = "error"; S.srcError = String(e);
    renderSource(); refresh();
    return;
  }
  // Pré-remplissage de la carte et de la caméra à partir des noms de clips (A003_C001… / A002C004…)
  const m = S.info.clips[0]?.name.match(/^([A-Z])(\d{3})/);
  if (m) { $("card").value = m[1] + m[2]; if (!$("camera").value) $("camera").value = m[1]; }
  renderSource(); buildClipTable(); renderDests(); updateName(); refresh();
}

async function addDest() {
  if (S.running) return;
  const p = await api.pickFolder("dest");
  if (!p) return;
  if (p === S.source) return log("error", t("same_as_src"));
  if (S.dests.some((d) => d.path === p)) return;
  const d = { path: p, free: null };
  S.dests.push(d);
  renderDests();
  try { d.free = (await api.disk(p)).free; } catch {}
  renderDests(); refresh();
}

function renderDests() {
  const need = S.info?.total_bytes ?? 0;
  $("destCount").textContent = S.dests.length ? t("single_read", { n: S.dests.length }) : t("none");
  $("dests").innerHTML = S.dests.map((d, k) => {
    const low = d.free != null && need > d.free;
    const pct = d.free ? Math.min(100, (need / d.free) * 100) : 0;
    const txt = d.free == null ? t("free_unknown")
      : t("free", { free: fmtBytes(d.free) }) + (need ? t("occupies", { p: num(pct.toFixed(pct < 1 ? 2 : 0)) }) : "") + (low ? t("low_space") : "");
    return `<div class="dest"><div class="n">${k + 1}</div><div>
        <div class="path">${esc(d.path)}</div>
        <div class="free ${low ? "low" : ""}"><span style="width:${low ? 100 : pct}%"></span></div>
        <div class="freetxt">${esc(txt)}</div>
      </div><button class="x" data-k="${k}" title="${esc(t("remove"))}">×</button></div>`;
  }).join("");
  $("dests").querySelectorAll(".x").forEach((b) => (b.onclick = () => { if (!S.running) { S.dests.splice(+b.dataset.k, 1); renderDests(); refresh(); } }));
}

function folderName() {
  const d = new Date(), p = (x) => String(x).padStart(2, "0");
  const san = (s) => s.trim().replace(/[^\p{L}\p{N}_-]/gu, "_").replace(/^_+|_+$/g, "");
  return [`${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}_${p(d.getHours())}${p(d.getMinutes())}`,
    ...["project", "day", "camera", "card"].map((f) => san($(f).value)).filter(Boolean)].join("_");
}
function updateName() { if (!S.running && !S.summary) $("fname").textContent = folderName(); }

function lastOk() {
  const s = S.summary;
  return !!s && !s.cancelled && s.files_failed === 0 && s.files_ok === s.total_files && !(s.source_changes?.length);
}

function refresh() {
  const need = S.info?.total_bytes ?? 0;
  const lack = S.dests.some((d) => d.free != null && need > d.free);
  const ready = S.info && S.dests.length && !lack;
  $("go").disabled = !S.running && !ready && !lastOk();
  if (!S.cancelArmed) $("go").textContent = S.running ? t("cancel") : S.summary ? t(lastOk() ? "new_offload" : "retry") : t("start");
  $("go").classList.toggle("stop", S.running);
  if (S.running) $("footMsg").textContent = t("f_running");
  else if (!S.summary)
    $("footMsg").textContent = !S.info ? t("f_pick_src") : !S.dests.length ? t("f_add_dest")
      : lack ? t("f_low") : t("f_ready", { size: fmtBytes(need), n: S.dests.length, algo: $("algo").selectedOptions[0].text.split(" ")[0] });
}

// ------------------------------------------------------------------ Tableau des clips

function renderClipsPlaceholder() { if (!S.info) $("clipsWrap").innerHTML = `<div class="placeholder">${esc(t("clips_here"))}</div>`; }

function buildClipTable(keepStates) {
  const c = S.info.clips;
  if (!keepStates) {
    S.fileToClip = [];
    c.forEach((cl, k) => cl.files.forEach((fi) => (S.fileToClip[fi] = k)));
    S.clipDone = c.map(() => 0); S.clipState = c.map(() => ["wait", 0]);
  }
  if (!c.length) { $("clipsWrap").innerHTML = `<div class="placeholder">${esc(t("no_clips"))}</div>`; return; }
  $("clipsWrap").innerHTML = `<table><thead><tr><th style="width:40px">#</th><th>${esc(t("th_clip"))}</th><th>${esc(t("th_type"))}</th><th style="text-align:right">${esc(t("th_files"))}</th><th style="text-align:right">${esc(t("th_size"))}</th><th style="width:170px">${esc(t("th_status"))}</th></tr></thead><tbody>
    ${c.map((cl, k) => `<tr id="clip${k}"><td class="mono">${k + 1}</td>
      <td class="name">${esc(cl.name)}<small>${esc(cl.rel_path)}</small></td>
      <td class="mono">${esc(t(cl.is_folder ? "kind_folder" : "kind_file"))}</td>
      <td class="mono" style="text-align:right">${cl.files.length}</td>
      <td class="mono" style="text-align:right">${fmtBytes(cl.bytes)}</td>
      <td id="cs${k}"></td></tr>`).join("")}</tbody></table>`;
  c.forEach((_, k) => setClip(k, ...S.clipState[k]));
  if (!keepStates) $("clipsWrap").scrollTop = 0;
}
function rebuildClipStates() { buildClipTable(true); }

function setClip(k, state, frac = 0) {
  S.clipState[k] = [state, frac];
  const el = $("cs" + k); if (!el) return;
  el.innerHTML = state === "ok" ? `<span class="st ok"><span class="dot"></span>${esc(t("st_verified"))}</span>`
    : state === "err" ? `<span class="st err"><span class="dot"></span>${esc(t("st_error"))}</span>`
    : state === "run" ? `<span class="st run"><span class="dot"></span>${Math.round(frac * 100)} %</span><span class="mini"><span style="width:${frac * 100}%"></span></span>`
    : `<span class="st wait"><span class="dot"></span>${esc(t("pending"))}</span>`;
  $("clip" + k).classList.toggle("active", state === "run");
}

// ------------------------------------------------------------------ Déchargement

function resetDashboard() {
  $("pct").textContent = "0"; $("phase").textContent = t("waiting");
  $("ringArc").style.strokeDashoffset = 414.69; $("ringArc").style.stroke = "var(--accent)";
  ["speed", "eta", "elapsed"].forEach((id) => ($(id).textContent = "-"));
  $("filesDone").textContent = S.info ? `0/${S.info.total_files}` : "-";
  ["barCopy", "barVerify"].forEach((id) => ($(id).style.width = "0"));
  ["pCopy", "pVerify"].forEach((id) => ($(id).textContent = "0 %"));
  $("cur").textContent = t("no_offload");
}

// Remet l'écran à zéro pour la carte suivante (destinations et identification conservées)
function resetView() {
  S.summary = null; S.info = null; S.source = null; S.srcState = "empty";
  $("banner").innerHTML = "";
  ["openPdf", "reveal"].forEach((b) => $(b).classList.add("hide"));
  $("card").value = ""; $("notes").value = "";
  renderSource(); renderClipsPlaceholder(); resetDashboard();
  setChip("", "ready");
  updateName(); renderDests(); refresh();
}

async function newOffload() {
  log("info", t("new_log"));
  resetView();
  await chooseSource();
}

async function start() {
  if (!S.info || !S.dests.length) return;
  const job = {
    source: S.source, destinations: S.dests.map((d) => d.path), algo: $("algo").value, lang: LANG,
    source_reread: $("reread").checked,
    ...Object.fromEntries(FIELDS.map((f) => [f, $(f).value.trim()])),
  };
  S.summary = null; $("banner").innerHTML = "";
  ["openPdf", "reveal"].forEach((b) => $(b).classList.add("hide"));
  buildClipTable();
  resetDashboard();
  $("log").innerHTML = "";
  S.reread = job.source_reread; S.nDest = job.destinations.length;
  S.running = true; refresh(); setChip("run", "chip_copy");
  try { await api.start(job); } catch (e) { S.running = false; refresh(); setChip("err", "chip_error"); log("fatal", String(e)); }
}

// Double clic pour annuler (pas de boîte de dialogue bloquante)
function cancel() {
  if (S.cancelArmed && Date.now() - S.cancelArmed < 3000) { api.cancel(); S.cancelArmed = 0; $("go").textContent = t("cancelling"); return; }
  S.cancelArmed = Date.now();
  $("go").textContent = t("confirm_cancel");
  setTimeout(() => { if (S.cancelArmed && Date.now() - S.cancelArmed >= 2900) { S.cancelArmed = 0; refresh(); } }, 3000);
}

function onEvent(e) {
  switch (e.type) {
    case "started":
      Object.assign(S, { work: e.work_bytes, totalBytes: e.total_bytes, totalFiles: e.total_files, n: e.work_bytes / e.total_bytes - 1 || 1,
        completedBytes: 0, filesDone: 0, t0: now(), samples: [] });
      S.fileSizes = [];
      $("fname").textContent = e.folder_name;
      $("filesDone").textContent = `0/${e.total_files}`;
      break;
    case "file": {
      S.curIndex = e.index; S.fileSizes[e.index] = e.size;
      $("cur").innerHTML = `<b>${esc(e.rel.split("/").pop())}</b>  ${esc(e.rel)}`;
      const k = S.fileToClip[e.index];
      if (k != null && S.clipState[k][0] === "wait") setClip(k, "run", 0);
      if (k != null) $("clip" + k)?.scrollIntoView({ block: "nearest", behavior: "smooth" });
      break;
    }
    case "progress": progress(e); break;
    case "file_result": {
      const r = e.result; S.filesDone++;
      S.completedBytes += r.size * (1 + S.n);
      const k = S.fileToClip[e.index];
      if (k != null) {
        if (!r.ok) setClip(k, "err");
        S.clipDone[k] += r.size;
        const cl = S.info.clips[k];
        if (S.clipState[k][0] !== "err") setClip(k, cl.files.every((fi) => fi <= e.index) ? "ok" : "run", S.clipDone[k] / cl.bytes);
      }
      $("filesDone").textContent = `${S.filesDone}/${S.totalFiles}`;
      break;
    }
    case "log": log(e.level, e.message);
      if (e.level === "fatal") { S.running = false; refresh(); setChip("err", "chip_error"); }
      break;
    case "finished": finish(e.summary); break;
  }
}

function progress(e) {
  const tt = now();
  S.samples.push([tt, e.done]);
  while (S.samples.length > 2 && tt - S.samples[0][0] > 3) S.samples.shift();
  const [t1, d1] = S.samples[0];
  const rate = tt > t1 ? (e.done - d1) / (tt - t1) : 0;
  const pct = e.total ? e.done / e.total : 0;
  // Répartition copie / vérification
  const size = S.fileSizes[S.curIndex] || 0;
  const inFile = Math.max(0, e.done - S.completedBytes);
  const copyBytes = S.completedBytes / (1 + S.n) + Math.min(inFile, size);
  const verBytes = S.completedBytes / (1 + S.n) * S.n + Math.max(0, inFile - size);
  const pc = S.totalBytes ? copyBytes / S.totalBytes : 0, pv = S.totalBytes ? verBytes / (S.totalBytes * S.n) : 0;
  $("barCopy").style.width = pc * 100 + "%"; $("pCopy").textContent = Math.floor(pc * 100) + " %";
  $("barVerify").style.width = pv * 100 + "%"; $("pVerify").textContent = Math.floor(pv * 100) + " %";
  $("pct").textContent = Math.floor(pct * 100);
  $("ringArc").style.strokeDashoffset = 414.69 * (1 - pct);
  const verifying = inFile > size;
  const rereading = S.reread && inFile > size * (1 + S.nDest);
  $("phase").textContent = t(rereading ? "ph_source" : verifying ? "ph_verify" : "ph_copy");
  setChip("run", verifying ? "chip_verify" : "chip_copy");
  $("speed").innerHTML = rate ? `${Math.round(rate / 1e6)}<small style="font-size:11px;color:var(--muted)"> ${esc(t("mbs"))}</small>` : "-";
  $("eta").textContent = rate ? fmtTime((e.total - e.done) / rate) : "-";
  $("elapsed").textContent = fmtTime(tt - S.t0);
  const k = S.fileToClip[S.curIndex];
  if (k != null && S.clipState[k][0] === "run") {
    const cl = S.info.clips[k];
    setClip(k, "run", Math.min(1, (S.clipDone[k] + Math.min(inFile / (1 + S.n), size)) / cl.bytes));
  }
}

function finishTexts(sum) {
  const ok = lastOk();
  $("phase").textContent = t(ok ? "ph_verified" : sum.cancelled ? "ph_cancelled" : "ph_errors");
  $("cur").innerHTML = ok ? `<b>${esc(sum.folder_name)}</b>  ${esc(t("cur_done"))}` : "";
  $("footMsg").textContent = ok ? t("done_msg", { n: sum.files_ok, d: sum.dest_dirs.length, r: Math.round(sum.avg_mb_s) })
    : sum.cancelled ? t("cancelled_msg") : sum.files_failed ? t("errors_msg", { n: sum.files_failed }) : t("src_changed_msg");
}

function finish(sum) {
  S.running = false; S.cancelArmed = 0; S.summary = sum; refresh();
  const ok = lastOk();
  setChip(ok ? "ok" : "err", ok ? "chip_verified" : sum.cancelled ? "chip_cancelled" : "chip_errors");
  $("ringArc").style.stroke = ok ? "var(--ok)" : "var(--err)";
  if (ok) { $("pct").textContent = "100"; $("ringArc").style.strokeDashoffset = 0; }
  finishTexts(sum);
  if (sum.reports?.length) ["openPdf", "reveal"].forEach((b) => $(b).classList.remove("hide"));
  $("banner").innerHTML = `<div class="banner ${ok ? "ok" : "err"}">${esc(t(ok ? "banner_ok" : sum.cancelled ? "banner_cancel" : "banner_err"))}
    <span>${esc(t(ok ? "banner_ok_sub" : "banner_err_sub"))}</span></div>`;
  setTimeout(() => ($("banner").innerHTML = ""), 9000);
  try { new Audio("data:audio/wav;base64," + chime(ok)).play().catch(() => {}); } catch {}
}

async function reverify() {
  if (S.running) return;
  const p = await api.pickMhl();
  if (!p) return;
  log("info", t("reverify_start", { p }));
  setChip("run", "chip_reverify");
  try {
    const [n, errs] = await api.reverify(p);
    if (errs.length) { errs.forEach((x) => log("error", x)); setChip("err", "chip_errors"); }
    else { log("ok", t("reverify_ok", { n })); setChip("ok", "chip_verified"); }
  } catch (e) { log("fatal", String(e)); setChip("err", "chip_error"); }
}

// ------------------------------------------------------------------ Divers

function setChip(cls, key) {
  S.chipCls = cls; S.chipKey = key;
  const c = $("chip"); c.className = "chip " + cls; c.textContent = t(key);
}
function log(level, msg) {
  const d = new Date(), tm = d.toTimeString().slice(0, 8);
  $("log").insertAdjacentHTML("beforeend", `<div class="${esc(level)}"><span class="t">${tm}</span>${esc(msg)}</div>`);
  $("log").scrollTop = 1e9;
}
// petit signal sonore généré (deux notes montantes = OK, descendantes = erreur)
function chime(ok) {
  const sr = 22050, notes = ok ? [880, 1320] : [440, 330], dur = 0.16, n = Math.floor(sr * dur * notes.length);
  const buf = new DataView(new ArrayBuffer(44 + n * 2));
  const w = (o, s) => [...s].forEach((c, i) => buf.setUint8(o + i, c.charCodeAt(0)));
  w(0, "RIFF"); buf.setUint32(4, 36 + n * 2, true); w(8, "WAVEfmt "); buf.setUint32(16, 16, true); buf.setUint16(20, 1, true);
  buf.setUint16(22, 1, true); buf.setUint32(24, sr, true); buf.setUint32(28, sr * 2, true); buf.setUint16(32, 2, true);
  buf.setUint16(34, 16, true); w(36, "data"); buf.setUint32(40, n * 2, true);
  for (let i = 0; i < n; i++) {
    const tt = i / sr, f = notes[Math.min(notes.length - 1, Math.floor(tt / dur))], lt = tt % dur;
    const env = Math.min(1, lt * 60) * Math.exp(-lt * 14);
    buf.setInt16(44 + i * 2, Math.sin(2 * Math.PI * f * tt) * env * 9000, true);
  }
  let s = ""; new Uint8Array(buf.buffer).forEach((b) => (s += String.fromCharCode(b)));
  return btoa(s);
}

init();
