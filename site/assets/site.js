// Progressive enhancement for the site: theme toggle, before/after slider and the download links.
// Without JavaScript the page still works: the download buttons point to the releases page.
(function () {
  "use strict";

  // ---- Theme toggle -------------------------------------------------------------------------------------------
  function currentTheme() {
    return document.documentElement.getAttribute("data-theme") === "light" ? "light" : "dark";
  }
  function applyTheme(theme) {
    if (theme === "light") document.documentElement.setAttribute("data-theme", "light");
    else document.documentElement.removeAttribute("data-theme");
  }
  function initTheme() {
    var button = document.querySelector("[data-theme-toggle]");
    if (!button) return;
    button.addEventListener("click", function () {
      var next = currentTheme() === "light" ? "dark" : "light";
      applyTheme(next);
      try {
        localStorage.setItem("theme", next);
      } catch (e) {
        /* storage blocked: the choice just lasts for this page */
      }
    });
    // Follow the system as long as the visitor has not chosen.
    var query = window.matchMedia("(prefers-color-scheme: light)");
    query.addEventListener("change", function (event) {
      var stored = null;
      try {
        stored = localStorage.getItem("theme");
      } catch (e) {
        /* ignore */
      }
      if (!stored) applyTheme(event.matches ? "light" : "dark");
    });
  }

  // ---- Before/after slider ------------------------------------------------------------------------------------
  function initCompare() {
    var box = document.querySelector("[data-compare]");
    if (!box) return;
    var range = box.querySelector(".compare-range");
    function update() {
      box.style.setProperty("--pos", range.value + "%");
    }
    range.addEventListener("input", update);
    update();
  }

  // ---- Downloads ----------------------------------------------------------------------------------------------
  function detectPlatform() {
    var ua = navigator.userAgent.toLowerCase();
    var data = navigator.userAgentData;
    var platform = ((data && data.platform) || navigator.platform || "").toLowerCase();
    if (/android|iphone|ipad|ipod/.test(ua)) return "mobile";
    if (platform.indexOf("mac") === 0 || ua.indexOf("macintosh") >= 0) {
      // iPadOS pretends to be a Mac but has a touch screen.
      return navigator.maxTouchPoints > 1 ? "mobile" : "mac";
    }
    if (platform.indexOf("win") === 0 || ua.indexOf("windows") >= 0) return "windows";
    if (platform.indexOf("linux") >= 0 || ua.indexOf("linux") >= 0 || ua.indexOf("cros") >= 0) return "linux";
    return "unknown";
  }

  var PLATFORM_NAMES = { mac: "macOS", windows: "Windows", linux: "Linux" };
  var KIND_PATTERNS = {
    dmg: /\.dmg$/i,
    exe: /\.exe$/i,
    appimage: /\.appimage$/i,
    deb: /\.deb$/i,
  };
  var KIND_LABELS = { dmg: "the .dmg", exe: "the installer", appimage: "the AppImage", deb: "the .deb" };

  function archLabel(name) {
    if (/universal/i.test(name)) return "universal";
    if (/aarch64|arm64/i.test(name)) return "ARM64";
    if (/x64|amd64|x86_64/i.test(name)) return "64-bit Intel/AMD";
    return "";
  }
  function formatSize(bytes) {
    return bytes > 0 ? (bytes / 1048576 >= 10 ? Math.round(bytes / 1048576) : (bytes / 1048576).toFixed(1)) + " MB" : "";
  }

  function fetchLatestRelease(repo) {
    var key = "latest-release:" + repo;
    try {
      var cached = JSON.parse(sessionStorage.getItem(key) || "null");
      if (cached && Date.now() - cached.time < 10 * 60 * 1000) return Promise.resolve(cached.release);
    } catch (e) {
      /* no cache */
    }
    return fetch("https://api.github.com/repos/" + repo + "/releases/latest", {
      headers: { Accept: "application/vnd.github+json" },
    }).then(function (response) {
      if (!response.ok) throw new Error("GitHub answered " + response.status);
      return response.json().then(function (release) {
        try {
          sessionStorage.setItem(key, JSON.stringify({ time: Date.now(), release: release }));
        } catch (e) {
          /* no cache */
        }
        return release;
      });
    });
  }

  function fillButtons(release) {
    var assets = release.assets || [];
    Object.keys(KIND_PATTERNS).forEach(function (kind) {
      var holder = document.querySelector('[data-kind="' + kind + '"]');
      if (!holder) return;
      var matches = assets.filter(function (asset) {
        return KIND_PATTERNS[kind].test(asset.name);
      });
      if (matches.length === 0) return; // keep the fallback link to the releases page
      var primary = holder.querySelector(".btn").classList.contains("primary");
      holder.textContent = "";
      matches.forEach(function (asset) {
        var arch = matches.length > 1 ? archLabel(asset.name) : "";
        var link = document.createElement("a");
        link.className = "btn" + (primary ? " primary" : "");
        link.href = asset.browser_download_url;
        link.textContent = "Download " + KIND_LABELS[kind] + (arch ? " (" + arch + ")" : "");
        var file = document.createElement("span");
        file.className = "file";
        file.textContent = asset.name + (asset.size ? " · " + formatSize(asset.size) : "");
        var wrap = document.createElement("div");
        wrap.appendChild(link);
        wrap.appendChild(file);
        holder.appendChild(wrap);
      });
    });
    var info = document.querySelector("[data-release]");
    if (info) {
      var version = String(release.tag_name || "").replace(/^v/, "");
      var date = release.published_at ? new Date(release.published_at).toLocaleDateString("en", { year: "numeric", month: "long", day: "numeric" }) : "";
      info.textContent = "";
      info.appendChild(document.createTextNode("Latest version " + version + (date ? ", released " + date : "") + ". "));
      var notes = document.createElement("a");
      notes.href = release.html_url;
      notes.textContent = "Release notes";
      info.appendChild(notes);
      info.appendChild(document.createTextNode("."));
    }
  }

  function initDownloads() {
    var platform = detectPlatform();
    var name = PLATFORM_NAMES[platform];
    if (name) {
      var card = document.querySelector('[data-platform="' + platform + '"]');
      if (card) {
        card.setAttribute("data-recommended", "");
        card.querySelector(".badge").hidden = false;
      }
      var hero = document.querySelector("[data-hero-download]");
      if (hero) hero.textContent = "Download for " + name;
      var hint = document.querySelector("[data-hero-hint]");
      if (hint) hint.textContent = "Free and open source · also available for the other systems";
    } else if (platform === "mobile") {
      var mobileHint = document.querySelector("[data-hero-hint]");
      if (mobileHint) mobileHint.textContent = "Siderea Planets is a desktop app. Download it on your Mac, Windows or Linux computer.";
    }

    var repo = document.body.getAttribute("data-repo");
    if (!repo || !window.fetch) return;
    fetchLatestRelease(repo)
      .then(fillButtons)
      .catch(function () {
        var info = document.querySelector("[data-release]");
        if (info) info.textContent = "The newest version could not be looked up. The buttons lead to the releases page.";
      });
  }

  initTheme();
  initCompare();
  initDownloads();
})();
