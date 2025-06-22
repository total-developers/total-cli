use std::env;
use std::io::{self, Write};
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
        println!("Installing package: {}", install_command);
        let status = Command::new(install_command)
            .status()
            .expect("Failed to execute installation command");

        if !status.success() {
            eprintln!("Error installing package.");
            std::process::exit(1);
        }
    }
}
pub fn create_rust_scaffold(project_name: &String) -> () {
    // Check if Rust is installed
    if let Err(_) = Command::new("rustc").arg("--version").status() {
        println!("Rust is not installed. Installing Rust...");
        
        // Run Rustup installer
        if let Err(err) = Command::new("curl").arg("--proto").arg(" '=https'").arg("--tlsv1.2").arg("-sSf").arg("https://sh.rustup.rs").status() {
            eprintln!("Error: {}", err);
            exit(1);
        }
        
        // Check if installation was successful
        if let Err(err) = Command::new("rustup-init").status() {
            eprintln!("Error: {}", err);
            exit(1);
        }

        println!("Rust installation completed successfully.");
    } else {
        if let Err(err) = Command::new("cargo").arg("new").arg(project_name).status() {
            eprintln!("Error: {}", err);
            exit(1);
        }
        println!(
            "Rust project '{}' successfully scaffolded with cargo.",
            project_name
        );
        
    }
}
pub fn create_vue_scaffold(project_name: &String) -> () {
    println!("Staring a prelimanary scan to your enviornment...");
    if Path::new(&project_name).exists() {
        println!("Directory '{}' already exists", project_name);
        exit(1);
    }
    //TODO CHECK IF THIS WORKS

    // Check and install npm
    package_manager::install_if_not_found("npm", "your_npm_install_command_here");

    // Check and install Vue
    package_manager::install_if_not_found("vue", "npm install -g @vue/cli");
    // Find the path to vue
    let vue_path = match which("vue") {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error finding vue: {}", e);
            std::process::exit(1);
        }
    };
    println!("[1/3] Starting the Vue scaffolding process...");
    // Create a new Vue project using Vue
    let vue_create_command = Command::new(&vue_path)
        .arg("create")
        .arg("-d")
        .arg(project_name)
        .output();
    println!("[2/3] Building fresh packages...");
    // Check for vue create errors
    match vue_create_command {
        Ok(output) => {
            if output.status.success() {
                // Print standard output
                if !output.stdout.is_empty() {
                    println!("[3/3] Done.");
                }
            } else {
                // Print standard error in case of failure
                eprintln!(
                    "Error initializing Vue project: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to execute Vue create: {}", e);
            std::process::exit(1);
        }
    }
    println!(
        "Vue project '{}' successfully scaffolded with npm.",
        project_name
    );
}

