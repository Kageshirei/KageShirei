# Kageshirei Sample Extension

This is a sample extension for Kageshirei. It is a simple extension that does nothing but print some messages to the
console.
Use this as a template to create your own extensions.

## Installation

This extension should be used only with the `sample_extension_loader` executable (from `/libs/kageshirei-extensions`).

To install this, compile the loader with a simple `cargo build --bins` run in the `libs/kageshirei-extensions`
directory.

Then, compile this extension with `cargo build` in this directory.

Finally, move to the `target` directory, (if you run the command above without any edit you should enter the `debug`
directory) and copy the `sample_extension.dll` (or `sample_extension.so` on Linux) to the `extensions` directory (create
it if not exists), then run the `sample_extension_loader` executable.