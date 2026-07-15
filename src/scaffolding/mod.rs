use std::fs;
use std::io;
use std::path::Path;
use std::process::{exit, Command};
use which::which;

mod package_manager {
    use std::process::Command;
    use which::which;

    pub fn install_if_not_found(package_name: &str, install_command: &str) {
        if which(package_name).is_err() {
            install_package(install_command);
        }
    }

    fn install_package(install_command: &str) {
        println!("Installing: {}", install_command);
        let mut parts = install_command.split_whitespace();
        let cmd = match parts.next() {
            Some(c) => c,
            None => {
                eprintln!("Invalid install command.");
                std::process::exit(1);
            }
        };
        let args: Vec<&str> = parts.collect();
        let status = Command::new(cmd)
            .args(&args)
            .status()
            .expect("Failed to execute installation command");

        if !status.success() {
            eprintln!("Error installing package.");
            std::process::exit(1);
        }
    }
}

#[derive(Clone, Copy)]
enum ScaffoldKind {
    Python,
    Rust,
    Vue,
}

impl ScaffoldKind {
    fn manifest(self, project_name: &str) -> String {
        let project_name = project_name.replace('\\', "\\\\").replace('"', "\\\"");
        let (
            framework,
            language,
            run_command,
            run_args,
            entrypoint,
            build_command,
            build_args,
            dev_command,
            dev_args,
            test_command,
            test_args,
            logs_directory,
            logs_pattern,
            logs_main,
            template,
        ) = match self {
            Self::Python => (
                "python",
                "python",
                "python",
                r#"["main.py"]"#,
                "main.py",
                "python",
                r#"["-m", "pip", "install", "-r", "requirements.txt"]"#,
                "python",
                r#"["main.py"]"#,
                "python",
                r#"["-m", "unittest", "discover"]"#,
                "logs",
                "*.log",
                "",
                "python",
            ),
            Self::Rust => (
                "cargo",
                "rust",
                "cargo",
                r#"["run"]"#,
                "src/main.rs",
                "cargo",
                r#"["build", "--release"]"#,
                "cargo",
                r#"["run"]"#,
                "cargo",
                r#"["test"]"#,
                "target",
                "*.log",
                "",
                "rust",
            ),
            Self::Vue => (
                "vue",
                "javascript",
                "npm",
                r#"["run", "dev"]"#,
                "src/main.js",
                "npm",
                r#"["run", "build"]"#,
                "npm",
                r#"["run", "dev"]"#,
                "npm",
                r#"["run", "test:unit"]"#,
                "logs",
                "*.log",
                "",
                "vue",
            ),
        };

        format!(
            r#"[project]
name = "{project_name}"
description = ""
version = "0.1.0"
framework = "{framework}"
language = "{language}"
environment = "development"

[run]
command = "{run_command}"
args = {run_args}
working_directory = "."
entrypoint = "{entrypoint}"

[build]
command = "{build_command}"
args = {build_args}

[dev]
command = "{dev_command}"
args = {dev_args}

[test]
command = "{test_command}"
args = {test_args}

[logs]
directory = "{logs_directory}"
patterns = ["{logs_pattern}"]
main = "{logs_main}"

[cleanup]
enabled = true
analyze_logs = true
generate_report = true
delete_logs = false
truncate_logs = true
retention_days = 7

[ai]
directory = ".total/ai"

[deploy]
type = ""
root = ""

[health]
endpoint = "/health"

[environment]
env_file = ".env"

[metadata]
owner = ""
created_with = "total-cli"
template = "{template}"
"#
        )
    }
}

fn write_app_manifest(
    project_path: &Path,
    project_name: &str,
    kind: ScaffoldKind,
) -> io::Result<()> {
    let total_directory = project_path.join(".total");
    fs::create_dir_all(&total_directory)?;
    fs::write(
        total_directory.join("app.toml"),
        kind.manifest(project_name),
    )
}

fn finish_scaffold(project_name: &str, kind: ScaffoldKind) {
    if let Err(err) = write_app_manifest(Path::new(project_name), project_name, kind) {
        eprintln!("Failed to create .total/app.toml: {}", err);
        exit(1);
    }
}

fn write_python_project(project_path: &Path, project_name: &str) -> io::Result<()> {
    fs::create_dir_all(project_path.join("tests"))?;
    fs::write(
        project_path.join("main.py"),
        "def main():\n    print(\"Hello from Total!\")\n\n\nif __name__ == \"__main__\":\n    main()\n",
    )?;
    fs::write(project_path.join("requirements.txt"), "")?;
    fs::write(
        project_path.join(".gitignore"),
        "__pycache__/\n*.py[cod]\n.venv/\nvenv/\n.env\n",
    )?;
    write_app_manifest(project_path, project_name, ScaffoldKind::Python)
}

