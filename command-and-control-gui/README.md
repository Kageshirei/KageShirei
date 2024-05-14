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
> The server component is able to compile the GUI only on debian-based systems, this may be extended in the future to
> other systems.

## Installation - manual method

If you want to build the GUI manually, you can follow the instructions below.

### Requirements

1) **For Windows users** - Windows WSL (Windows Subsystem for Linux) with debian-based distribution (Ubuntu, Debian,
   etc.)
2) [NVM (Node Version Manager)](https://github.com/nvm-sh/nvm?tab=readme-ov-file#installing-and-updating) - for Windows
   users, check also [this link](https://github.com/nvm-sh/nvm?tab=readme-ov-file#important-notes)
3) PNPM
4) [Tauri's prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)