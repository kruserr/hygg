<p align="center">
  <a href="https://github.com/kruserr/hygg" target="_blank">
    <img width="300" src="https://raw.githubusercontent.com/kruserr/hygg/main/assets/logo/logo.svg">
  </a>
</p>

# hygg
Simplifying the way you read

## Overview
The goal of this project is to build an ebook and document reader that has a minimal set of features, that make reading enjoyable on a desktop computer.

A large emphasis is on making a minimalistic and distraction free environment for you to easily focus on what is important, the content.

Furthermore we are working on building a seamless experience for reading ebooks but also more generally documents, both on a desktop computer and a tablet or e-reader with a browser.

## Features
- CLI client
  - Converts regular or scanned PDF or EPUB to plain text
  - Justifies the plain text to specified column width
  - Horizontally centers the text
  - Minimalistic less like interactive reader with vim like bindings (still work in progress)
  - Saves progress
  - Cross platform
  - Each component in the CLI client is exposed as a UNIX style utility for easy code reuse in your own open source project

## Quick start guide
### Install the CLI client
```sh
cargo install --locked hygg
hygg doc.pdf
```

for scanned document support
```sh
sudo apt install ocrmypdf tesseract-ocr-eng
```

then use the `--ocr=true` flag
```sh
hygg --ocr=true doc.pdf
```

For further install instructions read the [Getting started page](https://github.com/kruserr/hygg/blob/main/docs/pages/getting-started.md)

## Documentation
Visit the [Documentation](https://github.com/kruserr/hygg/blob/main/docs/README.md)

## Roadmap
- [x] Plain text format support
- [x] PDF format support
- [x] EPUB format support
- [x] Convert scanned documents and images to plain text with ocrmypdf
- [x] Auto saving progress
- [x] Integrated command line with vim like commands
- [x] Server to sync progress and books
- [x] Text selection and yanking
- [ ] Execute commands from yanked text
- [ ] Text highlighting with server sync
- [ ] Offline PWA web client
- [ ] Start screen to show overview of books and progress
- [ ] Full test suite
- [ ] CI / CD pipelines
- [ ] Minimal build feature flag, for only basic converters, no server integration and no ai
- [ ] Image to ascii art converter
- [ ] Natural sounding ai voice model for text to speech narration
- [ ] Run all inference directly in rust no external runtime deps
- [ ] Support more ebook and document formats
