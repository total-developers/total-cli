mod args;

use args::TotalArgs;
use clap::Parser;
use std::process::Command;

mod scaffolding;
use std::io;

use scaffolding::create_vue_scaffold;
use scaffolding::create_rust_scaffold;
use dialoguer::theme::ColorfulTheme;

fn check_dlltool() {
    if which::which("dlltool.exe").is_err() {
        eprintln!("Warning: 'dlltool.exe' not found in PATH. Some build steps (especially for Windows targets) may fail. \
If you are building for Windows and need MinGW tools, please install them and ensure 'dlltool.exe' is in your PATH.");
    }
}

fn main() {
    check_dlltool();
    let args: TotalArgs = TotalArgs::parse();
    match &args.entity_type {
        args::EntityType::Create(project) => {
            let PROJECT_LANGUAGE: String = project.language.to_lowercase();
            let PROJECT_TITLE: String = project.title.to_lowercase();
            

            match PROJECT_LANGUAGE.as_str(){
                "rust" => {
                    println!("Creating a program named: {:?}, in {:?}", PROJECT_TITLE, PROJECT_LANGUAGE);
                    create_rust_scaffold(&PROJECT_TITLE)
                },
                "vue" => {
                    println!("Creating a program named: {:?}, in {:?}", PROJECT_TITLE, PROJECT_LANGUAGE);
                    create_vue_scaffold(&PROJECT_TITLE);
                },

                _ => println!("Invalid"),
            }
        }
        args::EntityType::Delete(project) => {
            use std::path::PathBuf;
            use dialoguer::Confirm;

            let path_str = &project.path;
            let mut path = PathBuf::from(path_str);

            // On Windows, convert to UNC path to handle reserved device names and long paths
            #[cfg(windows)]
            {
                if !path_str.starts_with(r"\\?\") {
                    // Make path absolute if it isn't already
                    let absolute_path = if path.is_absolute() {
                        path
                    } else {
                        std::env::current_dir()
                            .unwrap_or_default()
                            .join(&path)
                    };

                    // Convert to string and add UNC prefix
                    let path_string = absolute_path.to_string_lossy().replace("/", "\\");
                    path = PathBuf::from(format!(r"\\?\{}", path_string));
                }
            }

            // Try to get metadata to check if path exists and determine type
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
                .with_prompt(format!("Are you sure you want to delete this {}?", path_type))
                .default(false)
                .interact();

            match confirmation {
                Ok(true) => {
                    let result = if metadata.is_file() {
                        std::fs::remove_file(&path)
                    } else if metadata.is_dir() {
                        std::fs::remove_dir_all(&path)
                    } else {
                        Err(io::Error::new(io::ErrorKind::Other, "Unsupported path type"))
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
        args::EntityType::Run(run) => {
            let lang = run.path.to_lowercase();
            match lang.as_str() {
                "rust" => {
                    println!("Running Rust project with 'cargo run'...");
                    let status = Command::new("cargo")
                        .arg("run")
                        .status()
                        .expect("Failed to run cargo");
                    if !status.success() {
                        eprintln!("Failed to run Rust project.");
                    }
                }
                "vue" => {
                    println!("Running Vue project with 'npm run serve'...");
                    let status = Command::new("npm")
                        .arg("run")
                        .arg("serve")
                        .status()
                        .expect("Failed to run npm");
                    if !status.success() {
                        eprintln!("Failed to run Vue project.");
                    }
                }
                "php" => {
                    use std::path::Path;
                    // Laravel with Vue: check for both artisan and package.json
                    let is_laravel = Path::new("artisan").exists();
                    let is_vue = Path::new("package.json").exists()
                        && std::fs::read_to_string("package.json")
                            .map(|c| c.contains("\"vue\""))
                            .unwrap_or(false);

                    if is_laravel && is_vue {
                        println!("Detected Laravel project with Vue frontend.");
                        println!("Starting backend: 'php artisan serve'...");
                        let backend = Command::new("php")
                            .arg("artisan")
                            .arg("serve")
                            .spawn();
                        match backend {
                            Ok(mut child) => {
                                println!("Starting frontend: 'npm run dev'...");
                                let frontend = Command::new("npm.cmd")
                                    .arg("run")
                                    .arg("dev")
                                    .spawn();
                                match frontend {
                                    Ok(mut child2) => {
                                        let _ = child.wait();
                                        let _ = child2.wait();
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to run Vue frontend: {}", e);
                                        let _ = child.kill();
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to run Laravel backend: {}", e);
                            }
                        }
                    } else if is_laravel {
                        println!("Detected Laravel project. Running 'php artisan serve'...");
                        let status = Command::new("php")
                            .arg("artisan")
                            .arg("serve")
                            .status()
                            .expect("Failed to run php artisan serve");
                        if !status.success() {
                            eprintln!("Failed to run Laravel project.");
                        }
                    } else {
                        println!("Running PHP project with 'php -S localhost:8000'...");
                        let status = Command::new("php")
                            .arg("-S")
                            .arg("localhost:8000")
                            .status()
                            .expect("Failed to run PHP built-in server");
                        if !status.success() {
                            eprintln!("Failed to run PHP project.");
                        }
                    }
                }
                "python" => {
                    println!("Running Python project...");
                    let script = if std::path::Path::new("main.py").exists() {
                        "main.py".to_string()
                    } else if std::path::Path::new("app.py").exists() {
                        "app.py".to_string()
                    } else {
                        // Try to parse --path/-p from std::env::args()
                        let mut script_path: Option<String> = None;
                        let mut args = std::env::args().peekable();
                        while let Some(arg) = args.next() {
                            if arg == "--path" || arg == "-p" {
                                if let Some(val) = args.next() {
                                    script_path = Some(val);
                                    break;
                                }
                            }
                        }
                        match script_path {
                            Some(path) => path,
                            None => {
                                eprintln!("No 'main.py' or 'app.py' found in the current directory.\nPlease specify a script with --path <file.py> or -p <file.py>.");
                                return;
                            }
                        }
                    };
                    let status = Command::new("python")
                        .arg(&script)
                        .status()
                        .expect("Failed to run python script");
                    if !status.success() {
                        eprintln!("Failed to run Python project.");
                    }
                }
                _ => {
                    println!("Unsupported language for run: {}", lang);
                }
            }
        }
    }
}

fn install_npm() {
    // Add code here to install npm using the appropriate package manager for your OS
    // For example, you might use a command like `sudo apt-get install npm` on Ubuntu
    // or `brew install npm` on macOS.
    // Customize this function based on your system's package manager.
}

fn install_vue() {
    // Add code here to install Vue using npm
    // For example, you might use a command like `npm install -g @vue/cli`
    // Customize this function based on the specific installation command for Vue.
}