mod args;

use args::TotalArgs;
use clap::Parser;
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;

mod delete;
mod scaffolding;

use scaffolding::create_vue_scaffold;
use scaffolding::create_rust_scaffold;

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
            delete::run(&project.path);
        }
        args::EntityType::Run(run) => {
            let lang = run.path.to_lowercase();
            match lang.as_str() {
                "rust" => {
                    println!("Running Rust project with 'cargo run'...");
                    let _ = std::io::stdout().flush();
                    let status = Command::new("cargo")
                        .arg("run")
                        .args(&run.extra_args)
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status()
                        .expect("Failed to run cargo");
                    if !status.success() {
                        eprintln!("Failed to run Rust project.");
                    }
                }
                "vue" => {
                    println!("Running Vue project with 'npm run dev'...");
                    let _ = std::io::stdout().flush();
                    let status = Command::new("npm.cmd")
                        .arg("run")
                        .arg("dev")
                        .args(&run.extra_args)
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status()
                        .expect("Failed to run npm");
                    if !status.success() {
                        eprintln!("Failed to run Vue project.");
                    }
                }
                "php" => {
                    use std::path::Path;
                    let is_laravel = Path::new("artisan").exists();
                    let is_vue = Path::new("package.json").exists()
                        && std::fs::read_to_string("package.json")
                            .map(|c| c.contains("\"vue\""))
                            .unwrap_or(false);

                    if is_laravel && is_vue {
                        println!("Detected Laravel + Vue project.");
                        println!("Starting backend: 'php artisan serve'...");
                        println!("Starting frontend: 'npm run dev'...");
                        let _ = std::io::stdout().flush();

                        let backend = Command::new("php")
                            .arg("artisan")
                            .arg("serve")
                            .args(&run.extra_args)
                            .stdin(Stdio::inherit())
                            .stdout(Stdio::inherit())
                            .stderr(Stdio::inherit())
                            .spawn();

                        match backend {
                            Ok(mut child) => {
                                let frontend = Command::new("npm.cmd")
                                    .arg("run")
                                    .arg("dev")
                                    .args(&run.extra_args)
                                    .stdin(Stdio::null())
                                    .stdout(Stdio::inherit())
                                    .stderr(Stdio::inherit())
                                    .spawn();
                                match frontend {
                                    Ok(mut child2) => {
                                        // Wait on both concurrently so neither blocks the other's output
                                        let t1 = thread::spawn(move || child.wait());
                                        let t2 = thread::spawn(move || child2.wait());
                                        let _ = t1.join();
                                        let _ = t2.join();
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
                        let _ = std::io::stdout().flush();
                        let status = Command::new("php")
                            .arg("artisan")
                            .arg("serve")
                            .args(&run.extra_args)
                            .stdin(Stdio::inherit())
                            .stdout(Stdio::inherit())
                            .stderr(Stdio::inherit())
                            .status()
                            .expect("Failed to run php artisan serve");
                        if !status.success() {
                            eprintln!("Failed to run Laravel project.");
                        }
                    } else {
                        println!("Running PHP project with 'php -S localhost:8000'...");
                        let _ = std::io::stdout().flush();
                        let status = Command::new("php")
                            .arg("-S")
                            .arg("localhost:8000")
                            .args(&run.extra_args)
                            .stdin(Stdio::inherit())
                            .stdout(Stdio::inherit())
                            .stderr(Stdio::inherit())
                            .status()
                            .expect("Failed to run PHP built-in server");
                        if !status.success() {
                            eprintln!("Failed to run PHP project.");
                        }
                    }
                }
                "next" | "nextjs" | "next.js" => {
                    println!("Running Next.js project with 'npm run dev'...");
                    let _ = std::io::stdout().flush();
                    let status = Command::new("npm.cmd")
                        .arg("run")
                        .arg("dev")
                        .args(&run.extra_args)
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status()
                        .expect("Failed to run Next.js dev server");
                    if !status.success() {
                        eprintln!("Failed to run Next.js project.");
                    }
                }
                "python" => {
                    println!("Running Python project...");
                    let script = if std::path::Path::new("main.py").exists() {
                        "main.py".to_string()
                    } else if std::path::Path::new("app.py").exists() {
                        "app.py".to_string()
                    } else {
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
                    let _ = std::io::stdout().flush();
                    let status = Command::new("python")
                        .arg(&script)
                        .args(&run.extra_args)
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
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

