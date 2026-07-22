use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug)]
struct Detection {
    language: String,
    framework: String,
    runtime: String,
    entrypoint: String,
    run: Vec<String>,
    build: Vec<String>,
    test: Vec<String>,
    env_file: String,
    log_directory: String,
    log_patterns: Vec<String>,
}

pub fn run() -> Result<(), String> {
    let root = std::env::current_dir().map_err(|e| e.to_string())?;
    let config = root.join(".total/app.toml");
    if config.exists() {
        return Err(".total/app.toml already exists; initialization did not overwrite it".into());
    }

    let detection = detect(&root)?;
    let project_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("application");
    let contents = render(project_name, &detection);
    contents
        .parse::<toml::Value>()
        .map_err(|e| format!("Generated configuration failed validation: {e}"))?;

    create_support(&root).map_err(|e| format!("Failed to create Total support files: {e}"))?;
    fs::write(&config, contents).map_err(|e| format!("Failed to write .total/app.toml: {e}"))?;

    println!("Initialized Total CLI support for {project_name}.");
    println!("  language:  {}", detection.language);
    println!("  framework: {}", detection.framework);
    println!("  runtime:   {}", detection.runtime);
    println!("  entrypoint: {}", detection.entrypoint);
    println!("Created .total/app.toml and Total support infrastructure.");
    Ok(())
}

fn detect(root: &Path) -> Result<Detection, String> {
    // Framework-specific backend markers take precedence over package.json.
    // Laravel applications normally contain package.json for their Vite frontend,
    // but that does not make the application itself a JavaScript project.
    if root.join("artisan").is_file() {
        detect_php(root)
    } else if root.join("Cargo.toml").is_file() {
        detect_rust(root)
    } else if root.join("manage.py").is_file()
        || root.join("pyproject.toml").is_file()
        || root.join("requirements.txt").is_file()
        || root.join("main.py").is_file()
        || root.join("app.py").is_file()
    {
        detect_python(root)
    } else if root.join("composer.json").is_file() || root.join("index.php").is_file() {
        detect_php(root)
    } else if root.join("package.json").is_file() {
        let language = if root.join("tsconfig.json").is_file() {
            "typescript"
        } else {
            "javascript"
        };
        detect_javascript(root, language)
    } else {
        Err("Could not detect a supported application. Expected artisan, Cargo.toml, Python application files, composer.json, index.php, or package.json".into())
    }
}

pub fn detach() -> Result<(), String> {
    let root = std::env::current_dir().map_err(|e| e.to_string())?;
    let total = root.join(".total");
    if !total.exists() {
        return Err("No .total directory found; nothing to detach".into());
    }

    for file in [
        "app.toml",
        "ai/README.md",
        "logs/.gitkeep",
        "reports/.gitkeep",
    ] {
        let path = total.join(file);
        if path.is_file() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove {}: {e}", path.display()))?;
        }
    }
    for directory in ["ai", "logs", "reports", ""] {
        let path = total.join(directory);
        if path.is_dir() {
            match fs::remove_dir(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::DirectoryNotEmpty => {}
                Err(error) => return Err(format!("Failed to remove {}: {error}", path.display())),
            }
        }
    }
    println!("Detached Total CLI support. User-created files in .total were preserved.");
    Ok(())
}

fn detect_rust(root: &Path) -> Result<Detection, String> {
    require(root, "Cargo.toml", "Rust")?;
    let entrypoint = if root.join("src/main.rs").is_file() {
        "src/main.rs"
    } else {
        "src/lib.rs"
    };
    Ok(base(
        "rust",
        "cargo",
        "cargo",
        entrypoint,
        &["cargo", "run"],
        &["cargo", "build", "--release"],
        &["cargo", "test"],
        root,
    ))
}

