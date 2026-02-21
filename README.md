# epub2braille

> Converts EPUB books to binary Braille format compatible with embossing devices.

![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-purple)

## Overview

**epub2braille** is a desktop application that converts `.epub` digital books into binary Braille format (`.bin`) ready for use with Braille embossers. The conversion follows the Spanish Braille standard (CBE/ONCE), including proper syllabification, capitalization prefixes, and 30-cell line formatting.

Simply drag and drop your `.epub` file onto the app window — no configuration needed.

![Demo](https://via.placeholder.com/600x300?text=drag+%26+drop+demo)

## Features

- Drag & drop interface — no command line required
- Full Spanish Braille encoding (CBE/ONCE standard)
- Accented characters and special Spanish characters (á, é, í, ó, ú, ñ, ü)
- Automatic syllabification with hyphenation rules
- Uppercase and number prefix support
- 30-cell line formatting with proper word wrapping
- Output saved alongside the original file

## Download

Head to the [Releases](../../releases) page and download the installer for your platform:

| Platform | File |
|----------|------|
| macOS    | `.dmg` |
| Windows  | `.msi` or `.exe` |
| Linux    | `.AppImage` or `.deb` |

## Building from source

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (LTS)
- [pnpm](https://pnpm.io/)
- Tauri system dependencies for your platform → [see Tauri docs](https://v2.tauri.app/start/prerequisites/)

### Steps

```bash
git clone https://github.com/EmanuelFonseca2023/epub2braille.git
cd epub2braille
pnpm install
cargo tauri build
```

## How it works

1. The app reads the `.epub` file (which is a ZIP archive containing XHTML files)
2. Extracts text content from the spine in reading order
3. Encodes each character into its Braille cell byte using the CBE/ONCE mapping
4. Applies syllabification rules for proper line-break hyphenation
5. Formats the output into 30-cell lines with control bytes
6. Writes the result as a `.bin` file in the same directory as the input

## Output format

Each line in the output binary consists of:
- 30 Braille cell bytes (one byte per cell, bit-encoded)
- 1 control byte for line break

Unused cells at the end of a line are padded with `0x00`.

## Tech stack

- **Rust** — Braille encoding engine, EPUB parsing
- **Tauri 2** — desktop app framework
- **SvelteKit** — frontend UI
- **zip** — EPUB extraction
- **scraper** — HTML/XHTML text extraction

## License

MIT © Emanuel Fonseca

