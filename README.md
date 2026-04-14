# file-extension-grouper

A fast, zero-dependency Rust CLI that organizes loose files in a folder by moving them into subfolders named after their file extension.

```
Downloads/
  photo.PNG      →  Downloads/png/photo.png
  resume.PDF     →  Downloads/pdf/resume.pdf
  notes.txt      →  Downloads/txt/notes.txt
  archive.tar.gz →  Downloads/gz/archive.tar.gz
  README         →  Downloads/no_extension/README
```

## Installation

**Prerequisites:** [Rust](https://rustup.rs) 1.56+

```bash
git clone https://github.com/timfinnigan/file-extension-grouper.git
cd file-extension-grouper
cargo install --path .
```

The binary is installed as `groupr`.

## Usage

```
groupr <directory> [--dry-run]
```

| Argument | Description |
|---|---|
| `<directory>` | Path to the folder to organize |
| `--dry-run` | Preview what would be moved without touching any files |

### Examples

```bash
# Organize your Downloads folder
groupr ~/Downloads

# Preview first
groupr ~/Downloads --dry-run

# Organize the current directory
groupr .
```

### Sample output

```
/Users/tim/Downloads/photo.PNG -> /Users/tim/Downloads/png/photo.PNG
/Users/tim/Downloads/resume.pdf -> /Users/tim/Downloads/pdf/resume.pdf
/Users/tim/Downloads/notes.txt -> /Users/tim/Downloads/txt/notes.txt

Done: 3 file(s) moved.
```

## Behavior

- **Only top-level files are moved** — existing subfolders and their contents are never touched.
- **Extensions are lowercased** — `Photo.PNG` goes into `png/`, not `PNG/`.
- **Files without an extension** go into a `no_extension/` folder.
- **Collision avoidance** — if `png/photo.png` already exists at the destination, the incoming file is saved as `png/photo_1.png`, `png/photo_2.png`, etc.
- **Destination folders are created automatically** if they don't already exist.

## Development

```bash
cargo build           # debug build
cargo build --release # optimized build → target/release/groupr
cargo test            # run unit tests
```

## License

MIT
