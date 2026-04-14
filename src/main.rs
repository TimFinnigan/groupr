use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: feg <directory> [--dry-run]");
        eprintln!("  Groups loose files in <directory> into subfolders by extension.");
        eprintln!("  --dry-run  Show what would happen without moving any files.");
        process::exit(1);
    }

    let target_dir = Path::new(&args[1]);
    let dry_run = args.iter().any(|a| a == "--dry-run");

    if !target_dir.exists() {
        eprintln!("Error: '{}' does not exist.", target_dir.display());
        process::exit(1);
    }
    if !target_dir.is_dir() {
        eprintln!("Error: '{}' is not a directory.", target_dir.display());
        process::exit(1);
    }

    match group_files(target_dir, dry_run) {
        Ok(moved) => {
            if moved == 0 {
                println!("No loose files found — nothing to do.");
            } else if dry_run {
                println!("\nDry run: {} file(s) would be moved.", moved);
            } else {
                println!("\nDone: {} file(s) moved.", moved);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

/// Returns the number of files moved (or that would be moved in dry-run mode).
fn group_files(dir: &Path, dry_run: bool) -> Result<usize, String> {
    // Collect all direct-child files (not directories).
    let mut by_ext: HashMap<String, Vec<PathBuf>> = HashMap::new();

    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Cannot read directory '{}': {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Error reading entry: {}", e))?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        // Skip hidden files (e.g. .DS_Store, .gitignore)
        let filename = entry.file_name();
        if filename.to_string_lossy().starts_with('.') {
            continue;
        }

        let ext = extension_key(&path);
        by_ext.entry(ext).or_default().push(path);
    }

    if by_ext.is_empty() {
        return Ok(0);
    }

    let mut total_moved = 0;

    let mut ext_keys: Vec<&String> = by_ext.keys().collect();
    ext_keys.sort(); // deterministic output

    for ext in ext_keys {
        let files = &by_ext[ext];
        let dest_dir = dir.join(&ext);

        if !dry_run && !dest_dir.exists() {
            fs::create_dir(&dest_dir)
                .map_err(|e| format!("Cannot create '{}': {}", dest_dir.display(), e))?;
        }

        for src in files {
            let filename = src.file_name().expect("file has a name");
            let dest = dest_dir.join(filename);

            if dry_run {
                println!(
                    "[dry-run] {} -> {}/",
                    src.display(),
                    dest_dir.display()
                );
            } else {
                // If a file with that name already exists in the destination,
                // add a numeric suffix to avoid clobbering it.
                let dest = unique_dest(&dest);
                fs::rename(src, &dest)
                    .map_err(|e| format!("Cannot move '{}': {}", src.display(), e))?;
                println!("{} -> {}", src.display(), dest.display());
            }
            total_moved += 1;
        }
    }

    Ok(total_moved)
}

/// Returns the extension as a lower-case string, or "no_extension" if absent.
fn extension_key(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_else(|| "no_extension".to_string())
}

/// If `path` already exists, append _1, _2, … to the stem until a free name is found.
fn unique_dest(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();
    let parent = path.parent().unwrap_or(Path::new("."));

    let mut counter = 1u32;
    loop {
        let candidate = parent.join(format!("{}_{}{}", stem, counter, ext));
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    // Bring in tempfile only for tests
    fn make_temp_dir() -> TempDir {
        tempfile::tempdir().expect("create temp dir")
    }

    #[test]
    fn extension_key_lowercase() {
        assert_eq!(extension_key(Path::new("photo.PNG")), "png");
        assert_eq!(extension_key(Path::new("doc.PDF")), "pdf");
        assert_eq!(extension_key(Path::new("README")), "no_extension");
    }

    #[test]
    fn groups_files_by_extension() {
        let tmp = make_temp_dir();
        let dir = tmp.path();

        File::create(dir.join("a.png")).unwrap();
        File::create(dir.join("b.png")).unwrap();
        File::create(dir.join("c.jpg")).unwrap();
        File::create(dir.join("notes.txt")).unwrap();

        let moved = group_files(dir, false).unwrap();
        assert_eq!(moved, 4);

        assert!(dir.join("png").join("a.png").exists());
        assert!(dir.join("png").join("b.png").exists());
        assert!(dir.join("jpg").join("c.jpg").exists());
        assert!(dir.join("txt").join("notes.txt").exists());
    }

    #[test]
    fn dry_run_moves_nothing() {
        let tmp = make_temp_dir();
        let dir = tmp.path();

        File::create(dir.join("a.png")).unwrap();

        let moved = group_files(dir, true).unwrap();
        assert_eq!(moved, 1);
        // File must still be in its original location
        assert!(dir.join("a.png").exists());
        assert!(!dir.join("png").exists());
    }

    #[test]
    fn hidden_files_are_skipped() {
        let tmp = make_temp_dir();
        let dir = tmp.path();

        File::create(dir.join(".DS_Store")).unwrap();
        File::create(dir.join(".gitignore")).unwrap();
        File::create(dir.join("visible.txt")).unwrap();

        let moved = group_files(dir, false).unwrap();
        assert_eq!(moved, 1); // only visible.txt
        assert!(dir.join(".DS_Store").exists());
        assert!(dir.join(".gitignore").exists());
    }

    #[test]
    fn no_files_returns_zero() {
        let tmp = make_temp_dir();
        // Put a subdirectory in — it should be ignored
        fs::create_dir(tmp.path().join("subdir")).unwrap();

        let moved = group_files(tmp.path(), false).unwrap();
        assert_eq!(moved, 0);
    }

    #[test]
    fn collision_avoidance() {
        let tmp = make_temp_dir();
        let dir = tmp.path();

        // Pre-create the destination dir with an existing file
        fs::create_dir(dir.join("png")).unwrap();
        File::create(dir.join("png").join("photo.png")).unwrap();
        // Loose file with the same name
        File::create(dir.join("photo.png")).unwrap();

        group_files(dir, false).unwrap();

        assert!(dir.join("png").join("photo.png").exists());
        assert!(dir.join("png").join("photo_1.png").exists());
    }
}
