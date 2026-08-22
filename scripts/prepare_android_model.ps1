$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$cppDir = Join-Path $repoRoot "android/app/src/main/cpp"
$llamaDir = Join-Path $cppDir "llama.cpp"
$assetDir = Join-Path $repoRoot "android/app/src/main/assets/models"
$model = Join-Path $assetDir "SmolLM2-135M-Instruct-Q2_K.gguf"

$llamaCommit = "9a286ac98d2cab74231bd3f1fc3f2b8bdf05422e"
$modelUrl = "https://huggingface.co/tensorblock/SmolLM2-135M-Instruct-GGUF/resolve/main/SmolLM2-135M-Instruct-Q2_K.gguf?download=true"

New-Item -ItemType Directory -Force $cppDir | Out-Null
New-Item -ItemType Directory -Force $assetDir | Out-Null

if (-not (Test-Path (Join-Path $llamaDir "CMakeLists.txt"))) {
    if (Test-Path $llamaDir) {
        Remove-Item -Recurse -Force $llamaDir
    }
    git clone https://github.com/ggml-org/llama.cpp.git $llamaDir
    Push-Location $llamaDir
    git checkout $llamaCommit
    Pop-Location
}

if (-not (Test-Path $model)) {
    Invoke-WebRequest -Uri $modelUrl -OutFile $model
}

if ((Get-Item $model).Length -lt 80000000) {
    throw "Downloaded model is unexpectedly small. Delete it and rerun this script."
}

Write-Host "Android native AI dependencies are ready."
Write-Host "llama.cpp: $llamaDir"
Write-Host "model: $model"
