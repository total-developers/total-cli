mod args;

use args::TotalArgs;
use clap::Parser;
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;

mod delete;
mod init;
mod installer;
mod scaffolding;

use scaffolding::create_python_scaffold;
use scaffolding::create_rust_scaffold;
use scaffolding::create_vue_scaffold;

fn manifest_language() -> Result<String, String> {
    let path = std::path::Path::new(".total/app.toml");
    if !path.exists() {
        return Err(format!(
            "No {} found. Run this command from a project created by Total CLI.",
            path.display()
        ));
    }

    let contents = std::fs::read_to_string(path)
        .map_err(|err| format!("Failed to read {}: {}", path.display(), err))?;
    manifest_language_from(&contents).map_err(|err| format!("{} in {}", err, path.display()))
}

fn manifest_language_from(contents: &str) -> Result<String, String> {
    let manifest: toml::Value = contents
        .parse()
        .map_err(|err| format!("Failed to parse manifest: {}", err))?;
    let project = manifest
        .get("project")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| "Manifest is missing a [project] section".to_string())?;

    let framework = project.get("framework").and_then(toml::Value::as_str);
    let language = project.get("language").and_then(toml::Value::as_str);
    let runnable = match framework.map(str::to_lowercase).as_deref() {
        Some("cargo") | Some("rust") => Some("rust"),
        Some("vue") => Some("vue"),
        Some("next") | Some("nextjs") | Some("next.js") => Some("next"),
        Some("laravel") | Some("php") => Some("php"),
        Some("python") => Some("python"),
        _ => match language.map(str::to_lowercase).as_deref() {
            Some("rust") => Some("rust"),
            Some("python") | Some("py") => Some("python"),
            Some("php") => Some("php"),
            Some("vue") => Some("vue"),
            Some("next") | Some("nextjs") | Some("next.js") => Some("next"),
            _ => None,
        },
    };

    runnable
        .map(str::to_string)
        .ok_or_else(|| "Could not determine a supported project type from manifest".to_string())
}

fn run_language(explicit: Option<&str>) -> Result<String, String> {
    match explicit {
        Some(language) => Ok(language.to_lowercase()),
        None => manifest_language(),
    }
}

fn run_inputs<'a>(
    language: Option<&'a String>,
    extra_args: &[String],
) -> (Option<&'a str>, Vec<String>) {
    match language {
        Some(value) if value.starts_with('-') => {
            let mut forwarded = Vec::with_capacity(extra_args.len() + 1);
            forwarded.push(value.clone());
            forwarded.extend_from_slice(extra_args);
            (None, forwarded)
        }
        Some(value) => (Some(value.as_str()), extra_args.to_vec()),
        None => (None, extra_args.to_vec()),
    }
}

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
        args::EntityType::Init => {
            if let Err(err) = init::run() {
                eprintln!("Initialization failed: {err}");
                std::process::exit(1);
            }
        }
        args::EntityType::Detach => {
            if let Err(err) = init::detach() {
                eprintln!("Detach failed: {err}");
                std::process::exit(1);
            }
        }
        args::EntityType::Create(project) => {
            let PROJECT_LANGUAGE: String = project.language.to_lowercase();
            let PROJECT_TITLE: String = project.title.to_lowercase();

            match PROJECT_LANGUAGE.as_str() {
                "python" | "py" => {
                    println!(
                        "Creating a program named: {:?}, in {:?}",
                        PROJECT_TITLE, PROJECT_LANGUAGE
                    );
                    create_python_scaffold(&PROJECT_TITLE)
                }
                "rust" => {
                    println!(
                        "Creating a program named: {:?}, in {:?}",
                        PROJECT_TITLE, PROJECT_LANGUAGE
                    );
                    create_rust_scaffold(&PROJECT_TITLE)
                }
                "vue" => {
                    println!(
                        "Creating a program named: {:?}, in {:?}",
                        PROJECT_TITLE, PROJECT_LANGUAGE
                    );
                    create_vue_scaffold(&PROJECT_TITLE);
                }

                _ => eprintln!(
                    "Unsupported language '{}'. Supported project types: python, rust, vue.",
                    PROJECT_LANGUAGE
                ),
            }
        }
        args::EntityType::Delete(project) => {
            delete::run(&project.path);
        }
        args::EntityType::Run(run) => {
            let (explicit_language, extra_args) =
                run_inputs(run.language.as_ref(), &run.extra_args);
            let lang = match run_language(explicit_language) {
                Ok(language) => language,
                Err(err) => {
                    eprintln!("{}", err);
                    return;
                }
            };
            match lang.as_str() {
                "rust" => {
                    println!("Running Rust project with 'cargo run'...");
                    if !installer::ensure_tool_available("cargo") {
                        return;
                    }
                    let _ = std::io::stdout().flush();
                    let status = Command::new("cargo")
                        .arg("run")
                        .args(&extra_args)
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
                        .args(&extra_args)
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
                            .args(&extra_args)
                            .stdin(Stdio::inherit())
                            .stdout(Stdio::inherit())
                            .stderr(Stdio::inherit())
                            .spawn();

                        match backend {
                            Ok(mut child) => {
                                let frontend = Command::new(installer::npm_command())
                                    .arg("run")
                                    .arg("dev")
                                    .args(&extra_args)
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
                            .args(&extra_args)
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
                            .args(&extra_args)
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
                        .args(&extra_args)
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
                    let (script, forwarded_args) = match python_script_and_args(&extra_args) {
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

#[cfg(test)]
mod tests {
    use crate::args::{EntityType, TotalArgs};
    use clap::Parser;
    use super::{manifest_language, manifest_language_from, run_inputs, run_language};

    #[test]
    fn explicit_language_preserves_the_original_run_behavior() {
        assert_eq!(run_language(Some("RUST")).unwrap(), "rust");
    }

    #[test]
    fn init_and_detach_commands_parse() {
        assert!(matches!(
            TotalArgs::try_parse_from(["total", "init"]).unwrap().entity_type,
            EntityType::Init
        ));
        assert!(matches!(
            TotalArgs::try_parse_from(["total", "detach"]).unwrap().entity_type,
            EntityType::Detach
        ));
    }

    #[test]
    fn run_accepts_both_original_and_automatic_forms() {
        let original = TotalArgs::try_parse_from(["total", "run", "rust", "--release"]).unwrap();
        let automatic = TotalArgs::try_parse_from(["total", "run", "--release"]).unwrap();

        match original.entity_type {
            EntityType::Run(run) => {
                assert_eq!(run.language.as_deref(), Some("rust"));
                assert_eq!(run.extra_args, ["--release"]);
            }
            _ => panic!("expected run command"),
        }
        match automatic.entity_type {
            EntityType::Run(run) => {
                let (language, args) = run_inputs(run.language.as_ref(), &run.extra_args);
                assert_eq!(language, None);
                assert_eq!(args, ["--release"]);
            }
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn language_is_detected_from_the_generated_manifest_fields() {
        assert_eq!(
            manifest_language_from("[project]\nframework = \"cargo\"\nlanguage = \"rust\"")
                .unwrap(),
            "rust"
        );
        assert_eq!(
            manifest_language_from("[project]\nframework = \"vue\"\nlanguage = \"javascript\"")
                .unwrap(),
            "vue"
        );
        assert_eq!(
            manifest_language_from("[project]\nframework = \"python\"\nlanguage = \"python\"")
                .unwrap(),
            "python"
        );
    }

    #[test]
    fn running_without_a_manifest_has_a_clear_error() {
        assert!(manifest_language().unwrap_err().contains(".total/app.toml"));
    }
}
