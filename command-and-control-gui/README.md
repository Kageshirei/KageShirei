# Command and Control GUI

## What is this?

This is the graphical user interface for the RS2 framework, it is a hybrid application that allows you to interact with
the agents, manage the server, and visualize the data collected by the agents.

## Installation - quick method

The easiest way to install the GUI is to use the server component to build it for you.
If you haven't built the server yet, you can follow the instructions [here](../README.md#server-installation).

Once you have the server built, you can use it to build the GUI as well.

```bash
rs2-server compile gui
```

This will download all the dependencies, compile them and create the final executable for you, you just have to run it.

> **NOTE**:
> The server component is able to compile the GUI **_ONLY_** on debian-based systems.

## Installation - manual method

If you want to build the GUI manually, you can follow the instructions below.

### Requirements

1) **For Windows users** - Windows WSL (Windows Subsystem for Linux) with debian-based distribution (Ubuntu, Debian,
   etc.)
2) [NVM (Node Version Manager)](https://github.com/nvm-sh/nvm?tab=readme-ov-file#installing-and-updating) - for Windows
   users, check also [this link](https://github.com/nvm-sh/nvm?tab=readme-ov-file#important-notes)
3) PNPM - you can install it after installing NVM with the following commands:
   ```bash
   nvm install lts/*
   nvm use lts/*
   nvm alias default lts/*
   npm install -g pnpm
   ```
4) [Tauri's prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)

### Building the GUI

Assuming you'll compile the Control Panel from the system where you'll use it (Windows for Windows users etc.) the after
you've installed the prerequisites, you can follow the steps below.

Clone the repository and navigate to the GUI directory, then run the following commands:

```bash
pnpm run tauri:build
```

This will compile the GUI and the bundles (`.appimage`/`.deb` for linux, `.msi`/`*-installer.exe` for windows), you will
find the binary in the root `target` folder under:

- `target/release/rs2-command-and-control` (linux)
- `target/release/bundle/deb/rs2-command-and-control_<version>_<arch>.deb` (linux)
- `target/release/bundle/appimage/rs2-command-and-control_<version>_<arch>.AppImage` (linux)
- `target/x86_64-pc-windows-msvc/release/rs2-command-and-control.exe` (windows)

### Cross-compiling the GUI

Cross compiling the GUI is possible, but it requires a bit more work, you can refer to the
[Tauri's documentation](https://tauri.app/v1/guides/building/cross-platform) for more information, or, as strongly
suggested,
you can use the server component to build the GUI for you, it will take care of dependency management and
cross-compilation
for you.
