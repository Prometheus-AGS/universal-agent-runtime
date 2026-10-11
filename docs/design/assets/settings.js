/* UAR settings pages — shared scaffolding. Load BEFORE shell.js.
   1. Injects the icon sprite once.
   2. Renders the settings sidebar into <aside data-settings-nav data-current="…">, so the nav
      is defined in one place (in React this is one <SettingsNav> fed by a route table).
   3. Wires controls every settings page shares: switches, segmented choices, detail tabs,
      master–detail source rows, mobile nav button, Esc back to chat.
   Pages listen for "uar:switch" / "uar:choice" / "uar:source" on the element or document. */
(function () {
  "use strict";
  const P = (d) => `<path d="${d}"/>`;
  const ICONS = {
    plus: P("M12 5v14M5 12h14"), search: '<circle cx="11" cy="11" r="7"/>' + P("m20 20-3.5-3.5"), back: P("M15 6l-6 6 6 6"),
    sidebar: '<rect x="3" y="4" width="18" height="16" rx="2.5"/>' + P("M9 4v16"),
    gear: '<circle cx="12" cy="12" r="3"/>' + P("M12 2v3M12 19v3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M2 12h3M19 12h3M4.9 19.1 7 17M17 7l2.1-2.1"),
    cpu: '<rect x="5" y="5" width="14" height="14" rx="2"/><rect x="9" y="9" width="6" height="6"/>' + P("M9 2v3M15 2v3M9 19v3M15 19v3M2 9h3M2 15h3M19 9h3M19 15h3"),
    tool: P("M14.7 6.3a4 4 0 0 0 5 5L22 9l-3-3-1.7 1.7zM3 21l8.5-8.5") + P("m10 7-4-4-3 3 4 4"),
    spark: P("M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z"),
    book: P("M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2z") + P("M4 19a2 2 0 0 1 2-2h13"),
    shield: P("M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6z") + P("m9 12 2 2 4-4"),
    sun: '<circle cx="12" cy="12" r="4"/>' + P("M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"),
    moon: P("M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z"),
    contrast: '<circle cx="12" cy="12" r="9"/>' + P("M12 3v18a9 9 0 0 0 0-18z"),
    desktop: '<rect x="3" y="4" width="18" height="12" rx="2"/>' + P("M8 20h8M12 16v4"),
    info: '<circle cx="12" cy="12" r="9"/>' + P("M12 11v5M12 8h.01"), x: P("M6 6l12 12M18 6 6 18"), check: P("m5 12 5 5L20 7"),
    refresh: P("M20 12a8 8 0 1 1-2.3-5.7M20 4v5h-5"), bolt: P("M13 2 4 14h7l-1 8 9-12h-7z"),
    more: '<circle cx="5" cy="12" r="1.2"/><circle cx="12" cy="12" r="1.2"/><circle cx="19" cy="12" r="1.2"/>',
    trash: P("M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13"), globe: '<circle cx="12" cy="12" r="9"/>' + P("M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"),
    term: '<rect x="3" y="4" width="18" height="16" rx="2.5"/>' + P("m7 9 3 3-3 3M12 15h5"),
    file: P("M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z") + P("M14 3v5h5"),
    db: '<ellipse cx="12" cy="6" rx="8" ry="3"/>' + P("M4 6v12c0 1.7 3.6 3 8 3s8-1.3 8-3V6M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3"),
    lock: '<rect x="5" y="11" width="14" height="10" rx="2"/>' + P("M8 11V7a4 4 0 0 1 8 0v4"),
    box: P("M12 3l8 4.5v9L12 21l-8-4.5v-9z") + P("M4 7.5 12 12l8-4.5M12 12v9"), list: P("M9 6h11M9 12h11M9 18h11M4 6h.01M4 12h.01M4 18h.01"),
    upload: P("M12 16V4M7 9l5-5 5 5M4 20h16"), download: P("M12 4v12M7 11l5 5 5-5M4 20h16"),
    git: '<circle cx="6" cy="6" r="2.5"/><circle cx="6" cy="18" r="2.5"/><circle cx="18" cy="8" r="2.5"/>' + P("M6 8.5v7M18 10.5c0 4-6 3-10.5 6.5"),
    play: P("M7 4v16l13-8z"), brain: P("M9 4a3 3 0 0 0-3 3 3 3 0 0 0-2 5 3 3 0 0 0 2 5 3 3 0 0 0 6 1V5a3 3 0 0 0-3-1zM15 4a3 3 0 0 1 3 3 3 3 0 0 1 2 5 3 3 0 0 1-2 5 3 3 0 0 1-6 1"),
    coins: '<ellipse cx="9" cy="7" rx="6" ry="3"/>' + P("M3 7v5c0 1.7 2.7 3 6 3s6-1.3 6-3V7M15 11.5c3.3 0 6 1.3 6 3s-2.7 3-6 3-6-1.3-6-3M21 14.5v3c0 1.7-2.7 3-6 3"),
    key: '<circle cx="8" cy="15" r="4"/>' + P("M11 12l9-9M17 6l3 3M14 9l2 2"),
    eye: P("M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z") + '<circle cx="12" cy="12" r="3"/>',
    folder: P("M3 6a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"),
    history: P("M3 12a9 9 0 1 0 3-6.7L3 8M3 3v5h5M12 7v5l3 2"), flask: P("M9 3h6M10 3v6L4.5 18.5A2 2 0 0 0 6.2 21h11.6a2 2 0 0 0 1.7-2.5L14 9V3M7 15h10"),
    type: P("M4 7V5h16v2M9 19h6M12 5v14"), layout: '<rect x="3" y="4" width="18" height="16" rx="2.5"/>' + P("M3 10h18M9 10v10"),
    keyboard: '<rect x="2" y="6" width="20" height="12" rx="2"/>' + P("M6 10h.01M10 10h.01M14 10h.01M18 10h.01M7 14h10"),
    alert: P("M12 3 2 20h20zM12 10v4M12 17h.01"), users: '<circle cx="9" cy="8" r="3.5"/>' + P("M2.5 20a6.5 6.5 0 0 1 13 0M16 4.5a3.5 3.5 0 0 1 0 7M18 14a6 6 0 0 1 3.5 6"),
    link: P("M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"),
    copy: '<rect x="8" y="8" width="12" height="12" rx="2"/>' + P("M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"),
    plug: P("M9 3v5M15 3v5M6 8h12v3a6 6 0 0 1-12 0zM12 17v4"), clock: '<circle cx="12" cy="12" r="9"/>' + P("M12 7v5l3 2"),
    bot: '<rect x="4" y="8" width="16" height="12" rx="3"/>' + P("M12 4v4M9 14h.01M15 14h.01"),
    runs: P("M4 6h16M4 12h10M4 18h16") + '<circle cx="18" cy="12" r="2"/>',
    layers: P("M12 3 3 8l9 5 9-5zM3 13l9 5 9-5M3 17.5l9 5 9-5"),
    route: '<circle cx="6" cy="18" r="2.5"/><circle cx="18" cy="6" r="2.5"/>' + P("M8.5 18H15a3 3 0 0 0 0-6H9a3 3 0 0 1 0-6h6.5"),
    chat: P("M4 5h16v11H9l-5 4z"), wand: P("M15 4V2M15 10V8M12 5h-2M20 5h-2M17.8 7.8 19 9M17.8 2.2 19 1M3 21l9-9M12.2 6.2 11 5"),
    stop: '<rect x="6" y="6" width="12" height="12" rx="2"/>', chevr: P("m9 6 6 6-6 6"), chevd: P("m6 9 6 6 6-6"),
    pin: P("M12 17v5M8 3h8l-1 7 3 3H6l3-3z"), scale: P("M12 3v18M5 7h14M5 7l-3 7a3.5 3.5 0 0 0 6 0zM19 7l-3 7a3.5 3.5 0 0 0 6 0zM8 21h8"),
  };
  const sprite = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  sprite.setAttribute("width", "0"); sprite.setAttribute("height", "0"); sprite.setAttribute("aria-hidden", "true"); sprite.style.position = "absolute";
  sprite.innerHTML = Object.entries(ICONS).map(([k, v]) => `<symbol id="i-${k}" viewBox="0 0 24 24">${v}</symbol>`).join("");
  document.body.prepend(sprite);

  /* ─── settings route table ─────────────────────────────────────────── */
  const ROUTES = [
    ["general", "General", "gear", null],
    ["models", "Models &amp; providers", "cpu", "models-and-providers.html"],
    ["tools", "Tools &amp; MCP", "tool", "tools-and-mcp.html"],
    ["skills", "Skills", "spark", "skills.html"],
    ["knowledge", "Knowledge &amp; memory", "book", "knowledge-and-memory.html"],
    ["governance", "Governance", "shield", "governance.html"],
    ["appearance", "Appearance", "sun", "appearance.html"],
    ["desktop", "Desktop", "desktop", "desktop.html"],
  ];
  const nav = document.querySelector("[data-settings-nav]");
  if (nav) {
    const cur = nav.dataset.current;
    const ic = (n) => `<svg class="icon"><use href="#i-${n}"/></svg>`;
    nav.classList.add("sidebar", "settings-nav");
    nav.setAttribute("aria-label", "Settings navigation");
    nav.innerHTML = `
      <div class="sidebar-top">
        <a class="brand" href="chat-workspace.html" aria-label="Back to chat"><img src="assets/brand/uar-mark-dark.svg" alt="" /><span class="brand-name">Settings<small>// DEFAULT WORKSPACE</small></span></a>
        <button class="icon-btn" data-action="sidebar" aria-expanded="true" aria-label="Collapse sidebar" title="Toggle sidebar (⌘B)">${ic("sidebar")}</button>
      </div>
      <button class="nav-item rail-only icon-btn" data-action="sidebar" aria-label="Expand sidebar">${ic("sidebar")}</button>
      <div class="sidebar-section"><a class="nav-item" href="chat-workspace.html" title="Back to chat">${ic("back")}<span class="grow">Back to chat</span><kbd>Esc</kbd></a></div>
      <div class="sidebar-section" data-od-id="settings-nav">
        <p class="sidebar-label">Settings</p>
        ${ROUTES.map(([id, label, icon, href]) => `<a class="nav-item" href="${href || "#"}" title="${label}"${id === cur ? ' aria-current="page"' : ""}${href ? "" : ' aria-disabled="true"'}>${ic(icon)}<span class="grow">${label}</span></a>`).join("")}
      </div>
      <div class="sidebar-scroll"></div>
      <div class="sidebar-bottom">
        <a class="nav-item" href="#" title="About">${ic("info")}<span class="grow">About</span><span class="mono" style="font-size:11px;color:var(--faint)">1.0.0</span></a>
        <div class="account"><span class="avatar">TJ</span><span class="who"><b>Travis James</b><span><i class="status-dot"></i>Runtime healthy</span></span></div>
      </div>`;
  }

  /* ─── workspace sidebar (Knowledge · Agents · A2UI registries · Runs) ── */
  const WORKSPACE = [
    ["projects", "Projects", "folder", "projects.html", ""],
    ["knowledge", "Knowledge", "book", "knowledge.html", ""],
    ["agents", "Agents", "bot", "agents.html", ""],
    ["registries", "A2UI registries", "layers", "a2ui-registries.html", ""],
    ["runs", "Runs", "runs", "runs.html", '<span class="chip live" style="height:20px;padding:0 7px;font-size:11px">2</span>'],
  ];
  const RECENT = [["Staging MCP timeouts", true], ["Rotate the Groq key"], ["Summarize the Cedar policy diff"], ["Knowledge base re-embed plan"], ["Why did run 7f3a stall?"]];
  const wnav = document.querySelector("[data-workspace-nav]");
  if (wnav) {
    const cur = wnav.dataset.current;
    const ic = (n) => `<svg class="icon"><use href="#i-${n}"/></svg>`;
    wnav.classList.add("sidebar"); wnav.setAttribute("aria-label", "Workspace navigation");
    wnav.innerHTML = `
      <div class="sidebar-top">
        <a class="brand" href="index.html" aria-label="Universal Agent Runtime home"><img src="assets/brand/uar-mark-dark.svg" alt="" /><span class="brand-name">Universal Agent Runtime<small>// UAR RUNTIME</small></span></a>
        <button class="icon-btn" data-action="sidebar" aria-expanded="true" aria-label="Collapse sidebar" title="Toggle sidebar (⌘B)">${ic("sidebar")}</button>
      </div>
      <button class="nav-item rail-only icon-btn" data-action="sidebar" aria-label="Expand sidebar">${ic("sidebar")}</button>
      <div class="sidebar-section" data-od-id="sidebar-primary">
        <a class="nav-item primary" href="chat-workspace.html?new=1" title="New chat (⌘⇧O)">${ic("plus")}<span class="grow">New chat</span><kbd data-kbd="⌘⇧O">⌘⇧O</kbd></a>
        <button class="nav-item" data-action="cmdk" title="Search (⌘K)">${ic("search")}<span class="grow">Search</span><kbd data-kbd="⌘K">⌘K</kbd></button>
      </div>
      <div class="sidebar-section" data-od-id="sidebar-workspace">
        <p class="sidebar-label">Workspace</p>
        ${WORKSPACE.map(([id, label, icon, href, extra]) => `<a class="nav-item" href="${href}" title="${label}"${id === cur ? ' aria-current="page"' : ""}>${ic(icon)}<span class="grow">${label}</span>${extra}</a>`).join("")}
      </div>
      <div class="sidebar-scroll" data-od-id="sidebar-recents">
        <p class="sidebar-label">Recent</p>
        ${RECENT.map(([t, live]) => `<a class="nav-item thread-item" href="chat-workspace.html">${live ? '<span class="dot" aria-label="Run in progress"></span>' : ""}<span class="grow">${t}</span></a>`).join("")}
      </div>
      <div class="sidebar-bottom" data-od-id="sidebar-account">
        <a class="nav-item" href="models-and-providers.html" title="Settings (⌘,)">${ic("gear")}<span class="grow">Settings</span><kbd data-kbd="⌘,">⌘,</kbd></a>
        <div class="account"><span class="avatar">TJ</span><span class="who"><b>Travis James</b><span><i class="status-dot"></i>Runtime healthy · 1.0.0</span></span></div>
      </div>`;
  }

  const emit = (el, name, detail) => el.dispatchEvent(new CustomEvent(name, { detail, bubbles: true }));

  /* ─── switches & segmented choices ─────────────────────────────────── */
  document.addEventListener("click", (e) => {
    const sw = e.target.closest(".switch[role=switch]");
    if (sw) {
      e.stopPropagation(); e.preventDefault();
      if (sw.hasAttribute("disabled") || sw.getAttribute("aria-disabled") === "true") return;
      const on = sw.getAttribute("aria-checked") !== "true"; sw.setAttribute("aria-checked", String(on));
      const row = sw.closest(".rule"); if (row && row.dataset.dims !== undefined) row.classList.toggle("off", !on);
      emit(sw, "uar:switch", { on, id: sw.dataset.id || sw.getAttribute("aria-label") }); return;
    }
    const ch = e.target.closest(".choice [role=radio]");
    if (ch && !ch.disabled) {
      const group = ch.parentElement; group.querySelectorAll("[role=radio]").forEach((r) => r.setAttribute("aria-checked", String(r === ch)));
      emit(group, "uar:choice", { value: ch.dataset.value || ch.textContent.trim(), label: ch.textContent.trim(), id: group.id });
    }
  }, true);
  document.addEventListener("keydown", (e) => {
    if ((e.key === " " || e.key === "Enter") && e.target.matches(".switch[role=switch]")) { e.preventDefault(); e.target.click(); }
    const ch = e.target.closest && e.target.closest(".choice [role=radio]");
    if (ch && (e.key === "ArrowRight" || e.key === "ArrowLeft")) {
      e.preventDefault(); const all = [...ch.parentElement.querySelectorAll("[role=radio]:not(:disabled)")]; const i = all.indexOf(ch);
      const n = all[(i + (e.key === "ArrowRight" ? 1 : all.length - 1)) % all.length]; n.focus(); n.click();
    }
  });

  /* ─── detail tabs: [data-dtab] buttons switch [data-dtabpanel] in the same .detail ── */
  document.addEventListener("click", (e) => {
    const t = e.target.closest("[data-dtab]"); if (!t) return;
    const scope = t.closest(".detail") || document;
    scope.querySelectorAll("[data-dtab]").forEach((x) => x.setAttribute("aria-selected", String(x === t)));
    scope.querySelectorAll("[data-dtabpanel]").forEach((p) => p.classList.toggle("active", p.dataset.dtabpanel === t.dataset.dtab));
  });

  /* ─── master–detail: .srow[data-src] selects; shows .view[data-view] if one matches ── */
  document.addEventListener("click", (e) => {
    const row = e.target.closest(".srow[data-src]"); if (!row || e.target.closest(".switch")) return;
    const list = row.closest(".sources"); if (!list) return;
    list.querySelectorAll(".srow[data-src]").forEach((x) => x.setAttribute("aria-current", String(x === row)));
    const md = row.closest(".md") || document; const id = row.dataset.src; const view = row.dataset.view || id;
    const views = md.querySelectorAll(".view[data-view]");
    if ([...views].some((v) => v.dataset.view === view)) views.forEach((v) => v.classList.toggle("active", v.dataset.view === view));
    emit(row, "uar:source", { id, view });
  });

  /* ─── list filter: input[data-filter="#selector"] hides rows by text ── */
  document.addEventListener("input", (e) => {
    const f = e.target.closest("input[data-filter]"); if (!f) return; const q = f.value.trim().toLowerCase();
    document.querySelectorAll(f.dataset.filter).forEach((r) => { r.style.display = !q || r.textContent.toLowerCase().includes(q) ? "" : "none"; });
  });

  /* ─── mobile nav button + Esc back to chat ─────────────────────────── */
  window.addEventListener("DOMContentLoaded", () => {
    const btn = document.getElementById("mobile-nav-btn"); if (!btn) return;
    const mq = window.matchMedia("(max-width: 760px)"); const sync = () => { btn.style.display = mq.matches ? "" : "none"; }; sync(); mq.addEventListener("change", sync);
  });
  document.addEventListener("keydown", (e) => {
    if (e.key !== "Escape" || e.defaultPrevented || !nav) return;
    const S = window.UARShell; if (!S) return;
    if (S.state.panelOpen) { S.closePanel(); return; }
    if (document.querySelector(".cmdk.open, .menu.open") || document.querySelector(".app.sidebar-mobile-open")) return;
    if (e.target.matches && e.target.matches("input, textarea, select")) return;
    location.href = "chat-workspace.html";
  });
})();
