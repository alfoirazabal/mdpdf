# MDPDF

**Convert Markdown to beautifully styled PDFs — and extract the source back out.**

MDPDF is a fast, self-contained CLI tool written in Rust that converts Markdown files into styled PDFs using headless Chrome for rendering. It ships with 11 built-in visual templates and supports custom CSS, KaTeX math rendering, and PDF metadata embedding. Every PDF it produces carries the original Markdown file embedded inside it, so you can always recover the source.

---

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Usage](#usage)
  - [render — Convert Markdown to PDF](#render--convert-markdown-to-pdf)
  - [extract — Recover Markdown from PDF](#extract--recover-markdown-from-pdf)
- [Templates](#templates)
- [Custom Templates](#custom-templates)
- [PDF Metadata](#pdf-metadata)
- [Options Reference](#options-reference)
- [How It Works](#how-it-works)
- [Building from Source](#building-from-source)
- [Dependencies](#dependencies)

---

## Features

- Converts any Markdown file to a styled, print-ready PDF
- 11 built-in templates across mobile, tablet, widescreen, and A4 print layouts — each available in light and dark variants
- Custom CSS template support for full visual control
- KaTeX math rendering built in — no external CDN required
- Markdown extensions: tables, footnotes, and strikethrough
- Optional raw HTML passthrough — keep `<div>`, `<span>`, custom tags, and inline styles intact in the output
- Embedded source: the original `.md` file is embedded inside the output PDF
- Bidirectional workflow: recover the source Markdown from any MDPDF-generated PDF
- Custom PDF metadata (Title, Author, Subject, Keywords, or any key)
- Adjustable PDF scale factor
- Single-page output mode: measures rendered content and sets the PDF page to fit it exactly — no clipping, no scrolling void
- Optional intermediate HTML output for debugging your styles
- Zero-margin PDF output with full background color support

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
cd mdpdf
cargo build --release
```

The compiled binary will be at `target/release/mdpdf`.

---

## Usage

MDPDF has two subcommands: `render` and `extract`.

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

MDPDF renders the document in Chrome, measures the actual content dimensions, and sets the PDF page size to match exactly — width and height hug the content with no clipping and no empty space below. Wide tables expand the page width rather than getting cut off. The template width is always respected; `--one-page` only adjusts the height to fit.

`--one-page` and `--scale` work together: scale is applied first, then the page dimensions are measured from the scaled output.

```sh
mdpdf render document.md --output document.pdf --one-page --scale 0.8
```

---

### `extract` — Recover Markdown from PDF

Any PDF produced by MDPDF contains the original Markdown file embedded inside it. Use the `extract` command to recover it.

```sh
mdpdf extract <INPUT> --output <OUTPUT>
```

**Example:**

```sh
mdpdf extract document.pdf --output recovered.md
```

This writes the embedded Markdown source to `recovered.md`. If the PDF was not created by MDPDF, the command exits with an error.

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

If the built-in templates do not fit your needs, you can supply your own CSS file with `--custom-template-path`. The CSS is injected into the `<head>` of the rendered HTML document and controls all visual styling.

```sh
mdpdf render document.md --output document.pdf --custom-template-path ./custom.css
```

A good starting point is to export the intermediate HTML with `--ghtml`, inspect the DOM structure, and write your CSS against it.

When `--custom-template-path` is set, `--template` is ignored.

---

## PDF Metadata

You can embed metadata into the output PDF using the `--cm` flag. Each value must follow the format `key=value`. The flag can be repeated to set multiple fields.

```sh
mdpdf render paper.md --output paper.pdf \
  --cm "Title=My Research Paper" \
  --cm "Author=Jane Smith" \
  --cm "Subject=Computer Science" \
  --cm "Keywords=Rust, PDF, CLI"
```

Standard PDF metadata keys are `Title`, `Author`, `Subject`, and `Keywords`, but any custom key is accepted. Values are stored as UTF-16 strings for full Unicode support.

MDPDF also automatically sets the `Creator` metadata field to `MDPDF v<version>`.

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
| `--one-page`                |       | Fit the entire document into a single PDF page by measuring rendered content dimensions. Compatible with `--scale`. | `false` |
| `--cm <KEY=VALUE>`          |       | Add a custom metadata field. Repeatable                                     |              |

### `extract`

| Flag / Argument       | Short | Description                                      |              |
|-----------------------|-------|--------------------------------------------------|--------------|
| `<INPUT>`             |       | Path to an MDPDF-generated PDF file              | *(required)* |
| `--output <OUTPUT>`   | `-o`  | Path for the recovered Markdown file             | *(required)* |

---

## How It Works

MDPDF processes a document through a well-defined pipeline:

1. **Read** — The input `.md` file is read from disk.
2. **Parse** — The Markdown is parsed to HTML using [comrak](https://github.com/kivikakk/comrak) with tables, footnotes, and strikethrough enabled. By default, raw HTML tags and unsafe link schemes embedded in the source are stripped. Passing `--allow-html` disables that sanitisation and lets them through as-is.
3. **Template** — The HTML body is wrapped in a full HTML document that includes the selected CSS template and the KaTeX math rendering library (CSS and JS are bundled into the binary at compile time, so no network access is needed).
4. **Render** — A temporary HTML file is written to disk and opened in headless Chrome via [headless_chrome](https://github.com/rust-headless-chrome/rust-headless-chrome). Chrome prints the page to PDF with zero margins, full background printing, and an auto-generated document outline. If `--one-page` is set, the rendered content dimensions (`scrollWidth` × `scrollHeight`) are measured via JavaScript before printing and passed as the paper size, so the PDF page hugs the content exactly.
5. **Embed** — The original `.md` source file is embedded into the PDF as an attached file using [lopdf](https://github.com/J-F-Liu/lopdf). Custom metadata fields are written to the PDF's Info dictionary.
6. **Clean up** — The temporary HTML file is deleted (unless `--ghtml` was passed).

The `extract` command reverses step 5: it reads the PDF's embedded file tree, locates the attached Markdown file, and writes it back to disk.

---

## Building from Source

Requires Rust 1.85 or later (edition 2024).

```sh
git clone https://github.com/alfoirazabal/mdpdf.git
cd mdpdf
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

KaTeX (CSS and JS) is bundled directly into the binary at compile time via `include_str!`.

---

## Author

Alfonso Irazabal Levy

---

## License

GPLv3. Attribution is appreciated but not required beyond what the license mandates.

See [LICENSE](LICENSE) for details.