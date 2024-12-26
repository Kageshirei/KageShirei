//! This module contains the command line logic for compiling the command and control GUI.
#![allow(
    clippy::expect_used,
    reason = "The expect is used to panic if the command fails when building the GUI"
)]
#![allow(
    unused,
    reason = "Currently compiling the agent in windows is not supported but it is on debian systems"
)]

use std::{env, process::Command};

use colored::Colorize as _;
use log::{error, info, warn};

/// List of required Debian packages
const REQUIRED_PACKAGES: [&str; 15] = [
    "libwebkit2gtk-4.1-dev",
    "build-essential",
    "curl",
    "wget",
    "file",
    "libssl-dev",
    "libgtk-3-dev",
    "libayatana-appindicator3-dev",
    "librsvg2-dev",
    "nsis",
    "lld",
    "llvm",
    "clang",
    "strace",
    "bash",
];

/// Checks if the current user is root
#[cfg(unix)]
fn check_root() -> Result<(), String> {
    if !nix::unistd::Uid::effective().is_root() {
        error!("This command must be run as root.");
        return Err("This command must be run as root".to_owned());
    }

    Ok(())
}

/// Installs a package using apt
fn install_packages() -> Result<(), String> {
    info!("Installing packages ...");
    let mut command = Command::new("apt");
    command.arg("install").arg("-y");

    for package in REQUIRED_PACKAGES.iter() {
        command.arg(package);
    }

    let status = command.status().expect("Failed to install packages");

    if !status.success() {
        error!("Failed to install one or more packages. Exiting.",);
        return Err("Failed to install package".to_owned());
    }

    Ok(())
}

/// Update the apt package list
fn update_apt() -> Result<(), String> {
    info!("Updating apt ...");
    let status = Command::new("apt")
        .arg("update")
        .arg("-y")
        .status()
        .expect("Failed to update apt");

    if !status.success() {
        error!("Failed to update apt. Exiting.");
        return Err("Failed to update apt".to_owned());
    }

    Ok(())
}

/// Update the apt package list
fn upgrade_apt() -> Result<(), String> {
    info!("Upgrading apt ...");
    let status = Command::new("apt")
        .arg("upgrade")
        .arg("-y")
        .status()
        .expect("Failed to update apt");

    if !status.success() {
        error!("Failed to upgrade apt. Exiting.");
        return Err("Failed to upgrade apt".to_owned());
    }

    Ok(())
}

/// Update the apt package list
fn autoremove_apt() -> Result<(), String> {
    info!("Running apt autoremove & autoclean...");
    let status = Command::new("apt")
        .arg("autoremove")
        .arg("-y")
        .status()
        .expect("Failed to run apt autoremove");

    if !status.success() {
        error!("Failed to run apt autoremove. Exiting.");
        return Err("Failed to run apt autoremove".to_owned());
    }

    let status = Command::new("apt")
        .arg("autoclean")
        .arg("-y")
        .status()
        .expect("Failed to run apt autoremove");

    if !status.success() {
        error!("Failed to run apt autoremove. Exiting.");
        return Err("Failed to run apt autoremove".to_owned());
    }

    Ok(())
}

/// Sources the Rust environment to the current process
fn source_rust_environment() -> Result<(), String> {
    // Source Rust environment
    if Command::new("sh")
        .arg("-c")
        .arg(". $HOME/.cargo/env")
        .status()
        .expect("Failed to source Rust environment")
        .success()
    {
        let cargo_bin_path = format!("{}/.cargo/bin", env::var("HOME").unwrap());
        env::set_var(
            "PATH",
            format!("{}:{}", cargo_bin_path, env::var("PATH").unwrap()),
        );
    }
    else {
        error!("Failed to source Rust environment. Exiting.");
        return Err("Failed to source Rust environment".to_owned());
    }

    Ok(())
}

/// Installs Rust using rustup
fn install_rust() -> Result<(), String> {
    if Command::new("rustc").arg("--version").output().is_err() {
        info!("Installing Rust...");
        let status = Command::new("sh")
            .arg("-c")
            .arg("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y")
            .status()
            .expect("Failed to install Rust");

        if !status.success() {
            error!("Failed to install Rust. Exiting.");
            return Err("Failed to install Rust".to_owned());
        }

        source_rust_environment()?;

        let status = Command::new("rustup")
            .arg("default")
            .arg("stable")
            .status()
            .expect("Failed to set the default Rust version");

        if !status.success() {
            error!("Failed to set the default Rust version. Exiting.");
            return Err("Failed to set the default Rust version".to_owned());
        }

        let status = Command::new("rustup")
            .arg("target")
            .arg("add")
            .arg("x86_64-pc-windows-msvc")
            .status()
            .expect("Failed to add the Windows target");

        if !status.success() {
            error!("Failed to add the Windows target. Exiting.");
            return Err("Failed to add the Windows target".to_owned());
        }

        let status = Command::new("rustup")
            .arg("target")
            .arg("add")
            .arg("x86_64-unknown-linux-gnu")
            .status()
            .expect("Failed to add the Linux target");

        if !status.success() {
            error!("Failed to add the Linux target. Exiting.");
            return Err("Failed to add the Linux target".to_owned());
        }
    }

    Ok(())
}

