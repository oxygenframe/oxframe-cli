# Installs the `oxframe` CLI from GitHub releases on Windows.
#   powershell -ExecutionPolicy Bypass -c "irm https://github.com/oxygenframe/oxframe-cli/releases/latest/download/install.ps1 | iex"
# Env: OXFRAME_VERSION (default: latest), OXFRAME_INSTALL_DIR (default: %LOCALAPPDATA%\oxframe\bin).
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$repo = 'oxygenframe/oxframe-cli'
$dir = if ($env:OXFRAME_INSTALL_DIR) { $env:OXFRAME_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'oxframe\bin' }
if ($env:PROCESSOR_ARCHITECTURE -ne 'AMD64') {
  throw "unsupported architecture $($env:PROCESSOR_ARCHITECTURE); use: cargo install oxframe-cli"
}
$target = 'x86_64-pc-windows-msvc'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
if ($env:OXFRAME_VERSION) { $tag = $env:OXFRAME_VERSION } else {
  # The latest release from the redirect of github.com/.../releases/latest,
  # not the API: unauthenticated API calls from CI runners hit a rate limit.
  $req = [Net.WebRequest]::Create("https://github.com/$repo/releases/latest")
  $req.AllowAutoRedirect = $false
  $res = $req.GetResponse()
  $location = $res.Headers['Location']
  $res.Close()
  if (-not $location -or $location -notmatch '/tag/([^/]+)$') {
    throw "could not find the latest release ($location); set OXFRAME_VERSION"
  }
  $tag = $Matches[1]
}
$name = "oxframe-$tag-$target"
$base = "https://github.com/$repo/releases/download/$tag"
$tmp = Join-Path ([IO.Path]::GetTempPath()) ([Guid]::NewGuid())
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
  Write-Host "downloading oxframe $tag for $target"
  Invoke-WebRequest -UseBasicParsing "$base/$name.zip" -OutFile "$tmp\m.zip"
  Invoke-WebRequest -UseBasicParsing "$base/$name.zip.sha256" -OutFile "$tmp\m.sha256"
  $expected = ((Get-Content "$tmp\m.sha256" -Raw).Trim() -split '\s+')[0].ToLower()
  $actual = (Get-FileHash "$tmp\m.zip" -Algorithm SHA256).Hash.ToLower()
  if ($expected -ne $actual) { throw 'checksum mismatch' }
  Expand-Archive "$tmp\m.zip" -DestinationPath $tmp
  New-Item -ItemType Directory -Force -Path $dir | Out-Null
  Copy-Item "$tmp\$name\oxframe.exe" (Join-Path $dir 'oxframe.exe') -Force
} finally {
  Remove-Item -Recurse -Force $tmp
}
Write-Host "installed $(Join-Path $dir 'oxframe.exe')"
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($userPath -split ';') -notcontains $dir) {
  [Environment]::SetEnvironmentVariable('Path', ($userPath.TrimEnd(';') + ";$dir").TrimStart(';'), 'User')
  Write-Host "added $dir to your PATH; open a new terminal to use oxframe"
}