fn detect_python(root: &Path) -> Result<Detection, String> {
    if !root.join("pyproject.toml").exists()
        && !root.join("requirements.txt").exists()
        && !root.join("main.py").exists()
        && !root.join("app.py").exists()
    {
        return Err("No Python application markers found (pyproject.toml, requirements.txt, main.py, or app.py)".into());
    }
    let entry = first_existing(root, &["main.py", "app.py", "manage.py", "src/main.py"])
        .unwrap_or_else(|| "main.py".into());
    let text = read_markers(root, &["pyproject.toml", "requirements.txt", &entry]);
    let (framework, run) = if text.contains("django") || entry == "manage.py" {
        ("django", vec!["python", "manage.py", "runserver"])
    } else if text.contains("fastapi") {
        ("fastapi", vec!["uvicorn", "main:app", "--reload"])
    } else if text.contains("flask") {
        ("flask", vec!["flask", "run"])
    } else {
        ("python", vec!["python", entry.as_str()])
    };
    Ok(base(
        "python",
        framework,
        "python",
        &entry,
        &run,
        &["python", "-m", "compileall", "."],
        &["python", "-m", "pytest"],
        root,
    ))
}

fn detect_javascript(root: &Path, language: &str) -> Result<Detection, String> {
    require(root, "package.json", "Node")?;
    let package = fs::read_to_string(root.join("package.json"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    let framework = if package.contains("\"next\"") {
        "next"
    } else if package.contains("\"vue\"") {
        "vue"
    } else if package.contains("\"react\"") {
        "react"
    } else if package.contains("\"express\"") {
        "express"
    } else {
        "node"
    };
    let ext = if language == "typescript" || root.join("tsconfig.json").exists() {
        "ts"
    } else {
        "js"
    };
    let candidates = [
        format!("src/main.{ext}"),
        format!("src/index.{ext}"),
        format!("index.{ext}"),
        format!("app.{ext}"),
        format!("server.{ext}"),
    ];
    let refs: Vec<&str> = candidates.iter().map(String::as_str).collect();
    let entry = first_existing(root, &refs).unwrap_or_else(|| candidates[0].clone());
    let manager = if root.join("pnpm-lock.yaml").exists() {
        "pnpm"
    } else if root.join("yarn.lock").exists() {
        "yarn"
    } else {
        "npm"
    };
    let (run, build, test) = if manager == "npm" {
        (
            vec!["npm", "run", "dev"],
            vec!["npm", "run", "build"],
            vec!["npm", "run", "test"],
        )
    } else {
        (
            vec![manager, "dev"],
            vec![manager, "build"],
            vec![manager, "test"],
        )
    };
    Ok(base(
        language, framework, "node", &entry, &run, &build, &test, root,
    ))
}

fn detect_php(root: &Path) -> Result<Detection, String> {
    if !root.join("composer.json").exists()
        && !root.join("index.php").exists()
        && !root.join("artisan").exists()
    {
        return Err(
            "No PHP application markers found (composer.json, index.php, or artisan)".into(),
        );
    }
    if root.join("artisan").exists() {
        Ok(base(
            "php",
            "laravel",
            "php",
            "artisan",
            &["php", "artisan", "serve"],
            &["composer", "install"],
            &["php", "artisan", "test"],
            root,
        ))
    } else {
        Ok(base(
            "php",
            "php",
            "php",
            "index.php",
            &["php", "-S", "localhost:8000"],
            &["composer", "install"],
            &["composer", "test"],
            root,
        ))
    }
}

fn base(
    language: &str,
    framework: &str,
    runtime: &str,
    entrypoint: &str,
    run: &[&str],
    build: &[&str],
    test: &[&str],
    root: &Path,
) -> Detection {
    let env_file = first_existing(root, &[".env", ".env.local", ".env.development"])
        .unwrap_or_else(|| ".env".into());
    let log_directory = if root.join("storage/logs").is_dir() {
        "storage/logs"
    } else if root.join("var/log").is_dir() {
        "var/log"
    } else {
        ".total/logs"
    };
    Detection {
        language: language.into(),
        framework: framework.into(),
        runtime: runtime.into(),
        entrypoint: entrypoint.into(),
        run: strings(run),
        build: strings(build),
        test: strings(test),
        env_file,
        log_directory: log_directory.into(),
        log_patterns: vec!["*.log".into(), "*.jsonl".into()],
    }
}

fn render(name: &str, d: &Detection) -> String {
    format!(
        r#"[project]
name = {}
language = {}
framework = {}
runtime = {}
entrypoints = [{}]

[commands]
run = {}
build = {}
test = {}

[environment]
env_file = {}
mode = "development"

[logging]
directory = {}
patterns = {}
structured = true

[monitoring]
enabled = true
health_endpoint = "/health"

[ai]
enabled = true
context_directory = ".total/ai"

[maintenance]
reports_directory = ".total/reports"
retention_days = 7
"#,
        q(name),
        q(&d.language),
        q(&d.framework),
        q(&d.runtime),
        q(&d.entrypoint),
        array(&d.run),
        array(&d.build),
        array(&d.test),
        q(&d.env_file),
        q(&d.log_directory),
        array(&d.log_patterns)
    )
}

fn create_support(root: &Path) -> io::Result<()> {
    for directory in [".total/ai", ".total/logs", ".total/reports"] {
        fs::create_dir_all(root.join(directory))?;
    }
    fs::write(
        root.join(".total/ai/README.md"),
        "# Total AI context\n\nAdd project-specific instructions and operational context here.\n",
    )?;
    fs::write(root.join(".total/logs/.gitkeep"), "")?;
    fs::write(root.join(".total/reports/.gitkeep"), "")?;
    Ok(())
}

fn require(root: &Path, marker: &str, kind: &str) -> Result<(), String> {
    if root.join(marker).exists() {
        Ok(())
    } else {
        Err(format!(
            "No {kind} application detected: {marker} is missing"
        ))
    }
}
fn first_existing(root: &Path, candidates: &[&str]) -> Option<String> {
    candidates
        .iter()
        .find(|p| root.join(p).exists())
        .map(|p| (*p).to_string())
}
fn read_markers(root: &Path, files: &[&str]) -> String {
    files
        .iter()
        .filter_map(|p| fs::read_to_string(root.join(p)).ok())
        .collect::<Vec<_>>()
        .join("\n")
        .to_ascii_lowercase()
}
fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).into()).collect()
}
fn q(value: &str) -> String {
    toml::Value::String(value.into()).to_string()
}
fn array(values: &[String]) -> String {
    values
        .iter()
        .map(|v| q(v))
        .collect::<Vec<_>>()
        .join(", ")
        .pipe(|s| format!("[{s}]"))
}

