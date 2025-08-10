param(
    [Parameter(Mandatory=$true)][string]$SourceEnv,   # 例: C:\Users\Keiji\StudioKeKe\profinaut\.env
    [Parameter(Mandatory=$true)][string]$TargetEnv,   # 例: C:\Users\Keiji\StudioKeKe\profinaut\docker\vault\.env
    [Parameter(Mandatory=$true)][string]$AppendEnv    # 例: C:\Users\Keiji\StudioKeKe\profinaut\docker\vault\env.generated
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (!(Test-Path $SourceEnv)) { throw "SourceEnv not found: $SourceEnv" }
if (!(Test-Path (Split-Path $TargetEnv -Parent))) { New-Item -ItemType Directory -Path (Split-Path $TargetEnv -Parent) | Out-Null }
if (!(Test-Path $AppendEnv)) { throw "AppendEnv not found: $AppendEnv" }

# 1) マスターをコピー（既存があればバックアップ）
if (Test-Path $TargetEnv) {
    Copy-Item $TargetEnv "$TargetEnv.bak.$((Get-Date).ToString('yyyyMMddHHmmss'))" -Force
}
Copy-Item $SourceEnv $TargetEnv -Force

# 2) 追記するキー抽出
$appendLines = Get-Content $AppendEnv | Where-Object { $_ -match '^[A-Za-z_][A-Za-z0-9_]*=' }
$keys = $appendLines | ForEach-Object { ($_ -split '=',2)[0] } | Sort-Object -Unique

# 3) 既存キー削除
$dstLines = Get-Content $TargetEnv
$filtered = foreach ($line in $dstLines) {
    if ($line -match '^[A-Za-z_][A-Za-z0-9_]*=') {
        $key = ($line -split '=',2)[0]
        if ($keys -contains $key) { continue } # drop
    }
    $line
}

# 4) 追記
$header = "# ---- appended by Merge-Env.ps1 ($(Get-Date -Format s)) ----"
$merged = @($filtered) + "" + $header + (Get-Content $AppendEnv)
$merged | Set-Content -Encoding UTF8 $TargetEnv

Write-Host "✅ merged: $SourceEnv + $AppendEnv → $TargetEnv"
