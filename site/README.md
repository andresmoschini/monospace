# Monospace Website

This directory contains the source for the static landing page for Monospace, published to GitHub
Pages at <https://andresmoschini.github.io/monospace/>.

The site is built with pure HTML5, CSS3, and vanilla JavaScript. It requires zero build steps, zero
Node or bundler dependencies, and works out of the box when opened directly in any browser.

---

## 1. Directory structure

```text
site/
├── assets/                                 # Media assets (PNG screenshots and animated GIFs)
│   ├── demo-connectors.gif                 # Connector auto-routing demonstration
│   ├── demo-edit-text-resize.gif           # In-place text editing and resizing demo
│   ├── demo-undo-menu.gif                  # Menu navigation and undo/redo history demo
│   ├── hero.png                            # Terminal showcase canvas (hero visual)
│   ├── screenshot-diagram-edit-menu.png    # Interactive editor interface preview
│   ├── screenshot-diagram-note.png         # Architecture diagram with notes preview
│   ├── screenshot-sequence-diagram-selection.png # Sequence diagram preview
│   └── screenshot-showcase.png             # Stroke and shape showcase
├── config.js                               # Centralized link, version, and download configuration
├── index.html                              # Main landing page markup
├── README.md                               # Site maintainer and editing documentation
├── robots.txt                              # Crawler policy and sitemap location
├── sitemap.xml                             # The page's single URL, for search engines
└── style.css                               # Mobile-first stylesheet with CSS custom properties
```

---

## 2. Updating download URLs and crates.io links

All dynamic and release-dependent URLs are centralized in [`site/config.js`](./config.js). **Do not
hardcode release URLs inside `index.html`.**

Open `site/config.js` and edit the values in `window.MONOSPACE_CONFIG`:

```javascript
window.MONOSPACE_CONFIG = {
  version: "0.1.1", // Change when cutting a release
  repoUrl: "https://github.com/andresmoschini/monospace",
  sponsorUrl: "https://github.com/sponsors/andresmoschini", // Sponsor button and footer link
  koFiUrl: "https://ko-fi.com/andresmoschini", // Ko-fi link in the footer
  cratesIoUrl: "https://crates.io/crates/monospace", // Replace with real crates.io URL
  releaseUrl: "https://github.com/andresmoschini/monospace/releases/tag/v0.1.0", // Checksums link
  // One entry per download button, keyed by its data-download-platform attribute.
  downloads: {
    windows: {
      url: "https://github.com/andresmoschini/monospace/releases/download/v0.1.0/monospace-v0.1.0-x86_64-pc-windows-msvc.zip",
    },
    "macos-arm64": {
      url: "https://github.com/andresmoschini/monospace/releases/download/v0.1.0/monospace-v0.1.0-aarch64-apple-darwin.tar.gz",
    },
    "macos-x86_64": {
      url: "https://github.com/andresmoschini/monospace/releases/download/v0.1.0/monospace-v0.1.0-x86_64-apple-darwin.tar.gz",
    },
    "linux-x86_64": {
      url: "https://github.com/andresmoschini/monospace/releases/download/v0.1.0/monospace-v0.1.0-x86_64-unknown-linux-gnu.tar.gz",
    },
    "linux-arm64": {
      url: "https://github.com/andresmoschini/monospace/releases/download/v0.1.0/monospace-v0.1.0-aarch64-unknown-linux-gnu.tar.gz",
    },
  },
};
```

### Placeholder behavior

If a download or crates.io URL is empty, starts with `TODO`, or begins with
`https://example.com/TODO`:

- The button displays as disabled with a `"Coming soon"` badge.
- Clicking does nothing, avoiding deceptive 404 links.
- Replacing the placeholder with a valid URL automatically activates the button without needing
  changes to the HTML.

---

## 3. Editing content and updating media assets

### Modifying copy and sections

All copy lives directly in [`site/index.html`](./index.html), organized into semantic sections:

