[CmdletBinding()]
Param(
  [Parameter(Mandatory=$true)]
  [ValidateSet("windows-2022", "ubuntu-22.04", "ubuntu-24.04", "macos-13", "macos-14")]
  [string]$Runner,

  [Parameter(Mandatory=$true)]
  [ValidateSet("amd64", "x64", "arm64")]
  [string]$Arch,

  [Parameter(Mandatory=$true)]
  [string]$Version
)

$ErrorActionPreference = "Stop"

function Write-Output
{
    param ( [string]$name, [string]$value )

    Write-Host ("::set-output name=$name::$value")
}

function Get-Sem-Version
{
    param ( [string]$version )

    $index = $version.LastIndexOf("-")

    if ($index -eq -1) {
        return $version
    }


    $numbers = $version.Remove($index).Split("-")
    $numbers = @($numbers | ForEach-Object {
        if ($_ -eq "2020") {
            "20"
        } else {
            $_
        }
    })

    return [string]::Join(".", $numbers)
}

New-Item -Path . -Name "output" -ItemType "directory" -Force

switch($Runner)
{
  { $_ -in @("ubuntu-22.04", "ubuntu-24.04") }
  {
    cargo install cargo-deb

    if ($Arch -eq "arm64") {
      # Cross-compilation setup for ARM64
      rustup target add aarch64-unknown-linux-gnu

      # Install cross-compilation toolchain
      sudo apt-get update
      sudo apt-get install -y gcc-aarch64-linux-gnu pkg-config

      # Set environment variables for linker and static OpenSSL
      $env:CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "aarch64-linux-gnu-gcc"
      $env:PKG_CONFIG_ALLOW_CROSS = "1"
      $env:OPENSSL_STATIC = "1"

      # Cross-compile for ARM64
      cargo deb --manifest-path=cli/Cargo.toml --target=aarch64-unknown-linux-gnu --output=output
    } else {
      # Native compilation for AMD64
      cargo deb --manifest-path=cli/Cargo.toml --output=output
    }

    $artifactName = ls output
    $finalName = "EventStoreDB.Cloud.CLI-$Version-1.${Runner}_${Arch}.deb"
    Push-Location "output"
    Move-Item -Path $artifactName $finalName
    Write-Output "artifact_name" $finalName
    Write-Output "content_type" "application/vnd.debian.binary-package"
    Pop-Location
  }

  windows-2022
  {
    cargo build --bin esc --release
    Move-Item -Path (Join-Path "target" (Join-Path "release" "esc.exe")) (Join-Path "output" "esc.exe")
    Push-Location output
    $artifactName = "EventStoreDB.Cloud.CLI-Windows-$Arch-$Version.zip"
    Write-Output "artifact_name" $artifactName
    Write-Output "content_type" "application/zip"
    Compress-Archive -Path "esc.exe" -DestinationPath $artifactName
    Pop-Location
  }

  { $_ -in @("macos-13", "macos-14") }
  {
    cargo build --bin esc --release

    $archSuffix = if ($Arch -eq "arm64") { "-arm64" } else { "" }
    $packageName = "EventStoreDB.Cloud.CLI-OSX$archSuffix-$Version.pkg"
    $semVer = Get-Sem-Version $Version

    New-Item -Path . -Name "macbuild" -ItemType "directory"
    Copy-Item -Path "target/release/esc" "macbuild"

    pkgbuild --root macbuild --identifier com.eventstore.cloud.cli --ownership recommended --version $semVer --install-location /usr/local/bin "output/$packageName"

    Write-Output "artifact_name" $packageName
    Write-Output "content_type" "application/octet-stream"
  }
}
