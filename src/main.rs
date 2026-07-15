mod args;

use args::TotalArgs;
use clap::Parser;
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;

mod delete;
mod installer;
mod scaffolding;

use scaffolding::create_vue_scaffold;
use scaffolding::create_rust_scaffold;

fn python_script_and_args(extra_args: &[String]) -> Option<(String, Vec<String>)> {
    let mut forwarded_args = Vec::new();
    let mut script_path: Option<String> = None;
    let mut args = extra_args.iter();

    while let Some(arg) = args.next() {
        if arg == "--path" || arg == "-p" {
            script_path = args.next().cloned();
        } else {
            forwarded_args.push(arg.clone());
        }
    }

    let script = if let Some(path) = script_path {
        path
    } else if std::path::Path::new("main.py").exists() {
        "main.py".to_string()
    } else if std::path::Path::new("app.py").exists() {
        "app.py".to_string()
    } else {
        return None;
    };

    Some((script, forwarded_args))
}

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
                    if !installer::ensure_tool_available("cargo") {
                        return;
                    }
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
                    if !installer::ensure_tool_available("npm") {
                        return;
                    }
                    let _ = std::io::stdout().flush();
                    let status = Command::new(installer::npm_command())
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

                    if !installer::ensure_tool_available("php") {
                        return;
                    }
                    if is_vue && !installer::ensure_tool_available("npm") {
                        return;
                    }

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
                                let frontend = Command::new(installer::npm_command())
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
                    if !installer::ensure_tool_available("npm") {
                        return;
                    }
                    let _ = std::io::stdout().flush();
                    let status = Command::new(installer::npm_command())
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
                    println!("Running Python project with 'uv run python'...");
                    let (script, forwarded_args) = match python_script_and_args(&run.extra_args) {
                        Some(result) => result,
                        None => {
                            eprintln!("No 'main.py' or 'app.py' found in the current directory.\nPlease specify a script with --path <file.py> or -p <file.py>.");
                            return;
                        }
                    };
                    if !installer::ensure_tool_available("uv") {
                        return;
                    }
                    let _ = std::io::stdout().flush();
                    let status = Command::new("uv")
                        .arg("run")
                        .arg("python")
                        .arg(&script)
                        .args(&forwarded_args)
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status()
                        .expect("Failed to run python script with uv");
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
