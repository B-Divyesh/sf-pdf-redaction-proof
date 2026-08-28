$ErrorActionPreference = "Stop"
$repo = "B-Divyesh/sf-pdf-redaction-proof"
$base = "https://github.com/$repo/releases/latest/download"
$temp = Join-Path ([System.IO.Path]::GetTempPath()) ("redaction-proof-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $temp | Out-Null
try {
  Invoke-WebRequest "$base/SHA256SUMS" -OutFile (Join-Path $temp "SHA256SUMS")
  $line = Get-Content (Join-Path $temp "SHA256SUMS") | Where-Object { $_ -match '\.msi$' } | Select-Object -First 1
  if (-not $line) { throw "No Windows MSI is available in the latest release." }
  $parts = $line -split '\s+', 2
  $expected = $parts[0].ToLower()
  $asset = $parts[1].Trim()
  $target = Join-Path $temp $asset
  Invoke-WebRequest "$base/$asset" -OutFile $target
  $actual = (Get-FileHash -Algorithm SHA256 $target).Hash.ToLower()
  if ($actual -ne $expected) { throw "SHA-256 mismatch; the installer was not run." }
  Write-Host "SHA-256 verified. Starting the unsigned Redaction Proof installer..."
  Start-Process msiexec.exe -ArgumentList "/i `"$target`"" -Wait
  Write-Host "Redaction Proof installer finished."
} finally {
  Remove-Item -Recurse -Force $temp -ErrorAction SilentlyContinue
}
