// site/config.js
// Centralized configuration for Monospace website links, versions, and downloads.
//
// ============================================================================
// HOW TO EDIT THIS FILE
// ============================================================================
// 1. Version: Update the version string and releaseUrl when a new release is cut.
// 2. Download URLs: Point each entry (or a "https://example.com/TODO-..." placeholder) at
//    the direct download link for that platform's release archive or installer.
// 3. crates.io: Replace the placeholder with the published package page.
//
// Placeholder detection:
// While an entry is empty, begins with "TODO", or starts with "https://example.com/TODO",
// the site marks that download button as "Coming soon" and disables clicks, preventing
// confusing 404 navigation. Once you supply a real URL, the button activates automatically.
// ============================================================================

window.MONOSPACE_CONFIG = {
  version: "0.1.1",
  repoUrl: "https://github.com/andresmoschini/monospace",
  sponsorUrl: "https://github.com/sponsors/andresmoschini",
  koFiUrl: "https://ko-fi.com/andresmoschini",
  cratesIoUrl: "https://crates.io/crates/monospace",
  releaseUrl: "https://github.com/andresmoschini/monospace/releases/tag/v0.1.1",
  // Keyed by the value of each button's data-download-platform attribute. A card's
  // data-download-status badge reads "Available" once any of its buttons has a real URL.
  downloads: {
    windows: {
      url: "https://github.com/andresmoschini/monospace/releases/download/v0.1.1/monospace-v0.1.1-x86_64-pc-windows-msvc.zip",
    },
    "macos-arm64": {
      url: "https://github.com/andresmoschini/monospace/releases/download/v0.1.1/monospace-v0.1.1-aarch64-apple-darwin.tar.gz",
    },
    "macos-x86_64": {
      url: "https://github.com/andresmoschini/monospace/releases/download/v0.1.1/monospace-v0.1.1-x86_64-apple-darwin.tar.gz",
    },
    "linux-x86_64": {
      url: "https://github.com/andresmoschini/monospace/releases/download/v0.1.1/monospace-v0.1.1-x86_64-unknown-linux-gnu.tar.gz",
    },
    "linux-arm64": {
      url: "https://github.com/andresmoschini/monospace/releases/download/v0.1.1/monospace-v0.1.1-aarch64-unknown-linux-gnu.tar.gz",
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
      el.textContent = cfg.version || "0.1.1";
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

    // Release page link
    document.querySelectorAll("[data-config='release-link']").forEach(function (el) {
      if (!isPlaceholder(cfg.releaseUrl)) el.setAttribute("href", cfg.releaseUrl);
    });

    // Sponsor and Ko-fi links
    document.querySelectorAll("[data-config='sponsor-link']").forEach(function (el) {
      if (!isPlaceholder(cfg.sponsorUrl)) el.setAttribute("href", cfg.sponsorUrl);
    });
    document.querySelectorAll("[data-config='ko-fi-link']").forEach(function (el) {
      if (!isPlaceholder(cfg.koFiUrl)) el.setAttribute("href", cfg.koFiUrl);
    });

    // Platform downloads
    var available = {};
    document.querySelectorAll("[data-download-platform]").forEach(function (button) {
      var key = button.getAttribute("data-download-platform");
      var platform = key.split("-")[0];
      var entry = (cfg.downloads && cfg.downloads[key]) || {};
      if (isPlaceholder(entry.url)) {
        button.setAttribute("aria-disabled", "true");
        button.classList.add("disabled");
        button.removeAttribute("href");
        button.setAttribute("tabindex", "-1");
        if (!(platform in available)) available[platform] = false;
      } else {
        button.removeAttribute("aria-disabled");
        button.classList.remove("disabled");
        button.setAttribute("href", entry.url);
        button.removeAttribute("tabindex");
        available[platform] = true;
      }
    });
    document.querySelectorAll("[data-download-status]").forEach(function (badge) {
      if (available[badge.getAttribute("data-download-status")]) {
        badge.textContent = "Available";
        badge.classList.remove("badge-pending");
        badge.classList.add("badge-active");
      } else {
        badge.textContent = "Coming soon";
        badge.classList.add("badge-pending");
      }
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", applyConfig);
  } else {
    applyConfig();
  }
})();
