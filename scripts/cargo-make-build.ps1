# Display the cargo build command
Write-Host "[+] Running cargo +nightly build $env:BUILD_MODE --locked --target x86_64-pc-windows-msvc --bin kageshirei-server"

# Run the cargo build command
$env:RUSTFLAGS = "-Zmacro-backtrace"
& cargo +nightly build $env:BUILD_MODE --target x86_64-pc-windows-msvc --bin kageshirei-server