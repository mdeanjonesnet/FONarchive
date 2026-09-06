import { createVis, IDLE_SPEED, RUN_SPEED } from "./vis.js";

const LED_SEGMENTS = 24;
const LIST_CAP = 12;
const DEST_KEY = "fonarch-dest";

const listEl = document.getElementById("list");
const ledEl = document.getElementById("led");
const statusEl = document.getElementById("status");
const gatherBtn = document.getElementById("gather");
const archiveBtn = document.getElementById("archive");
const gearBtn = document.getElementById("gear");
const menuEl = document.getElementById("menu");
const quitBtn = document.getElementById("quit");
const destLabel = document.getElementById("dest-label");
const canvas = document.getElementById("vis");

const vis = createVis(canvas);
let helpMode = true;
let running = false;
let nameQueue = [];
let latestProgress = null;
let progressRaf = 0;
/** null = Desktop */
let destParent = localStorage.getItem(DEST_KEY) || null;
let desktopPath = "";

function inTauri() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function savePlace() {
  if (!destParent || (desktopPath && destParent === desktopPath)) {
    return "your Desktop";
  }
  const name = destParent.split(/[/\\]/).filter(Boolean).pop();
  return name ? `your ${name}` : "your chosen folder";
}

function helpLines() {
  return [
    "GATHER finds your hidden",
    "fonts and organizes them",
    "in a folder on",
    savePlace(),
    "",
    "ARCHIVE does the same but",
    "creates a .zip archive of",
    "your hidden fonts",
  ];
}

function pinListBottom() {
  listEl.scrollTop = listEl.scrollHeight;
}

function showHelp() {
  helpMode = true;
  listEl.innerHTML = helpLines()
    .map((line) => `<div class="row">${line || "\u00a0"}</div>`)
    .join("");
  pinListBottom();
}

function setStatus(text) {
  statusEl.textContent = text;
}

function setLed(filled) {
  const n = Math.max(0, Math.min(LED_SEGMENTS, filled));
  const cells = ledEl.children;
  for (let i = 0; i < cells.length; i++) {
    cells[i].classList.toggle("on", i < n);
  }
}

function quantize(index, total) {
  if (!total || index <= 0) return 0;
  return Math.ceil((index / total) * LED_SEGMENTS);
}

function trimList(keepEnd) {
  const cap = keepEnd ? LIST_CAP + 1 : LIST_CAP;
  while (listEl.children.length > cap) {
    const first = listEl.firstChild;
    if (first && first.classList && first.classList.contains("end")) break;
    listEl.removeChild(first);
  }
}

function pushName(name) {
  if (helpMode) {
    listEl.innerHTML = "";
    helpMode = false;
  }
  const row = document.createElement("div");
  row.className = "row";
  row.textContent = name;
  const end = listEl.querySelector(".row.end");
  if (end) {
    listEl.insertBefore(row, end);
  } else {
    listEl.appendChild(row);
  }
  trimList(Boolean(end));
  pinListBottom();
}

function pushEnd() {
  const old = listEl.querySelector(".row.end");
  if (old) old.remove();
  const row = document.createElement("div");
  row.className = "row end";
  row.textContent = "END";
  listEl.appendChild(row);
  trimList(true);
  pinListBottom();
}

function setMenuOpen(open) {
  menuEl.hidden = !open;
  gearBtn.setAttribute("aria-expanded", open ? "true" : "false");
}

async function openExternal(url) {
  if (inTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("open_url", { url });
    return;
  }
  window.open(url, "_blank", "noopener,noreferrer");
}

function setRunning(on) {
  running = on;
  gatherBtn.disabled = on;
  archiveBtn.disabled = on;
  vis.setSpeed(on ? RUN_SPEED : IDLE_SPEED);
}

function cssHex(name, fallback) {
  const raw = getComputedStyle(document.body).getPropertyValue(name).trim();
  return raw || fallback;
}

function applyTheme(name) {
  if (name === "cyan") name = "c64";
  document.body.dataset.theme = name;
  localStorage.setItem("fonarch-theme", name);
  vis.setAccent(cssHex("--accent", "#ffb000"));
  vis.setBg(cssHex("--bg", "#000000"));
  document.querySelectorAll(".theme-btn").forEach((btn) => {
    btn.setAttribute("aria-pressed", btn.dataset.theme === name ? "true" : "false");
  });
}

function refreshDestLabel() {
  destLabel.textContent = destParent ? destParent : "Desktop";
  destLabel.title = destParent || desktopPath || "Desktop";
  if (helpMode) showHelp();
}

function setDest(path) {
  destParent = path || null;
  if (destParent) localStorage.setItem(DEST_KEY, destParent);
  else localStorage.removeItem(DEST_KEY);
  refreshDestLabel();
}