/// Installs NVM
fn install_nvm(shell: &str, config_file: &str) -> Result<(), String> {
    if Command::new("nvm").arg("--version").output().is_err() {
        info!("Installing NVM...");
        let status = Command::new(shell)
            .arg("-c")
            .arg("curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash")
            .status()
            .expect("Failed to install NVM");

        if !status.success() {
            error!("Failed to install NVM. Exiting.");
            return Err("Failed to install NVM".to_owned());
        }

        let status = Command::new(shell)
            .arg("-c")
            .arg(format!(". {} && nvm install --lts", config_file))
            .status()
            .expect("Failed to install the latest LTS Node.js version");

        if !status.success() {
            error!("Failed to install the latest LTS Node.js version. Exiting.");
            return Err("Failed to install the latest LTS Node.js version".to_owned());
        }
    }

    Ok(())
}

/// Installs PNPM
fn install_pnpm(shell: &str, config_file: &str) -> Result<(), String> {
    if Command::new("pnpm").arg("--version").output().is_err() {
        info!("Installing PNPM...");
        let status = Command::new(shell)
            .arg("-c")
            .arg(format!(". {} && npm i -g pnpm", config_file))
            .status()
            .expect("Failed to install PNPM");

        if !status.success() {
            error!("Failed to install PNPM. Exiting.");
            return Err("Failed to install PNPM".to_owned());
        }
    }

    Ok(())
}

/// Builds the client application
fn install_xwin() -> Result<(), String> {
    info!("Installing XWin to cross-compile the control panel ...");
    let status = Command::new("cargo")
        .arg("install")
        .arg("cargo-xwin")
        .status()
        .expect("Failed to install XWin");

    if !status.success() {
        error!("Failed to install XWin. Exiting.");
        return Err("Failed to install XWin".to_owned());
    }

    Ok(())
}

/// Builds the client application
fn build_command_and_control(shell: &str, config_file: &str) -> Result<(), String> {
    info!("Building the client application ...");

    let status = Command::new(shell)
        .arg("-c")
        .arg(format!(
            ". {} && pnpm install && pnpm run tauri:build",
            config_file
        ))
        .status()
        .expect("Failed to build the client application");

    if !status.success() {
        error!("Failed to build the client application. Exiting.");
        return Err("Failed to build the client application".to_owned());
    }

    info!("Cross compiling for windows ...");
    let status = Command::new(shell)
        .arg("-c")
        .arg(format!(
            ". {} && pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc",
            config_file
        ))
        .status();

    if status.is_err() || status.is_ok_and(|s| !s.success()) {
        warn!("XWin exited with an error. This was expected, windows bundles won't be available. Continuing ...");
    }

    info!("Client application built successfully.");
    info!(
        "Find the compiled clients at:
{}
    - target/release/kageshirei-command-and-control
    - target/release/bundle/deb/kageshirei-command-and-control_<version>_<arch>.deb
    - target/release/bundle/appimage/kageshirei-command-and-control_<version>_<arch>.AppImage
{}
    - target/x86_64-pc-windows-msvc/release/kageshirei-command-and-control.exe
",
        "Linux".bold(),
        "Windows".bold()
    );

    Ok(())
}

/// Compiles the command and control GUI
pub fn compile() -> Result<(), String> {
    #[cfg(unix)]
    {
        let home = env::var("HOME").expect("HOME environment variable not found");
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()); // Default to bash if SHELL is not set

        // Determine the appropriate configuration file
        let config_file = if shell.contains("zsh") {
            format!("{}/.zshrc", home)
        }
        else {
            format!("{}/.bashrc", home)
        };

        // Ensure the command is executed as root
        check_root()?;

        // Update the apt package list
        update_apt()?;

        // Upgrade the apt package list
        upgrade_apt()?;

        // Check and install required packages
        install_packages()?;

        // Remove unused packages anc clean the apt cache
        autoremove_apt()?;

        // Check and install Rust
        install_rust()?;

        // Check and install NVM
        install_nvm(&shell, &config_file)?;

        // Check and install PNPM
        install_pnpm(&shell, &config_file)?;

        // Change to the specified directory and run the PNPM build script
        let command_and_control_gui = "command-and-control-gui";
        if env::set_current_dir(command_and_control_gui).is_err() {
            error!("Failed to change directory to {}", command_and_control_gui);
            return Err("Failed to change directory".to_owned());
        }

        // Install XWin
        install_xwin()?;

        // Build the client application
        build_command_and_control(&shell, &config_file)?;
        Ok(())
    }
    #[cfg(windows)]
    {
        error!("This command is only available on Unix systems");
        Err("This command is only available on Unix systems".to_owned())
    }
}
