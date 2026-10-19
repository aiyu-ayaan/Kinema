# Package Kinema Windows portable zip and installer
Param(
    [string]$Version = "0.1.0"
)

Write-Host "Packaging Kinema for Windows (version $Version)..."
pnpm run build:web
cargo build --release --manifest-path backend/Cargo.toml
Write-Host "Windows packaging workflow prepared."