function buildLed() {
  ledEl.innerHTML = "";
  for (let i = 0; i < LED_SEGMENTS; i++) {
    ledEl.appendChild(document.createElement("i"));
  }
}

async function currentWindow() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  return getCurrentWindow();
}

async function startJob(mode) {
  if (running) return;
  setRunning(true);
  setLed(0);
  nameQueue = [];
  latestProgress = null;
  listEl.innerHTML = "";
  helpMode = true;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("run_job", { mode, dest: destParent });
  } catch (err) {
    setRunning(false);
    vis.setSpeed(IDLE_SPEED);
    setStatus(String(err));
    if (!listEl.children.length) showHelp();
  }
}

function flushProgress() {
  progressRaf = 0;
  const p = latestProgress;
  if (!p) return;
  setRunning(true);
  if (nameQueue.length) {
    for (const name of nameQueue) pushName(name);
    nameQueue = [];
  }
  setLed(quantize(p.index, p.total));
  if (p.kind === "zip") {
    setStatus(`Zipping (${p.index}/${p.total})`);
  } else {
    setStatus(`Gathering ${p.name} (${p.index}/${p.total})`);
  }
}

function onPayload(p) {
  switch (p.kind) {
    case "status":
      setStatus(p.text);
      break;
    case "found":
      setStatus(`Found livetype (${p.via})`);
      break;
    case "plan":
      setStatus(`Catalog ${p.total} fonts / ${p.families} families`);
      break;
    case "font":
      nameQueue.push(p.name);
      latestProgress = p;
      if (!progressRaf) progressRaf = requestAnimationFrame(flushProgress);
      break;
    case "zip":
      latestProgress = p;
      if (!progressRaf) progressRaf = requestAnimationFrame(flushProgress);
      break;
    case "warn":
      setStatus(p.text);
      break;
    case "done":
      flushProgress();
      latestProgress = null;
      pushEnd();
      setLed(LED_SEGMENTS);
      setRunning(false);
      setStatus(destParent ? "Saved." : "Saved to Desktop");
      break;
    case "failed":
      flushProgress();
      latestProgress = null;
      setRunning(false);
      vis.setSpeed(IDLE_SPEED);
      setStatus(p.message || "Failed");
      break;
    default:
      break;
  }
}

buildLed();
showHelp();
setLed(0);
setStatus("Ready...");
refreshDestLabel();

const saved = localStorage.getItem("fonarch-theme") || "amber";
applyTheme(saved);

new ResizeObserver(() => vis.resize()).observe(canvas);

gatherBtn.addEventListener("click", () => startJob("gather"));
archiveBtn.addEventListener("click", () => startJob("archive"));

gearBtn.addEventListener("click", (e) => {
  e.stopPropagation();
  setMenuOpen(menuEl.hidden);
});
document.addEventListener("click", (e) => {
  if (!menuEl.hidden && !menuEl.contains(e.target) && e.target !== gearBtn) {
    setMenuOpen(false);
  }
});
document.querySelectorAll(".theme-btn").forEach((btn) => {
  btn.addEventListener("click", () => applyTheme(btn.dataset.theme));
});
document.querySelectorAll("a.ext").forEach((a) => {
  a.addEventListener("click", async (e) => {
    e.preventDefault();
    e.stopPropagation();
    try {
      await openExternal(a.href);
    } catch {
      /* overlay stays up */
    }
  });
});

document.getElementById("browse").addEventListener("click", async (e) => {
  e.stopPropagation();
  if (!inTauri()) return;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const picked = await open({
    directory: true,
    multiple: false,
    title: "FONarch save folder",
    defaultPath: destParent || desktopPath || undefined,
  });
  if (typeof picked === "string" && picked) setDest(picked);
});

document.getElementById("reset-dest").addEventListener("click", (e) => {
  e.stopPropagation();
  setDest(null);
});

document.getElementById("min").addEventListener("click", async () => {
  if (!inTauri()) return;
  (await currentWindow()).minimize();
});
document.getElementById("close").addEventListener("click", async () => {
  if (!inTauri()) return;
  (await currentWindow()).close();
});
quitBtn.addEventListener("click", async () => {
  if (!inTauri()) return;
  (await currentWindow()).close();
});

async function connectTauri() {
  if (!inTauri()) return;
  const { listen } = await import("@tauri-apps/api/event");
  const { invoke } = await import("@tauri-apps/api/core");
  await listen("fonarch", (event) => onPayload(event.payload));
  try {
    desktopPath = await invoke("desktop_path");
    refreshDestLabel();
  } catch {
    desktopPath = "";
  }
}

connectTauri();

window.addEventListener("keydown", (e) => {
  if (e.key === "Escape" && !menuEl.hidden) {
    setMenuOpen(false);
    return;
  }
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "q") {
    if (inTauri()) currentWindow().then((w) => w.close());
  }
});
