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

    if let Err(err) = Command::new("cargo").arg("new").arg(project_name).status() {
        eprintln!("Error: {}", err);
        exit(1);
    }
    println!(
        "Rust project '{}' successfully scaffolded with cargo.",
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

    println!("Vue project '{}' successfully scaffolded.", project_name);
}
