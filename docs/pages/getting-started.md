### [<-](../README.md)

## Getting Started
### Install the Rust toolchain
For UNIX type operating systems run the following command:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

For further install instructions, read the Rust docs https://www.rust-lang.org/learn/get-started

### Install and run with cargo
```sh
cargo install --locked hygg
hygg doc.pdf
```

If the `hygg` binary executable is not found, checkout how to add `~/.cargo/bin` to your path.

e.g. for the fish shell you add the following to your config:

~/.config/fish/config.fish
```fish
fish_add_path ~/.cargo/bin
```

### Git and cargo
Clone the repo, build from source and run
```sh
git clone https://github.com/kruserr/hygg.git
cd hygg
cargo run -- test-data/pdf/pdfreference1.7old-1-50.pdf
```

Clone the repo, build from source, install and run
```sh
git clone https://github.com/kruserr/hygg.git
cd hygg
cargo install --locked --path hygg
hygg test-data/pdf/pdfreference1.7old-1-50.pdf
```

### Using with pandoc for additional formats
Hygg supports PDF and EPUB files natively. For other document formats (DOCX, ODT, RTF, HTML, Markdown, etc.), you can use pandoc:

#### Install pandoc on Debian 12
```sh
sudo apt install pandoc
```

#### Usage examples
With pandoc installed, hygg can automatically handle many more formats:
```sh
# Direct file reading (pandoc will be used automatically)
hygg document.docx
hygg presentation.pptx
hygg webpage.html
hygg article.md

# Explicit conversion using pipes
pandoc document.docx --to=plain | hygg

# Reading from stdin (already plain text)
echo "Hello, World!" | hygg
cat readme.txt | hygg
```

When pandoc is not installed, hygg will provide a helpful message suggesting installation for unsupported formats.
