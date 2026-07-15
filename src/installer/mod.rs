use dialoguer::Confirm;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use which::which;

/// npm's executable is `npm.cmd` on Windows and `npm` elsewhere.
pub fn npm_command() -> &'static str {
    if cfg!(windows) {
        "npm.cmd"
    } else {
        "npm"
    }
}

/// Checks that `tool` is on PATH; if not, offers to download and install it
/// using whatever install mechanism the current OS provides. Returns true
/// once the tool is available in this session.
pub fn ensure_tool_available(tool: &str) -> bool {
    if which(tool).is_ok() {
        return true;
    }

    let confirmed = Confirm::new()
        .with_prompt(format!(
            "{} is not installed. Download and install it now?",
            tool
        ))
        .default(true)
        .interact()
        .unwrap_or(false);

    if !confirmed {
        return false;
    }

    println!("Installing {}...", tool);
    let installed = match tool {
        "cargo" => install_rust(),
        "npm" | "node" => install_node(),
        "php" => install_php(),
        "uv" => install_uv(),
        other => {
            eprintln!("Don't know how to install '{}'.", other);
            false
        }
    };

    if !installed {
        eprintln!("{} installation failed.", tool);
        return false;
    }

    refresh_path();
    if which(tool).is_ok() {
        true
    } else {
        eprintln!(
            "{} was installed, but it is not available in PATH for this session. Restart your terminal and try again.",
            tool
        );
        false
    }
}

enum PackageManager {
    Winget,
    Brew,
    Apt,
    Dnf,
    Pacman,
}

fn detect_package_manager() -> Option<PackageManager> {
    if cfg!(windows) {
        return if which("winget").is_ok() {
            Some(PackageManager::Winget)
        } else {
            None
        };
    }
    let candidates = [
        ("brew", PackageManager::Brew),
        ("apt-get", PackageManager::Apt),
        ("dnf", PackageManager::Dnf),
        ("pacman", PackageManager::Pacman),
    ];
    for (name, pm) in candidates {
        if which(name).is_ok() {
            return Some(pm);
        }
    }
    None
}

fn install_rust() -> bool {
    if cfg!(windows) {
        if which("winget").is_ok() {
            run_winget("Rustlang.Rustup")
        } else {
            // No winget: download rustup-init directly.
            run_powershell(
                "iwr https://win.rustup.rs/x86_64 -OutFile \"$env:TEMP\\rustup-init.exe\"; & \"$env:TEMP\\rustup-init.exe\" -y",
            )
        }
    } else {
        // rustup's script works on any Unix without a package manager.
        run_unix_shell(
            "if command -v curl >/dev/null 2>&1; then curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; else wget -qO- https://sh.rustup.rs | sh -s -- -y; fi",
        )
    }
}

fn install_node() -> bool {
    match detect_package_manager() {
        Some(PackageManager::Winget) => run_winget("OpenJS.NodeJS.LTS"),
        Some(PackageManager::Brew) => run_unix_shell("brew install node"),
        Some(PackageManager::Apt) => apt_install("nodejs npm"),
        Some(PackageManager::Dnf) => run_unix_shell(&with_sudo("dnf install -y nodejs npm")),
        Some(PackageManager::Pacman) => {
            run_unix_shell(&with_sudo("pacman -S --noconfirm nodejs npm"))
        }
        None => {
            no_package_manager_help("Node.js", "https://nodejs.org");
            false
        }
    }
}

fn install_php() -> bool {
    match detect_package_manager() {
        Some(PackageManager::Winget) => run_winget("PHP.PHP.8.4"),
        Some(PackageManager::Brew) => run_unix_shell("brew install php"),
        Some(PackageManager::Apt) => apt_install("php-cli"),
        Some(PackageManager::Dnf) => run_unix_shell(&with_sudo("dnf install -y php-cli")),
        Some(PackageManager::Pacman) => run_unix_shell(&with_sudo("pacman -S --noconfirm php")),
        None => {
            no_package_manager_help("PHP", "https://www.php.net/downloads");
            false
        }
    }
}

fn install_uv() -> bool {
    if cfg!(windows) {
        run_powershell("irm https://astral.sh/uv/install.ps1 | iex")
    } else {
        run_unix_shell(
            "if command -v curl >/dev/null 2>&1; then curl -LsSf https://astral.sh/uv/install.sh | sh; else wget -qO- https://astral.sh/uv/install.sh | sh; fi",
        )
    }
}

fn no_package_manager_help(tool: &str, url: &str) {
    if cfg!(windows) {
        eprintln!(
            "winget was not found, so {} cannot be installed automatically. Install it manually from {}",
            tool, url
        );
    } else if cfg!(target_os = "macos") {
        eprintln!(
            "Homebrew was not found, so {} cannot be installed automatically. Install Homebrew (https://brew.sh) or install {} manually from {}",
            tool, tool, url
        );
    } else {
        eprintln!(
            "No supported package manager (apt-get, dnf, pacman, brew) was found, so {} cannot be installed automatically. Install it manually from {}",
            tool, url
        );
    }
}

fn apt_install(packages: &str) -> bool {
    let script = format!(
        "{} && {}",
        with_sudo("apt-get update"),
        with_sudo(&format!("apt-get install -y {}", packages))
    );
    run_unix_shell(&script)
}

/// Prefixes a command with sudo when available (e.g. not needed inside
/// root containers, where sudo itself is often missing).
fn with_sudo(command: &str) -> String {
    if which("sudo").is_ok() {
        format!("sudo {}", command)
    } else {
        command.to_string()
    }
}

fn run_winget(id: &str) -> bool {
    run_status(
        Command::new("winget").args([
            "install",
            "--id",
            id,
            "-e",
            "--accept-source-agreements",
            "--accept-package-agreements",
        ]),
    )
}

fn run_unix_shell(script: &str) -> bool {
    run_status(Command::new("sh").arg("-c").arg(script))
}

fn run_powershell(script: &str) -> bool {
    run_status(
        Command::new("powershell")
            .arg("-ExecutionPolicy")
            .arg("ByPass")
            .arg("-c")
            .arg(script),
    )
}

fn run_status(command: &mut Command) -> bool {
    let status = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    match status {
        Ok(status) => status.success(),
        Err(err) => {
            eprintln!("Failed to start installer: {}", err);
            false
        }
    }
}

/// Makes freshly installed tools visible to this session: on Windows,
/// re-reads PATH from the registry (installers update it there, not in the
/// current process); everywhere, appends common per-user install dirs.
fn refresh_path() {
    let current = env::var_os("PATH").unwrap_or_default();
    let mut paths: Vec<PathBuf> = env::split_paths(&current).collect();
    let mut extra: Vec<PathBuf> = Vec::new();

    if cfg!(windows) {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "[Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')",
            ])
            .output();
        if let Ok(output) = output {
            if output.status.success() {
                let registry_path =
                    OsString::from(String::from_utf8_lossy(&output.stdout).trim().to_string());
                extra.extend(env::split_paths(&registry_path));
            }
        }
    }

    if let Some(home_dir) = home::home_dir() {
        extra.push(home_dir.join(".cargo").join("bin"));
        extra.push(home_dir.join(".local").join("bin"));
    }

    for dir in extra {
        if !dir.as_os_str().is_empty() && dir.exists() && !paths.iter().any(|p| p == &dir) {
            paths.push(dir);
        }
    }

    if let Ok(new_path) = env::join_paths(paths) {
        env::set_var("PATH", new_path);
    }
}
