mod args;

use args::TotalArgs;
use clap::Parser;
use std::process::Command;

use std::env;
mod scaffolding;
use winreg::enums::*;
use winreg::RegKey;
use std::io;

use scaffolding::create_vue_scaffold;
use scaffolding::create_rust_scaffold;
use dialoguer::{theme::ColorfulTheme, Select};

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
        args::EntityType::Delete(project ) => {
            println!("Listing all installed programs:");
            let programs_lm = list_installed_programs(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall");
            let programs_cu = list_installed_programs(HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall");

            let mut all_programs = Vec::new();

            if let Ok(mut programs) = programs_lm {
                all_programs.append(&mut programs);
            }
            if let Ok(mut programs) = programs_cu {
                all_programs.append(&mut programs);
            }

            if all_programs.is_empty() {
                println!("No installed programs found.");
            } else {
                match select_program(&all_programs) {
                    Ok(selected_program) => println!("You selected: {}", selected_program),
                    Err(e) => eprintln!("Failed to select a program: {}", e),
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
                    use clap::Parser;
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

fn list_installed_programs(hkey: winreg::HKEY, path: &str) -> io::Result<Vec<Program>> {
    let reg_key = RegKey::predef(hkey);
    let uninstall_key = reg_key.open_subkey_with_flags(path, KEY_READ)?;
    let mut programs = Vec::new();

    for subkey_name in uninstall_key.enum_keys().flatten() {
        let subkey = uninstall_key.open_subkey_with_flags(&subkey_name, KEY_READ)?;

        // Try to get the DisplayName value
        if let Ok(display_name) = subkey.get_value::<String, _>("DisplayName") {
            // Try to get the InstallLocation value
            if let Ok(install_location) = subkey.get_value::<String, _>("InstallLocation") {
                programs.push(Program {
                    name: display_name,
                    path: install_location,
                });
            }
        }
    }

    Ok(programs)
}
#[derive(Debug)]
struct Program {
    name: String,
    path: String,
}


fn select_program(programs: &[Program]) -> io::Result<String> {
    let selections: Vec<String> = programs.iter().map(|p| p.name.clone()).collect();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a program to uninstall")
        .items(&selections)
        .default(0)
        .interact()?;

    Ok(programs[selection].path.clone())
}