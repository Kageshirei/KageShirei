# Display the cargo build command
Write-Host "[+] Running cargo +nightly build $env:BUILD_MODE --locked --target x86_64-pc-windows-msvc --bin rs2-server"

# Run the cargo build command
& cargo +nightly build $env:BUILD_MODE --target x86_64-pc-windows-msvc --bin rs2-server

# Locating PostgreSQL support DLLs
Write-Host "[+] Locating pgsql support DLLs"

# Find PostgreSQL path in the PATH environment variable
$postgresqlPath = $env:PATH.Split(';') | Where-Object { $_ -match "PostgreSQL" }

if ($postgresqlPath)
{
    Write-Host "[+] PostgreSQL Path Found: $postgresqlPath"
}
else
{
    Write-Host "[!] PostgreSQL Path not found in PATH"
    Write-Host "[!] Please add the path to the PostgreSQL bin directory to the PATH environment variable"
    exit 1
}

# Set the PostgreSQL bin path
$binPath = Join-Path -Path $postgresqlPath -ChildPath "bin"

# Set destination directory based on the build mode
if ($env:BUILD_MODE -eq "--release")
{
    $destDir = ".\target\x86_64-pc-windows-msvc\release"
}
else
{
    $destDir = ".\target\x86_64-pc-windows-msvc\debug"
}

# List of files to copy
$files = @("libcrypto-3-x64.dll", "libiconv-2.dll", "libintl-9.dll", "libpq.dll", "libssl-3-x64.dll", "libwinpthread-1.dll")

# Copy each file if it exists
foreach ($file in $files)
{
    $sourcePath = Join-Path -Path $binPath -ChildPath $file
    if (Test-Path $sourcePath)
    {
        Write-Host "Copying PostgreSQL $file to $destDir..."
        Copy-Item -Path $sourcePath -Destination $destDir
    }
    else
    {
        Write-Host "File $file not found in $binPath."
    }
}