- `#about`: What Monospace is, why ASCII matters, and the Spec-Driven Development philosophy.
- `#features`: Three feature highlights paired with animated GIFs.
- `#gallery`: Showcase grid with modal zoom overlay triggers.
- `#download`: Platform cards for pre-compiled binaries and crates.io instructions.
- `#faq`: Common questions about ASCII and box-drawing diagramming.
- `#support`: Visual sponsorship cards for GitHub Sponsors and Ko-fi (with direct on-page checkout).
- `footer`: Links to repository, issue tracker, crates.io, GitHub Sponsors, Ko-fi, and license.

### Adding or replacing screenshots and GIFs

1. Place new media into [`site/assets/`](./assets/). Use descriptive, stable lowercase filenames.
2. Keep raw quality intact (do not apply aggressive lossy compression that alters crisp character
   glyphs).
3. Update the `src` and `alt` attributes in `index.html`.
4. If updating the gallery, ensure the `data-lightbox` and `data-caption` attributes on the parent
   button point to the updated asset and carry an accurate description.

---

## 4. Previewing locally

Because the site is pure static files, you can preview it locally in several ways:

### Option A: Open directly in your browser

Double-click `site/index.html` or run:

```sh
# Windows PowerShell
Start-Process "site/index.html"
```

### Option B: Local HTTP server

Run a lightweight HTTP server from the repository root:

```sh
# Using Python
python -m http.server 8000 --directory site

# Or using Node / npx
npx serve site
```

Then visit <http://localhost:8000> in your browser. Verify layout responsiveness by testing in both
mobile and desktop viewport widths.

---

## 5. Deployment to GitHub Pages

The site is automatically deployed to GitHub Pages via the GitHub Actions workflow in
[`.github/workflows/pages.yml`](../.github/workflows/pages.yml).

### Triggers

- Automatic: Every push to the `main` branch that touches `site/**` or
  `.github/workflows/pages.yml`.
- Manual: Via `workflow_dispatch` on the GitHub Actions tab.

### Deployment status

Check deployment runs under the **Actions** tab on GitHub:
<https://github.com/andresmoschini/monospace/actions/workflows/pages.yml>.

### Enabling GitHub Pages (if not already enabled)

GitHub Pages deployment requires the repository Pages source to be set to **GitHub Actions**:

1. Navigate to repository **Settings** → **Pages** (under "Code and automation").
2. Under **Build and deployment** → **Source**, select **GitHub Actions**.
3. Once configured, workflow runs will deploy directly to
   <https://andresmoschini.github.io/monospace/>.

---

## 6. Search engine and social metadata

The page carries the metadata search engines and social networks read:

- `title`, `meta name="description"` and `link rel="canonical"` name the page and its one public
  URL.
- Open Graph and Twitter tags use absolute URLs, so the preview image resolves wherever the link is
  shared.
- Two `application/ld+json` blocks describe the project as a `SoftwareApplication` and answer four
  `FAQPage` questions. Keep each answer in the JSON-LD identical to the visible FAQ section.
- `sitemap.xml` lists the page, and `robots.txt` points at it.

GitHub Pages serves this directory under `/monospace/`, so the `robots.txt` here is not the host's
root `robots.txt` (`https://andresmoschini.github.io/robots.txt`) and crawlers are not required to
read it. Submit `https://andresmoschini.github.io/monospace/sitemap.xml` in Google Search Console
and Bing Webmaster Tools for the sitemap to be discovered; the remaining tags work without that
step.

The social preview image is `assets/hero.png`, declared at its real size. A dedicated 1200×630 card
would fit social networks better; if one is added, update the Open Graph and Twitter image tags with
its absolute URL and dimensions.

## 7. Customizing styling and branding

Theme variables are declared at the top of [`site/style.css`](./style.css) under `:root` and
`@media (prefers-color-scheme: light)`:

- **Accent color:** Modify `--accent` (default: `#e5a83b` phosphor amber), `--accent-hover`, and
  `--accent-dim`.
- **Typography:**
  - `--font-mono`: Used for headings, buttons, badges, code blocks, and terminal headers.
  - `--font-sans`: Used for body copy and paragraphs.
- **Layout width:** Adjust `--content-max-width` (default: `1040px`).
- **Dark & light modes:** Update the colors in the `@media (prefers-color-scheme: light)` block to
  tune daytime contrast.
