# Install NeoNexus as a Windows service.
#
# Run from an elevated PowerShell. The service account and paths are explicit
# inputs; credentials are not accepted by this script and must be provided by
# the service's external environment/secret mechanism.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)] [string]$BinaryPath,
    [string]$DataDir = "$env:ProgramData\NeoNexus",
    [string]$ServiceName = "NeoNexus",
    [string]$DisplayName = "NeoNexus Node Operations Workbench",
    [string]$Account = "NT AUTHORITY\LocalService"
)

$ErrorActionPreference = "Stop"
if (-not (Test-Path -LiteralPath $BinaryPath -PathType Leaf)) {
    throw "NeoNexus binary does not exist: $BinaryPath"
}
New-Item -ItemType Directory -Force -Path $DataDir | Out-Null

# sc.exe receives an argv-style quoted executable path, not a shell command.
# The service itself reads credentials from the configured machine/service
# environment; no token is interpolated here.
$quotedBinary = '"' + (Resolve-Path -LiteralPath $BinaryPath).Path + '" --web --bind 127.0.0.1 --port 8080'
& sc.exe create $ServiceName binPath= $quotedBinary start= auto DisplayName= $DisplayName obj= $Account
if ($LASTEXITCODE -ne 0) { throw "sc.exe create failed with exit code $LASTEXITCODE" }
& sc.exe failure $ServiceName actions= restart/5000/restart/5000/none/0 reset= 86400
if ($LASTEXITCODE -ne 0) { throw "sc.exe failure policy failed with exit code $LASTEXITCODE" }
Write-Output "Installed $ServiceName. Start it with: sc.exe start $ServiceName"