trait Pipe: Sized {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}
impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("total-{name}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
    #[test]
    fn rendered_configuration_is_valid_toml() {
        let d = base(
            "rust",
            "cargo",
            "cargo",
            "src/main.rs",
            &["cargo", "run"],
            &["cargo", "build"],
            &["cargo", "test"],
            Path::new("."),
        );
        render("sample", &d).parse::<toml::Value>().unwrap();
    }
    #[test]
    fn laravel_takes_precedence_over_its_javascript_frontend() {
        let root = fixture("laravel-detection");
        fs::write(root.join("artisan"), "").unwrap();
        fs::write(
            root.join("composer.json"),
            r#"{"require":{"laravel/framework":"^12.0"}}"#,
        )
        .unwrap();
        fs::write(
            root.join("package.json"),
            r#"{"devDependencies":{"vite":"latest","vue":"latest"}}"#,
        )
        .unwrap();

        let detected = detect(&root).unwrap();
        assert_eq!(detected.language, "php");
        assert_eq!(detected.framework, "laravel");
        assert_eq!(detected.entrypoint, "artisan");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn package_json_only_project_is_still_javascript() {
        let root = fixture("javascript-detection");
        fs::write(
            root.join("package.json"),
            r#"{"dependencies":{"vue":"latest"}}"#,
        )
        .unwrap();

        let detected = detect(&root).unwrap();
        assert_eq!(detected.language, "javascript");
        assert_eq!(detected.framework, "vue");
        fs::remove_dir_all(root).unwrap();
    }
}
