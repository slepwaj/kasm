# 构建 KASM 鸿蒙 HAP（PoC）
# 用法：在仓库根目录执行 pwsh -File apps/harmony/build.ps1
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot          # 仓库根
$project = Join-Path $root 'apps/harmony'
$deveco = 'D:\Huawei\DevEco Studio'
$sdk = if ($env:DEVECO_SDK_HOME) { $env:DEVECO_SDK_HOME } else { Join-Path $deveco 'sdk' }
$node = if ($env:NODE_HOME) { $env:NODE_HOME } else { Join-Path $deveco 'tools\node' }
$hvigor = Join-Path $deveco 'tools\hvigor\bin\hvigorw.bat'

# 1) Rust 静态库（仓库钉 1.96；OHOS 目标用 1.99.1 构建）
$env:RUSTUP_TOOLCHAIN = '1.99.1'
Push-Location $root
try {
    cargo build -p kasm-harmony-core --target aarch64-unknown-linux-ohos --release
    if ($LASTEXITCODE -ne 0) { throw 'cargo build 失败' }
} finally {
    Pop-Location
}

# 2) 静态库与词库
$libs = Join-Path $project 'entry/libs/arm64-v8a'
New-Item -ItemType Directory -Force $libs | Out-Null
Copy-Item (Join-Path $root 'target/aarch64-unknown-linux-ohos/release/libkasm_harmony_core.a') $libs -Force
$raw = Join-Path $project 'entry/src/main/resources/rawfile'
New-Item -ItemType Directory -Force $raw | Out-Null
Copy-Item (Join-Path $root 'assets/lexicon/dict.tsv') (Join-Path $raw 'dict.tsv') -Force

# 3) hvigor 要求 ASCII 路径：仓库路径含非 ASCII 时镜像到临时目录
$ascii = $project -match '^[\x00-\x7F]+$'
$work = if ($ascii) { $project } else { Join-Path $env:TEMP 'kasm-harmony-build' }
if (-not $ascii) {
    if (Test-Path $work) { Remove-Item $work -Recurse -Force }
    New-Item -ItemType Directory -Force $work | Out-Null
    Copy-Item (Join-Path $project '*') $work -Recurse -Force
    Copy-Item (Join-Path $project '.gitignore') $work -Force -ErrorAction SilentlyContinue
}

# 4) HAP
$env:NODE_HOME = $node
$env:DEVECO_SDK_HOME = $sdk
$env:PATH = "$node;$env:PATH"
Push-Location $work
try {
    & $hvigor --mode module -p product=default -p module=entry@default assembleHap
    if ($LASTEXITCODE -ne 0) { throw 'hvigor assembleHap 失败' }
} finally {
    Pop-Location
}

$hap = Get-ChildItem (Join-Path $work 'entry/build/default/outputs/default') -Filter '*.hap' | Select-Object -First 1
$dist = Join-Path $root 'dist'
New-Item -ItemType Directory -Force $dist | Out-Null
if ($hap) {
    Copy-Item $hap.FullName $dist -Force
    Write-Host "HAP: $(Join-Path $dist $hap.Name)"
} else {
    throw '没有找到 HAP'
}
