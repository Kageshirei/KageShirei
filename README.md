# RS2

RS2 is a modern, highly-customizable, command and control framework for red teams.

## Project structure

The project is structured into 4 main components:

- An `agent` that will be deployed on the target machine
- A `server` responsible for the management of agents, connections, and tasks
- A `command-and-control-gui` that will be used by red team operators to interact with the server
- A series of `libs` that will be shared between the different components

## Features & status

The project is still in its early stages, we have many features planned, we'll proceed to implement them as we go.

<details>
<summary>Features summary</summary>

- [Working features](#working-features)
- [Planned features](#planned-features)
- [Ideas / Future features](#ideas--future-features)

</details>

### Working features

#### Agent

- None :(

#### Server

- None :(

### Planned features

#### Agent

- None :(

#### Server

- None :(

### Ideas / Future features

#### Agent

- Multiple connections setup to connect back to the server, ordered by priority. (e.g. DNS, HTTP, HTTPS,
  etc). This will allow the agent to be more resilient to network restrictions.
- Protobuf communication between the agent and the server.
- JSON communication between the agent and the server.
- Ability to execute tasks on the agent.
- File/folder management on the agent (similar to explorer.exe).
- Extract information about the target machine (e.g. OS, architecture, etc).
- Ability to execute shell commands.
- Ability to execute PowerShell commands.
- Ability to execute Python scripts.
- HTTP/HTTPS connection to the server.
- DNS connection to the server.
- Feature toggles. (conditional compilation)

## Server installation

### Requirements

The server should come with no dependencies at all, most of its features should be able to run both on *nix and
Windows systems.

Compilation must be done before running it (no precompiled binaries). To compile the server only, clone the
repository and run the following commands:

```bash
cd rs2
cargo --version
```

If you don't have `cargo` installed, you can install it using [rustup](https://rustup.rs/#), then run the following
commands:

```bash
cargo build --release --bin rs2-server
```

This will compile the server in release mode, you can find the binary in `./target/release/rs2-server`.
Note that the compilation process may take a while, especially if you're compiling the project for the first time.
Additionally, the executable will be quite large, as it will contain all the dependencies statically linked and will be
optimized for performances (instead of size such as the agent and the control panel).

Refer to the [server's own documentation](server/README.md) for a list of commands and usage instructions.

#### Building on windows?

It can be done, even if some features are not available.
Refer to [this comment](https://github.com/diesel-rs/diesel/issues/587#issuecomment-574934244) for instruction on how to
overcome some common issues.

### The server component superpowers

The server component apart from being the main component of the RS2 framework, it also has some superpowers that makes
it a very versatile tool.

- **Agent compilation on demand**: The server can compile the agent on demand, this means that you can compile the agent
  with different configurations, features, and even different code. This is useful when you want to deploy the agent on
  different targets with different requirements.
- **Control panel compilation**: The server can compile the control panel autonomously, this means that you don't have
  to worry about building the control panel, installing the dependencies (lots of) as the server component will do it
  for you.

> **NOTE**:
> Self compilation is only available on debian-based systems, this may be extended in the future to other systems.

## Control panel installation

### Requirements

Unfortunately building the control panel is far from being as easy as building the server.
The control panel is an hybrid application built on top of Tauri + Next.js (React), this means that you'll need to have
lots of dependencies to build it.

Fortunately if you've previously built the `server`, you can use it to build the control panel as well.
It will download all the dependencies, compile them and create the final executable for you, you just have to run it.

To let the server do the job for you, you can run the following command:

```bash
./rs2-server compile gui
```

If for some reason you want to build the control panel manually, you can refer to the
[control panel's own documentation](command-and-control-gui/README.md).

## Contributing

We welcome contributions from the community, feel free to open an issue or a pull request if you want to help us improve
the project.
We have a [Code of Conduct](CODE_OF_CONDUCT.md) that we expect contributors to follow.

## License

This project is licensed under the GNU General Public License v2 - see the [LICENSE](LICENSE) file for details.
