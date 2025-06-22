# Run Cargo build command
Write-Host "Building the Rust project..."
cargo build

# Check if the build was successful
if ($LastExitCode -eq 0) {
    Write-Host "Build successful!"

    # Run Cargo install command
    Write-Host "Installing the Rust project..."
    cargo install --path .

    # Check if the installation was successful
    if ($LastExitCode -eq 0) {
        Write-Host "Installation successful!"
    }
    else {
        Write-Host "Installation failed. Please check the error messages."
    }
}
else {
    Write-Host "Build failed. Please check the error messages."
}
