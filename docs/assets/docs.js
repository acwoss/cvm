(() => {
  const NAV = [
    {
      title: "Start here",
      items: [
        { href: "index.html", label: "Overview", id: "overview" },
        { href: "getting-started.html", label: "Getting started", id: "getting-started" },
        { href: "concepts.html", label: "Core concepts", id: "concepts" },
      ],
    },
    {
      title: "Reference",
      items: [
        { href: "commands.html", label: "Command reference", id: "commands" },
        { href: "examples.html", label: "Examples", id: "examples" },
      ],
    },
    {
      title: "How-to guides",
      items: [
        { href: "how-to/create-and-open.html", label: "Create & open", id: "hto-create-open" },
        { href: "how-to/inherit-skills.html", label: "Inherit global skills", id: "hto-inherit" },
        { href: "how-to/share-team-setup.html", label: "Share a team setup", id: "hto-share" },
        { href: "how-to/auto-activate.html", label: "Auto-activate with .cvm", id: "hto-auto" },
        { href: "how-to/parallel-sessions.html", label: "Parallel sessions", id: "hto-parallel" },
        { href: "how-to/per-env-secrets.html", label: "Per-env secrets", id: "hto-secrets" },
        { href: "how-to/statusline.html", label: "Statusline badge", id: "hto-statusline" },
      ],
    },
  ];

  const PAGE = document.body.dataset.page || "";
  const DEPTH = Number(document.body.dataset.depth || "0");
  const prefix = DEPTH > 0 ? "../".repeat(DEPTH) : "";
  const assetPrefix = prefix;

  function applyTheme(theme) {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("cvm-theme", theme);
  }

  function initTheme() {
    const theme = localStorage.getItem("cvm-theme") || "dark";
    applyTheme(theme);
    const btn = document.getElementById("theme-toggle");
    if (btn) {
      btn.addEventListener("click", () => {
        const next = document.documentElement.getAttribute("data-theme") === "dark" ? "light" : "dark";
        applyTheme(next);
      });
    }
  }

  function renderSidebar() {
    const host = document.getElementById("docs-sidebar");
    if (!host) return;
    const nav = document.createElement("nav");
    nav.className = "docs-nav";
    NAV.forEach((group) => {
      const wrap = document.createElement("div");
      wrap.className = "docs-nav-group";
      wrap.innerHTML = `<div class="docs-nav-title">${group.title}</div>`;
      group.items.forEach((item) => {
        const a = document.createElement("a");
        a.href = prefix + item.href;
        a.textContent = item.label;
        if (item.id === PAGE) a.classList.add("active");
        wrap.appendChild(a);
      });
      nav.appendChild(wrap);
    });
    host.appendChild(nav);
  }

  function initMenu() {
    const btn = document.getElementById("menu-btn");
    const sidebar = document.getElementById("docs-sidebar");
    if (!btn || !sidebar) return;
    btn.addEventListener("click", () => sidebar.classList.toggle("open"));
    document.addEventListener("click", (e) => {
      if (window.innerWidth > 900) return;
      if (!sidebar.classList.contains("open")) return;
      if (sidebar.contains(e.target) || btn.contains(e.target)) return;
      sidebar.classList.remove("open");
    });
  }

  function wireTopLinks() {
    const home = document.getElementById("link-home");
    const github = document.getElementById("link-github");
    if (home) home.href = assetPrefix + "../index.html";
    if (github) github.href = "https://github.com/acwoss/cvm";
  }

  initTheme();
  renderSidebar();
  initMenu();
  wireTopLinks();
})();