pub fn create_python_scaffold(project_name: &str) {
    let project_path = Path::new(project_name);
    if project_path.exists() {
        eprintln!("Directory '{}' already exists.", project_name);
        exit(1);
    }

    let create_result = write_python_project(project_path, project_name);

    if let Err(err) = create_result {
        eprintln!(
            "Failed to scaffold Python project '{}': {}",
            project_name, err
        );
        exit(1);
    }

    println!(
        "Python project '{}' successfully scaffolded with .total/app.toml.",
        project_name
    );
}

pub fn create_rust_scaffold(project_name: &str) {
    if Command::new("rustc").arg("--version").output().is_err() {
        println!("Rust is not installed. Installing Rust...");

        #[cfg(target_os = "windows")]
        let install_result = Command::new("winget")
            .args(["install", "--id", "Rustlang.Rustup", "-e"])
            .status();

        #[cfg(not(target_os = "windows"))]
        let install_result = Command::new("sh")
            .arg("-c")
            .arg("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y")
            .status();

        match install_result {
            Ok(s) if s.success() => println!("Rust installed successfully."),
            Ok(_) => {
                eprintln!("Rust installation failed. Install manually from https://rustup.rs");
                exit(1);
            }
            Err(e) => {
                eprintln!("Failed to launch installer: {}", e);
                exit(1);
            }
        }
    }

    match Command::new("cargo").arg("new").arg(project_name).status() {
        Ok(status) if status.success() => finish_scaffold(project_name, ScaffoldKind::Rust),
        Ok(_) => {
            eprintln!("Error: cargo new exited with a non-zero status.");
            exit(1);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(1);
        }
    }
    println!(
        "Rust project '{}' successfully scaffolded with cargo and .total/app.toml.",
        project_name
    );
}

pub fn create_vue_scaffold(project_name: &str) {
    println!("Starting a preliminary scan of your environment...");

    if Path::new(project_name).exists() {
        eprintln!("Directory '{}' already exists.", project_name);
        exit(1);
    }

    // npm comes with Node.js; on Windows use winget, elsewhere use the system package manager
    #[cfg(target_os = "windows")]
    package_manager::install_if_not_found("npm", "winget install OpenJS.NodeJS.LTS");
    #[cfg(not(target_os = "windows"))]
    package_manager::install_if_not_found("npm", "brew install node");

    // Install Vue CLI if missing
    #[cfg(target_os = "windows")]
    package_manager::install_if_not_found("vue", "npm.cmd install -g @vue/cli");
    #[cfg(not(target_os = "windows"))]
    package_manager::install_if_not_found("vue", "npm install -g @vue/cli");

    let vue_path = match which("vue") {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error finding vue: {}", e);
            exit(1);
        }
    };

    println!("[1/3] Starting the Vue scaffolding process...");

    let vue_create_command = Command::new(&vue_path)
        .arg("create")
        .arg("-d")
        .arg(project_name)
        .status();

    match vue_create_command {
        Ok(status) if status.success() => {
            finish_scaffold(project_name, ScaffoldKind::Vue);
            println!("[2/3] Building fresh packages...");
            println!("[3/3] Done.");
        }
        Ok(_) => {
            eprintln!("Error: vue create exited with a non-zero status.");
            exit(1);
        }
        Err(e) => {
            eprintln!("Failed to execute Vue create: {}", e);
            exit(1);
        }
    }

    println!(
        "Vue project '{}' successfully scaffolded with .total/app.toml.",
        project_name
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn writes_rust_app_manifest_in_total_directory() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let project = std::env::temp_dir().join(format!("total-cli-manifest-{unique}"));

        write_app_manifest(&project, "my-app", ScaffoldKind::Rust).unwrap();
        let manifest = fs::read_to_string(project.join(".total/app.toml")).unwrap();

        assert!(manifest.contains("name = \"my-app\""));
        assert!(manifest.contains("framework = \"cargo\""));
        assert!(manifest.contains("entrypoint = \"src/main.rs\""));
        assert!(manifest.contains("[cleanup]"));
        assert!(manifest.contains("[metadata]"));

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn vue_manifest_uses_npm_commands() {
        let manifest = ScaffoldKind::Vue.manifest("web-app");

        assert!(manifest.contains("framework = \"vue\""));
        assert!(manifest.contains("args = [\"run\", \"dev\"]"));
        assert!(manifest.contains("template = \"vue\""));
    }

    #[test]
    fn creates_a_complete_python_project() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let parent = std::env::temp_dir().join(format!("total-cli-python-{unique}"));
        let project = parent.join("python-app");

        write_python_project(&project, "python-app").unwrap();

        let manifest = fs::read_to_string(project.join(".total/app.toml")).unwrap();
        assert!(project.join("main.py").is_file());
        assert!(project.join("requirements.txt").is_file());
        assert!(project.join("tests").is_dir());
        assert!(manifest.contains("language = \"python\""));
        assert!(manifest.contains("args = [\"main.py\"]"));

        fs::remove_dir_all(parent).unwrap();
    }
}
