# MDPDF

**Convert Markdown to beautifully styled PDFs — and extract the source back out.**

MDPDF is a fast, self-contained tool written in Rust that converts Markdown files into styled PDFs using headless Chrome for rendering. It works both as a CLI tool and as a desktop GUI application (powered by Tauri). It ships with 11 built-in visual templates and supports custom CSS, KaTeX math rendering, and PDF metadata embedding. Every PDF it produces carries the original Markdown source and the CSS template embedded inside it, so you can always recover both.

---

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Usage](#usage)
  - [GUI Mode](#gui-mode)
  - [render — Convert Markdown to PDF](#render--convert-markdown-to-pdf)
  - [extract — Recover Markdown from PDF](#extract--recover-markdown-from-pdf)
- [Templates](#templates)
- [Custom Templates](#custom-templates)
- [PDF Metadata](#pdf-metadata)
- [Configuration](#configuration)
- [Options Reference](#options-reference)
- [How It Works](#how-it-works)
- [Building from Source](#building-from-source)
- [Dependencies](#dependencies)

---

## Features

- **Dual interface:** desktop GUI and full-featured CLI in a single binary
- Converts any Markdown file to a styled, print-ready PDF
- 11 built-in templates across mobile, tablet, widescreen, and A4 print layouts — each available in light and dark variants
- Custom CSS template support for full visual control — live CSS editor in the GUI
- KaTeX math rendering built in — no external CDN required
- Markdown extensions: tables, footnotes, and strikethrough
- Optional raw HTML passthrough — keep `<div>`, `<span>`, custom tags, and inline styles intact in the output
- Embedded source: the original `.md` file **and** the CSS template are embedded inside the output PDF by default — disable with `--no-embed-css` (CLI) or the **Embed CSS template in PDF** checkbox (GUI)
- Bidirectional workflow: recover the Markdown source and the CSS template from any MDPDF-generated PDF; older PDFs without an embedded CSS are handled gracefully
- Custom PDF metadata (Title, Author, Subject, Keywords, or any key) — GUI offers preset fields
- Adjustable PDF scale factor
- Single-page output mode: measures rendered content and sets the PDF page to fit it exactly — no clipping, no scrolling void
- Manual page break mode: suppress all automatic Chrome page breaks and break only on explicit CSS indicators in the content
- Optional intermediate HTML output for debugging your styles
- Zero-margin PDF output with full background color support
- Configuration persistence: auto-saves settings to platform config directory, plus manual import/export
- Works fully offline — all assets are bundled at compile time

---

## Requirements

- **Google Chrome** (or Chromium) must be installed and accessible on your system. MDPDF uses it in headless mode to render the PDF.
- Rust toolchain (only required if building from source)

---

## Installation

### Pre-built binary

Download the latest release for your platform from the [Releases](../../releases) page and place the binary somewhere on your `PATH`.

```sh
# Example on Linux/macOS
chmod +x mdpdf
mv mdpdf /usr/local/bin/
```

### Build from source

```sh
git clone https://github.com/alfoirazabal/mdpdf.git
cd mdpdf/mdpdf
cargo build --release
```

The compiled binary will be at `target/release/mdpdf`.

---

## Usage

MDPDF automatically launches in **GUI mode** when run without arguments, or in **CLI mode** when invoked with a subcommand (`render` or `extract`).

On Windows, launching the GUI (e.g. by double-clicking the executable) does not spawn a console window.

---

### GUI Mode

Simply run `mdpdf` with no arguments to open the desktop application.

```sh
mdpdf
```

The GUI provides:

- **Two-column layout** — Left panel for Files, PDF Metadata, and Options; right panel for the live CSS editor
- **Template picker** — Select from 11 built-in templates; the CSS editor auto-populates with the template's styles for customization
- **PDF Metadata editor** — Dropdown with common preset fields (Title, Author, Subject, Keywords, Creator, Producer) plus custom key support
- **Embed CSS option** — "Embed CSS template in PDF" checkbox in the Options panel (checked by default); uncheck to omit the CSS from the embedded files in the output PDF
- **Extract tab** — Choose an input PDF, specify the output Markdown path, and optionally override the output CSS path (auto-filled as `<markdown_stem>.css`); the log shows whether a CSS template was also recovered
- **Progress indicator** — Always-visible bottom bar with real-time rendering/extraction progress and action buttons
- **Configuration persistence** — Settings auto-save to the platform config directory; Import/Export buttons allow sharing configs as JSON files

Output file paths: if you specify only a filename (without a directory), the output is placed in the same directory as the input file.

---

### `render` — Convert Markdown to PDF

```sh
mdpdf render <INPUT> --output <OUTPUT> [OPTIONS]
```

**Minimal example:**

```sh
mdpdf render document.md --output document.pdf
```

This reads `document.md`, applies the default A4 template, and writes `document.pdf`. The `.pdf` extension is added automatically if omitted from the output name.

**With a built-in template:**

```sh
mdpdf render document.md --output document.pdf --template widescreen-dark
```

**With a custom CSS file:**

```sh
mdpdf render document.md --output document.pdf --custom-template-path ./my-styles.css
```

**With custom metadata and a scale adjustment:**

```sh
mdpdf render report.md --output report.pdf \
  --template print-a4 \
  --cm "Author=Jane Smith" \
  --cm "Subject=Q3 Financial Report" \
  --scale 0.9
```

**Keep the intermediate HTML file** (useful for tweaking your template):

```sh
mdpdf render document.md --output document.pdf --ghtml
```

This produces both `document.pdf` and `document.pdf.html` so you can inspect exactly what Chrome is rendering.

**Disable CSS template embedding:**

```sh
mdpdf render document.md --output document.pdf --no-embed-css
```

By default, both the Markdown source and the CSS template are embedded inside the output PDF. Pass `--no-embed-css` to omit the CSS. The Markdown source is always embedded regardless of this flag. PDFs rendered with `--no-embed-css` will report "No CSS template was embedded in this PDF" when extracted.

**Allow raw HTML elements in your Markdown source:**

```sh
mdpdf render document.md --output document.pdf --allow-html
```

By default, comrak strips any HTML tags embedded in your Markdown — `<div>`, `<span>`, inline `style` attributes, custom elements, and so on. Passing `--allow-html` lets them through to the rendered output unchanged.

> **Note:** Because of how comrak works internally, this flag also permits unsafe link schemes such as `javascript:` and `data:` — the two cannot be separated at the library level. Only use `--allow-html` with Markdown source you control and trust.

**Fit the entire document to a single PDF page:**

```sh
mdpdf render document.md --output document.pdf --one-page
```

MDPDF reads the `@page` margin from the active template, measures the rendered content dimensions, and sets the PDF page to exactly fit the content plus its margins — no clipping, no empty space, no multiple pages. Wide tables push the page width out rather than getting cut off. The template's original margin and visual style are preserved unchanged.

`--one-page` and `--scale` are fully compatible: scale is applied first by Chrome, then dimensions are measured from the scaled result.

```sh
mdpdf render document.md --output document.pdf --one-page --scale 0.8
```

**Control page breaks manually:**

```sh
mdpdf render document.md --output document.pdf --manual-breaks
```

By default, Chrome breaks pages automatically based on the template's `@page` size. With `--manual-breaks`, all automatic page breaks are suppressed. Pages break only where the Markdown source contains an explicit CSS break indicator, for example:

```html
<div style="break-after: page"></div>
```

The template `@page` size, margins, and all other visual styling are preserved exactly as normal. `--allow-html` is not required for this to work — the break indicator is just an inline style on an HTML element that can be embedded directly in the Markdown source.

`--manual-breaks` and `--one-page` are mutually exclusive. Passing both together exits with an error.

---

### `extract` — Recover Markdown from PDF

Any PDF produced by MDPDF contains the original Markdown source embedded inside it. PDFs produced with CSS embedding enabled (the default) also carry the CSS template. Use the `extract` command to recover them.

```sh
mdpdf extract <INPUT> --output <OUTPUT> [--css-output <CSS_OUTPUT>]
```

**Example:**

```sh
mdpdf extract document.pdf --output recovered.md
```

This writes the embedded Markdown source to `recovered.md`. The CSS template is simultaneously extracted to `recovered.css` (the default — same stem as `--output`, `.css` extension). The command reports what was extracted:

```
Extracted Markdown → recovered.md
Extracted CSS template → recovered.css
```

If the PDF was rendered with `--no-embed-css`, or was produced by an older version of MDPDF that did not embed the CSS, the Markdown is still extracted and the output notes:

```
Extracted Markdown → recovered.md
Note: No CSS template was embedded in this PDF.
```

**Override the CSS output path:**

```sh
mdpdf extract document.pdf --output recovered.md --css-output styles/custom.css
```

If the PDF was not created by MDPDF the command exits with an error.

---

## Templates

MDPDF ships 11 built-in templates. Pass the template name to `--template` using the kebab-case identifier below.

| Template name           | Description                                                       |
|-------------------------|-------------------------------------------------------------------|
| `mobile-dark`           | Dark theme sized for mobile phone screens                         |
| `mobile-light`          | Light theme sized for mobile phone screens                        |
| `tablet-dark`           | Dark theme sized for tablet screens                               |
| `tablet-light`          | Light theme sized for tablet screens                              |
| `widescreen-dark`       | Dark theme for laptops and desktops                               |
| `widescreen-light`      | Light theme for laptops and desktops                              |
| `widescreen-cut-dark`   | Dark widescreen theme with cutout for browser and taskbar chrome  |
| `widescreen-cut-light`  | Light widescreen theme with cutout for browser and taskbar chrome |
| `print-a4-dark`         | Dark theme formatted for A4 paper printing                        |
| `print-a4-light`        | Light theme formatted for A4 paper printing                       |
| `print-a4`              | Clean white A4 theme for standard printing (default)              |

When no `--template` flag is provided, `print-a4` is used.

---

## Custom Templates

If the built-in templates do not fit your needs, you can supply your own CSS file with `--custom-template-path` (CLI) or edit the CSS directly in the GUI editor.

```sh
mdpdf render document.md --output document.pdf --custom-template-path ./custom.css
```

A good starting point is to export the intermediate HTML with `--ghtml`, inspect the DOM structure, and write your CSS against it.

When `--custom-template-path` is set, `--template` is ignored.

In the GUI, select any built-in template to populate the editor with its CSS, then modify it freely — the editor content is what gets rendered.

---

## PDF Metadata

You can embed metadata into the output PDF using the `--cm` flag (CLI) or the PDF Metadata section (GUI). Each value must follow the format `key=value`. The flag can be repeated to set multiple fields.

```sh
mdpdf render paper.md --output paper.pdf \
  --cm "Title=My Research Paper" \
  --cm "Author=Jane Smith" \
  --cm "Subject=Computer Science" \
  --cm "Keywords=Rust, PDF, CLI"
```

Standard PDF metadata keys are `Title`, `Author`, `Subject`, and `Keywords`, but any custom key is accepted. Values are stored as UTF-16 strings for full Unicode support.

In the GUI, a dropdown provides quick access to common metadata fields (Title, Author, Subject, Keywords, Creator, Producer) and also allows adding custom keys.

MDPDF also automatically sets the `Creator` metadata field to `MDPDF v<version>`.

---

## Configuration

MDPDF automatically saves your GUI settings (template, CSS, options, and metadata) to the platform-appropriate configuration directory:

| Platform | Config path                                    |
|----------|------------------------------------------------|
| Linux    | `~/.config/mdpdf/config.json`                  |
| macOS    | `~/Library/Application Support/mdpdf/config.json` |
| Windows  | `%APPDATA%\mdpdf\config.json`                  |

Settings are restored automatically when the application starts.

You can also manually export and import configurations as JSON files using the **Import** and **Export** buttons in the GUI header. This is useful for sharing rendering presets across machines or team members.

---

## Options Reference

### `render`

| Flag / Argument             | Short | Description                                                                 | Default      |
|-----------------------------|-------|-----------------------------------------------------------------------------|--------------|
| `<INPUT>`                   |       | Path to the input Markdown file                                             | *(required)* |
| `--output <OUTPUT>`         | `-o`  | Path for the output PDF file (`.pdf` extension added if missing)            | *(required)* |
| `--template <TEMPLATE>`     | `-t`  | Built-in template name (see [Templates](#templates))                        | `print-a4`   |
| `--custom-template-path`    | `-c`  | Path to a custom CSS file. Overrides `--template` when set                  |              |
| `--scale <SCALE>`           | `-s`  | PDF scale factor. Must be between `0.1` and `2.0`                          | `1.0`        |
| `--ghtml`                   |       | Keep the intermediate HTML file alongside the PDF output                    | `false`      |
| `--allow-html`              |       | Pass raw HTML elements in the Markdown source through to the output. Also permits unsafe link schemes (`javascript:`, `data:`). Use only with trusted input. | `false` |
| `--one-page`                |       | Fit the entire document into a single PDF page by measuring rendered content dimensions. Compatible with `--scale`. Mutually exclusive with `--manual-breaks`. | `false` |
| `--manual-breaks`           |       | Suppress automatic page breaks. Break only on explicit CSS indicators in the content (e.g. `break-after: page`). Mutually exclusive with `--one-page`. | `false` |
| `--cm <KEY=VALUE>`          |       | Add a custom metadata field. Repeatable                                     |              |
| `--no-embed-css`            |       | Omit the CSS template from the PDF's embedded files. The Markdown source is always embedded. | `false` |

### `extract`

| Flag / Argument             | Short | Description                                                                 |              |
|-----------------------------|-------|-----------------------------------------------------------------------------|--------------|
| `<INPUT>`                   |       | Path to an MDPDF-generated PDF file                                         | *(required)* |
| `--output <OUTPUT>`         | `-o`  | Path for the recovered Markdown file                                        | *(required)* |
| `--css-output <CSS_OUTPUT>` |       | Path for the recovered CSS file. Defaults to `<output_stem>.css`            |              |

---

## How It Works

MDPDF processes a document through a well-defined pipeline:

1. **Read** — The input `.md` file is read from disk.
2. **Parse** — The Markdown is parsed to HTML using [comrak](https://github.com/kivikakk/comrak) with tables, footnotes, and strikethrough enabled. By default, raw HTML tags and unsafe link schemes embedded in the source are stripped. Passing `--allow-html` disables that sanitisation and lets them through as-is.
3. **Template** — The HTML body is wrapped in a full HTML document that includes the selected CSS template and the KaTeX math rendering library (CSS and JS are bundled into the binary at compile time, so no network access is needed).
4. **Render** — A temporary HTML file is written to disk and opened in headless Chrome via [headless_chrome](https://github.com/rust-headless-chrome/rust-headless-chrome). Chrome prints the page to PDF with zero margins, full background printing, and an auto-generated document outline. If `--one-page` is set, the pipeline first reads the `@page` margin values from the loaded template stylesheet via JavaScript, then measures the rendered content's `scrollWidth` and `scrollHeight`, adds the margins to both dimensions, injects a `@page` size override to suppress page fragmentation, and passes the result as `paper_width`/`paper_height` to Chrome — producing a single page that fits the content exactly with the template's original spacing intact. If `--manual-breaks` is set, a style is injected that suppresses all automatic `break-before` and `break-after` behaviour on every element that does not carry an explicit inline break rule, leaving author-specified page breaks intact while preventing Chrome from inserting its own.
5. **Embed** — The original `.md` source file is embedded into the PDF as an attached file using [lopdf](https://github.com/J-F-Liu/lopdf). The active CSS template is also embedded by default (pass `--no-embed-css` to skip it). Custom metadata fields are written to the PDF's Info dictionary.
6. **Clean up** — The temporary HTML file is deleted (unless `--ghtml` was passed).

The `extract` command reverses step 5: it reads the PDF's embedded file tree, locates the attached Markdown source and writes it to disk, then attempts to locate the embedded CSS template and writes it alongside (defaulting to `<output_stem>.css`). If the CSS was not embedded (older PDFs or PDFs rendered with `--no-embed-css`), the Markdown is still recovered and the output notes that no CSS was found.

---

## Building from Source

Requires Rust 1.85 or later (edition 2024).

```sh
git clone https://github.com/alfoirazabal/mdpdf.git
cd mdpdf/mdpdf
cargo build --release
```

The release profile is optimized for minimal binary size: `opt-level = "z"`, LTO enabled, single codegen unit, panic = abort, and symbols stripped.

Run the tests:

```sh
cargo test
```

---

## Dependencies

| Crate              | Purpose                                              |
|--------------------|------------------------------------------------------|
| `comrak`           | Markdown parsing and HTML generation                 |
| `headless_chrome`  | Headless Chrome control for HTML-to-PDF rendering    |
| `lopdf`            | PDF manipulation, file embedding, and metadata       |
| `clap`             | Command-line argument parsing                        |
| `tokio`            | Async runtime for the Chrome rendering step          |
| `anyhow`           | Ergonomic error handling                             |
| `url`              | File path to `file://` URL conversion                |
| `serde_json`       | Parsing JS-evaluated content dimensions in one-page mode |
| `tauri`            | Desktop GUI framework                                |
| `tauri-plugin-dialog` | Native file/save dialogs for the GUI              |
| `dirs`             | Platform-appropriate config directory resolution     |

KaTeX (CSS and JS) is bundled directly into the binary at compile time via `include_str!`.

---

## Author

Alfonso Irazabal Levy

---

## License

GPLv3. Attribution is appreciated but not required beyond what the license mandates.

See [LICENSE](LICENSE) for details.
