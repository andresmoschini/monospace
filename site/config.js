// site/config.js
// Centralized configuration for Monospace website links, versions, and downloads.
//
// ============================================================================
// HOW TO EDIT THIS FILE
// ============================================================================
// 1. Version: Update the version string when a new release is cut.
// 2. Download URLs: Replace each "https://example.com/TODO-..." placeholder with
//    the direct download link for that platform's release archive or installer.
// 3. crates.io: Replace the placeholder with the published package page.
//
// Placeholder detection:
// While an entry is empty, begins with "TODO", or starts with "https://example.com/TODO",
// the site marks that download button as "Coming soon" and disables clicks, preventing
// confusing 404 navigation. Once you supply a real URL, the button activates automatically.
// ============================================================================

window.MONOSPACE_CONFIG = {
  version: "0.1.0",
  repoUrl: "https://github.com/andresmoschini/monospace",
  cratesIoUrl: "https://example.com/TODO-crates-io",
  downloads: {
    windows: {
      url: "https://example.com/TODO-download-windows",
      filename: "monospace-windows-x86_64.zip",
      label: "Download for Windows",
      badge: "x86_64 (.exe / .zip)",
    },
    macos: {
      url: "https://example.com/TODO-download-macos",
      filename: "monospace-macos-universal.tar.gz",
      label: "Download for macOS",
      badge: "Universal binary (.tar.gz)",
    },
    linux: {
      url: "https://example.com/TODO-download-linux",
      filename: "monospace-linux-x86_64.tar.gz",
      label: "Download for Linux",
      badge: "x86_64 (.tar.gz)",
    },
  },
};

(function () {
  function isPlaceholder(url) {
    if (!url || typeof url !== "string") return true;
    var trimmed = url.trim();
    return (
      trimmed === "" ||
      trimmed.startsWith("TODO") ||
      trimmed.startsWith("https://example.com/TODO") ||
      trimmed.startsWith("http://example.com/TODO")
    );
  }

  function applyConfig() {
    var cfg = window.MONOSPACE_CONFIG || {};

    // Version label
    var versionEls = document.querySelectorAll("[data-config='version']");
    versionEls.forEach(function (el) {
      el.textContent = cfg.version || "0.1.0";
    });

    // crates.io link
    var cratesLink = document.querySelector("[data-config='crates-link']");
    var cratesStatus = document.querySelector("[data-config='crates-status']");
    if (cratesLink) {
      if (isPlaceholder(cfg.cratesIoUrl)) {
        cratesLink.setAttribute("aria-disabled", "true");
        cratesLink.classList.add("disabled");
        cratesLink.removeAttribute("href");
        cratesLink.setAttribute("tabindex", "-1");
        if (cratesStatus) {
          cratesStatus.textContent = "Coming soon";
          cratesStatus.classList.add("badge-pending");
        }
      } else {
        cratesLink.removeAttribute("aria-disabled");
        cratesLink.classList.remove("disabled");
        cratesLink.setAttribute("href", cfg.cratesIoUrl);
        cratesLink.removeAttribute("tabindex");
        if (cratesStatus) {
          cratesStatus.textContent = "Available on crates.io";
          cratesStatus.classList.remove("badge-pending");
          cratesStatus.classList.add("badge-active");
        }
      }
    }

    // Platform downloads
    var platforms = ["windows", "macos", "linux"];
    platforms.forEach(function (platform) {
      var entry = (cfg.downloads && cfg.downloads[platform]) || {};
      var button = document.querySelector("[data-download-platform='" + platform + "']");
      var statusBadge = document.querySelector("[data-download-status='" + platform + "']");

      if (!button) return;

      if (isPlaceholder(entry.url)) {
        button.setAttribute("aria-disabled", "true");
        button.classList.add("disabled");
        button.removeAttribute("href");
        button.setAttribute("tabindex", "-1");
        if (statusBadge) {
          statusBadge.textContent = "Coming soon";
          statusBadge.classList.add("badge-pending");
        }
      } else {
        button.removeAttribute("aria-disabled");
        button.classList.remove("disabled");
        button.setAttribute("href", entry.url);
        button.removeAttribute("tabindex");
        if (statusBadge) {
          statusBadge.textContent = "Available";
          statusBadge.classList.remove("badge-pending");
          statusBadge.classList.add("badge-active");
        }
      }
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", applyConfig);
  } else {
    applyConfig();
  }
})();
