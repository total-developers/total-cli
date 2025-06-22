use std::process::Command;
use which::which;

pub fn install_if_not_found(package_name: &str, install_command: &str) {
    if which(package_name).is_err() {
        install_package(install_command);
    }
}

pub fn install_package(install_command: &str) {
    println!("Installing package: {}", install_command);
    // Split the command into program and args
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