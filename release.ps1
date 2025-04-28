[CmdletBinding()]
Param(
  [Parameter(Mandatory=$true)]
  [ValidateSet("windows-2019", "ubuntu-22.04", "macos-13")]
  [string]$Runner,

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
  ubuntu-20.04
  {
    cargo install cargo-deb
    cargo deb --manifest-path=cli/Cargo.toml --output=output
    $artifactName = ls output
    $finalName = "EventStoreDB.Cloud.CLI-$Version-1.${Runner}_amd64.deb"
    Push-Location "output"
    Move-Item -Path $artifactName $finalName
    Write-Output "artifact_name" $finalName
    Write-Output "content_type" "application/vnd.debian.binary-package"
    Pop-Location
  }

  windows-2019
  {
    cargo build --bin esc --release
    Move-Item -Path (Join-Path "target" (Join-Path "release" "esc.exe")) (Join-Path "output" "esc.exe")
    Push-Location output
    $artifactName = "EventStoreDB.Cloud.CLI-Windows-x64-$Version.zip"
    Write-Output "artifact_name" $artifactName
    Write-Output "content_type" "application/zip"
    Compress-Archive -Path "esc.exe" -DestinationPath $artifactName
    Pop-Location
  }

  macos-11
  {
    cargo build --bin esc --release

    $packageName = "EventStoreDB.Cloud.CLI-OSX-$Version.pkg"
    $semVer = Get-Sem-Version $Version

    New-Item -Path . -Name "macbuild" -ItemType "directory"
    Copy-Item -Path "target/release/esc" "macbuild"

    pkgbuild --root macbuild --identifier com.eventstore.cloud.cli --ownership recommended --version $semVer --install-location /usr/local/bin "output/$packageName"

    Write-Output "artifact_name" $packageName
    Write-Output "content_type" "application/octet-stream"
  }
}
