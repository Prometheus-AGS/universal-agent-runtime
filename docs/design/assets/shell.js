/* UAR operator shell — shared behaviour for the prototype screens.
   Sidebar collapse (⌘B), right inspector (open/close/tab/resize, ⌘\ closes),
   command palette (⌘K), theme toggle, toasts, popover menus, Tauri host detection.
   Framework-neutral on purpose: each piece maps 1:1 onto a React hook or a Zustand slice. */
(function () {
  "use strict";
  const STORE_KEY = "uar.shell.v1";
  const app = document.querySelector(".app");
  if (!app) return;
  const mac = /Mac|iPhone|iPad/.test(navigator.platform);
  const state = Object.assign({ sidebarCollapsed: false, panelOpen: false, panelTab: null, panelW: 440, theme: "dark" }, read());

  function read() { try { return JSON.parse(localStorage.getItem(STORE_KEY) || "{}"); } catch { return {}; } }
  function save() { try { localStorage.setItem(STORE_KEY, JSON.stringify(state)); } catch {} }
  function emit(name, detail) { app.dispatchEvent(new CustomEvent(name, { detail })); }

  /* ─── Tauri host ───────────────────────────────────────────────────── */
  if (window.__TAURI__ || window.__TAURI_INTERNALS__) document.documentElement.classList.add("is-tauri");
  document.querySelectorAll("[data-kbd]").forEach((el) => { el.textContent = mac ? el.dataset.kbd : el.dataset.kbd.replace("⌘", "Ctrl+").replace("⌥", "Alt+").replace("⇧", "Shift+"); });

  /* ─── theme ────────────────────────────────────────────────────────── */
  /* theme: dark | light | hc | system (system follows prefers-color-scheme). Appearance prefs ride the same store:
     density (comfortable | compact), motion (system | reduce | full), threadFs (px, conversation text only). */
  const sysLight = window.matchMedia("(prefers-color-scheme: light)");
  function resolvedTheme() { return state.theme === "system" ? (sysLight.matches ? "light" : "dark") : state.theme; }
  function applyTheme() {
    const root = document.documentElement; root.dataset.theme = resolvedTheme();
    root.dataset.density = state.density || "comfortable"; root.dataset.motion = state.motion || "system";
    root.style.setProperty("--thread-fs", (state.threadFs || 15) + "px");
    document.querySelectorAll("[data-action=theme]").forEach((b) => b.setAttribute("aria-pressed", resolvedTheme() === "light"));
  }
  function setPref(key, value) { state[key] = value; applyTheme(); save(); emit("uar:prefs", { key, value }); }
  sysLight.addEventListener("change", () => { if (state.theme === "system") applyTheme(); });
  applyTheme();

  /* ─── sidebar ──────────────────────────────────────────────────────── */
  function setSidebar(collapsed) {
    state.sidebarCollapsed = collapsed; app.classList.toggle("sidebar-collapsed", collapsed);
    document.querySelectorAll("[data-action=sidebar]").forEach((b) => { b.setAttribute("aria-expanded", String(!collapsed)); b.setAttribute("aria-label", collapsed ? "Expand sidebar" : "Collapse sidebar"); });
    save(); emit("uar:sidebar", { collapsed });
  }
  setSidebar(state.sidebarCollapsed);

  /* ─── right panel ──────────────────────────────────────────────────── */
  const panel = app.querySelector(".panel");
  function setPanelWidth(w) { state.panelW = Math.min(760, Math.max(360, Math.round(w))); app.style.setProperty("--panel-w", state.panelW + "px"); }
  setPanelWidth(state.panelW);
  function selectTab(id) {
    if (!panel) return;
    const tabs = panel.querySelectorAll("[role=tab]"); const panes = panel.querySelectorAll("[role=tabpanel]");
    let found = false;
    tabs.forEach((t) => { const on = t.dataset.tab === id; t.setAttribute("aria-selected", String(on)); t.tabIndex = on ? 0 : -1; if (on) found = true; });
    if (!found && tabs[0]) { id = tabs[0].dataset.tab; tabs[0].setAttribute("aria-selected", "true"); tabs[0].tabIndex = 0; }
    panes.forEach((p) => p.classList.toggle("active", p.dataset.tabpanel === id));
    state.panelTab = id; save(); if (state.panelOpen) syncPanelButtons(); emit("uar:paneltab", { tab: id });
  }
  function syncPanelButtons() { document.querySelectorAll("[data-action=panel]").forEach((b) => b.setAttribute("aria-pressed", String(state.panelOpen && (!b.dataset.tab || b.dataset.tab === state.panelTab)))); }
  function openPanel(tab, opts) {
    if (!panel) return;
    if (tab) selectTab(tab); else if (!state.panelTab) selectTab(null);
    const wasOpen = state.panelOpen;
    state.panelOpen = true; app.classList.add("panel-open");
    syncPanelButtons();
    panel.removeAttribute("aria-hidden");
    if (opts && opts.focus) { const t = panel.querySelector("[role=tab][aria-selected=true]"); if (t) t.focus(); }
    save(); if (!wasOpen) emit("uar:panel", { open: true });
  }
  function closePanel() {
    if (!panel || !state.panelOpen) return;
    state.panelOpen = false; app.classList.remove("panel-open");
    document.querySelectorAll("[data-action=panel]").forEach((b) => b.setAttribute("aria-pressed", "false"));
    panel.setAttribute("aria-hidden", "true"); save(); emit("uar:panel", { open: false });
  }
  function togglePanel(tab) { if (state.panelOpen && (!tab || tab === state.panelTab)) closePanel(); else openPanel(tab); }
  if (panel) {
    selectTab(state.panelTab);
    if (state.panelOpen && app.dataset.panelPersist !== "false") openPanel(state.panelTab); else { state.panelOpen = false; panel.setAttribute("aria-hidden", "true"); }
    panel.addEventListener("click", (e) => { const t = e.target.closest("[role=tab]"); if (t) selectTab(t.dataset.tab); });
    panel.addEventListener("keydown", (e) => {
      const t = e.target.closest("[role=tab]"); if (!t) return;
      const tabs = [...panel.querySelectorAll("[role=tab]")]; const i = tabs.indexOf(t);
      if (e.key === "ArrowRight" || e.key === "ArrowLeft") { e.preventDefault(); const n = tabs[(i + (e.key === "ArrowRight" ? 1 : tabs.length - 1)) % tabs.length]; selectTab(n.dataset.tab); n.focus(); }
    });
    const handle = panel.querySelector(".panel-handle");
    if (handle) {
      let startX = 0, startW = 0;
      const move = (e) => setPanelWidth(startW + (startX - e.clientX));
      const up = () => { app.classList.remove("panel-resizing"); window.removeEventListener("pointermove", move); window.removeEventListener("pointerup", up); save(); };
      handle.addEventListener("pointerdown", (e) => { startX = e.clientX; startW = state.panelW; app.classList.add("panel-resizing"); window.addEventListener("pointermove", move); window.addEventListener("pointerup", up); });
      handle.addEventListener("dblclick", () => { setPanelWidth(440); save(); });
      handle.setAttribute("role", "separator"); handle.setAttribute("aria-orientation", "vertical"); handle.setAttribute("aria-label", "Resize inspector"); handle.tabIndex = 0;
      handle.addEventListener("keydown", (e) => { if (e.key === "ArrowLeft") { setPanelWidth(state.panelW + 24); save(); } if (e.key === "ArrowRight") { setPanelWidth(state.panelW - 24); save(); } });
    }
    const overlay = app.querySelector(".panel-overlay"); if (overlay) overlay.addEventListener("click", closePanel);
  }

  /* ─── menus (popovers) ─────────────────────────────────────────────── */
  let openMenu = null;
  function closeMenus() { if (openMenu) { openMenu.classList.remove("open"); const t = document.querySelector(`[aria-controls="${openMenu.id}"]`); if (t) t.setAttribute("aria-expanded", "false"); openMenu = null; } }
  function toggleMenu(trigger) {
    const m = document.getElementById(trigger.getAttribute("aria-controls")); if (!m) return;
    if (openMenu === m) { closeMenus(); return; }
    closeMenus(); m.classList.add("open"); trigger.setAttribute("aria-expanded", "true"); openMenu = m;
    const first = m.querySelector(".menu-item, .input"); if (first) first.focus({ preventScroll: true });
  }
  document.addEventListener("pointerdown", (e) => { if (openMenu && !openMenu.contains(e.target) && !e.target.closest(`[aria-controls="${openMenu.id}"]`)) closeMenus(); });

  /* ─── command palette ──────────────────────────────────────────────── */
  const cmdk = document.querySelector(".cmdk"); const cmdkBack = document.querySelector(".cmdk-backdrop");
  function openCmdk() { if (!cmdk) return; cmdk.classList.add("open"); cmdkBack.classList.add("open"); const i = cmdk.querySelector("input"); i.value = ""; filterCmdk(""); i.focus(); }
  function closeCmdk() { if (!cmdk) return; cmdk.classList.remove("open"); cmdkBack.classList.remove("open"); }
  function filterCmdk(q) {
    if (!cmdk) return; q = q.trim().toLowerCase(); let first = null;
    cmdk.querySelectorAll(".cmdk-list .menu-item").forEach((it) => { const hit = !q || it.textContent.toLowerCase().includes(q); it.style.display = hit ? "" : "none"; it.classList.remove("sel"); if (hit && !first) first = it; });
    cmdk.querySelectorAll(".cmdk-list .menu-label").forEach((l) => { let n = l.nextElementSibling, any = false; while (n && !n.classList.contains("menu-label")) { if (n.style.display !== "none" && n.classList.contains("menu-item")) any = true; n = n.nextElementSibling; } l.style.display = any ? "" : "none"; });
    if (first) first.classList.add("sel");
  }
  if (cmdk) {
    cmdk.querySelector("input").addEventListener("input", (e) => filterCmdk(e.target.value));
    cmdk.addEventListener("keydown", (e) => {
      const items = [...cmdk.querySelectorAll(".cmdk-list .menu-item")].filter((i) => i.style.display !== "none"); const cur = items.findIndex((i) => i.classList.contains("sel"));
      if (e.key === "ArrowDown" || e.key === "ArrowUp") { e.preventDefault(); items.forEach((i) => i.classList.remove("sel")); const n = items[(cur + (e.key === "ArrowDown" ? 1 : items.length - 1) + items.length) % items.length]; if (n) { n.classList.add("sel"); n.scrollIntoView && null; } }
      if (e.key === "Enter") { e.preventDefault(); const s = items[cur] || items[0]; if (s) s.click(); }
    });
    cmdk.addEventListener("click", (e) => { const it = e.target.closest(".menu-item"); if (it) { closeCmdk(); run(it); } });
    cmdkBack.addEventListener("click", closeCmdk);
  }

  /* ─── toast ────────────────────────────────────────────────────────── */
  let toastTimer;
  function toast(msg, kind) {
    let t = document.querySelector(".toast"); if (!t) { t = document.createElement("div"); t.className = "toast"; t.setAttribute("role", "status"); t.innerHTML = '<span class="status-dot"></span><span class="msg"></span>'; document.body.appendChild(t); }
    t.querySelector(".msg").textContent = msg; const d = t.querySelector(".status-dot"); d.className = "status-dot" + (kind === "warn" ? " warn" : kind === "err" ? " err" : "");
    t.classList.add("show"); clearTimeout(toastTimer); toastTimer = setTimeout(() => t.classList.remove("show"), 2600);
  }

  /* ─── actions ──────────────────────────────────────────────────────── */
  function run(el) {
    const a = el.dataset.action; if (!a) return false;
    switch (a) {
      case "sidebar": setSidebar(!state.sidebarCollapsed); return true;
      case "sidebar-mobile": app.classList.toggle("sidebar-mobile-open"); return true;
      case "panel": togglePanel(el.dataset.tab); return true;
      case "panel-open": openPanel(el.dataset.tab, { focus: false }); return true;
      case "panel-close": closePanel(); return true;
      case "menu": toggleMenu(el); return true;
      case "cmdk": openCmdk(); return true;
      case "theme": setPref("theme", resolvedTheme() === "light" ? "dark" : "light"); return true;
      case "toast": toast(el.dataset.msg || "Done", el.dataset.kind); return true;
      case "nav": if (el.dataset.href) location.href = el.dataset.href; return true;
      default: emit("uar:action", { action: a, el }); return true;
    }
  }
  document.addEventListener("click", (e) => { const el = e.target.closest("[data-action]"); if (!el) return; if (el.closest(".menu") && el.dataset.action !== "menu" && !el.dataset.keepMenu) closeMenus(); run(el); });

  /* ─── keyboard ─────────────────────────────────────────────────────── */
  document.addEventListener("keydown", (e) => {
    const mod = mac ? e.metaKey : e.ctrlKey; const k = e.key.toLowerCase();
    if (e.key === "Escape") { if (cmdk && cmdk.classList.contains("open")) { closeCmdk(); return; } if (openMenu) { closeMenus(); return; } if (state.panelOpen && window.matchMedia("(max-width: 1180px)").matches) { closePanel(); return; } if (app.classList.contains("sidebar-mobile-open")) { app.classList.remove("sidebar-mobile-open"); return; } }
    if (!mod) return;
    if (k === "b" && !e.shiftKey) { e.preventDefault(); setSidebar(!state.sidebarCollapsed); }
    else if (k === "k" && !e.shiftKey) { e.preventDefault(); openCmdk(); }
    else if (k === "\\") { e.preventDefault(); togglePanel(); }
    else if (e.shiftKey && k === "d") { e.preventDefault(); togglePanel("run"); }
    else if (e.shiftKey && k === "e") { e.preventDefault(); togglePanel("files"); }
    else if (e.shiftKey && k === "b") { e.preventDefault(); togglePanel("browser"); }
    else if (e.shiftKey && k === "o") { e.preventDefault(); emit("uar:action", { action: "new-chat" }); toast("New chat"); }
    else if (k === "," ) { e.preventDefault(); emit("uar:action", { action: "settings" }); }
  });

  window.UARShell = { state, setPref, resolvedTheme, openPanel, closePanel, togglePanel, selectTab, setSidebar, toast, closeMenus, openCmdk };
})();
