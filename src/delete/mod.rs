use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use std::io;
use std::path::PathBuf;

pub fn run(path_input: &str) {
    // Strip trailing backslashes and stray quotes caused by Windows argv parsing
    // (a trailing `\` in a quoted shell argument like `"foo\"` makes the Windows
    // argument parser treat `\"` as an escaped quote, injecting a `"` into the value).
    let path_str = path_input.trim_end_matches(|c: char| c == '\\' || c == '"');
    let mut path = PathBuf::from(path_str);

    // On Windows, convert to UNC path to handle reserved device names and long paths
    #[cfg(windows)]
    {
        if !path_str.starts_with(r"\\?\") {
            // canonicalize resolves `.`/`..` and returns a \\?\ UNC path on Windows.
            // The \\?\ prefix bypasses normalization, so we must canonicalize first —
            // otherwise a path like `.\folder` becomes `\\?\C:\cwd\.\folder` which
            // Windows cannot find.
            match std::fs::canonicalize(&path) {
                Ok(canonical) => path = canonical,
                Err(_) => {
                    // Path doesn't exist; make absolute so the metadata error is clear.
                    if !path.is_absolute() {
                        path = std::env::current_dir().unwrap_or_default().join(&path);
                    }
                    let path_string = path.to_string_lossy().replace("/", "\\");
                    path = PathBuf::from(format!(r"\\?\{}", path_string));
                }
            }
        }
    }

    let metadata = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: Cannot access path '{}': {}", path_str, e);
            return;
        }
    };

    let path_type = if metadata.is_file() {
        "file"
    } else if metadata.is_dir() {
        "directory"
    } else {
        "path"
    };

    println!("Found {} at: {}", path_type, path_str);

    let confirmation = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Are you sure you want to delete this {}?",
            path_type
        ))
        .default(false)
        .interact();

    match confirmation {
        Ok(true) => {
            let result = if metadata.is_file() {
                std::fs::remove_file(&path)
            } else if metadata.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Unsupported path type",
                ))
            };

            match result {
                Ok(_) => println!("Successfully deleted: {}", path_str),
                Err(e) => eprintln!("Failed to delete {}: {}", path_str, e),
            }
        }
        Ok(false) => {
            println!("Deletion cancelled.");
        }
        Err(e) => {
            eprintln!("Error during confirmation: {}", e);
        }
    }
}
