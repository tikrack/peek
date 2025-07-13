# peek

`peek` is a modern, customizable, and extensible `ls` replacement written in Rust. It supports file listing, tree view, size formatting, filtering, and more — with beautiful color output.

## ✨ Features

- 📁 Basic file listing (like `ls`)
- 🎨 Colored output with custom folder color (hex)
- 📦 File size display (`--size` or `-s`)
- 🧾 Long format listing (`--long` or `-l`) with permissions, size, UID, GID, and modified time
- 🌲 Tree view (`--tree` or `-t`) with optional `--depth N`
- 👻 Show hidden files (`--all` or `-a`)
- 🔍 Glob pattern filtering (`--pattern` or `-p`)
- 🧠 Persistent color configuration saved in `~/.peekconfig`

## 🧪 Usage

### Basic Listing

```sh
peek
```

### Show file sizes

```sh
peek --size
peek -s
```

### Long listing

```sh
peek --long
peek -l
```

### Tree view

```sh
peek --tree
peek -t
```

### Tree with depth

```sh
peek -t --depth 2
```

### Show hidden files

```sh
peek --all
peek -a
```

### Set custom folder color

```sh
peek --dir-color FF00FF
peek --dir-color "#00FFFF"
```

(This is saved persistently in `~/.peekconfig`)

### Pattern matching (glob)

```sh
peek -p "*.rs"           # match all .rs files in current dir
peek -t -p "**/*.rs"     # match recursively all .rs files
peek -a -p "*lib*"       # match files with 'lib' in the name
```

## 🛠 Installation

1. Make sure you have Rust installed:
   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone and build:
   ```sh
   git clone https://github.com/yourname/peek.git
   cd peek
   cargo build --release
   ```

3. Optionally copy the binary to your bin directory:
   ```sh
   cp target/release/peek ~/.local/bin/
   ```

## 📂 Configuration

Configuration is stored in a JSON file at:

```
~/.peekconfig
```

Example content:

```json
{
  "dir_color": "FF00FF"
}
```

## 🔮 Coming Soon

- Recursive search (`--recursive`)
- Output to JSON format
- Sort by size/date/name

## 📄 License

MIT
