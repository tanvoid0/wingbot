# WingBot LLM pipeline runner (Windows PowerShell)
param(
    [string]$Project = "wingbot-lol",
    [switch]$Train,
    [switch]$SkipDdragon
)

$ErrorActionPreference = "Stop"
$Core = Join-Path $PSScriptRoot "core"
$VenvPython = Join-Path $Core ".venv\Scripts\python.exe"

if (-not (Test-Path $VenvPython)) {
    python -m venv (Join-Path $Core ".venv")
    & (Join-Path $Core ".venv\Scripts\pip.exe") install -r (Join-Path $Core "requirements.txt")
}

$projScripts = Join-Path $PSScriptRoot "projects\$Project\scripts"

if (-not $SkipDdragon) {
    & $VenvPython (Join-Path $projScripts "extract_ddragon.py")
}
& $VenvPython (Join-Path $projScripts "export_wingbot_rules.py")
& $VenvPython (Join-Path $projScripts "generate_matchups.py")
& $VenvPython (Join-Path $Core "merge_knowledge.py") --project $Project
& $VenvPython (Join-Path $Core "build_dataset.py") --project $Project
& $VenvPython (Join-Path $Core "eval.py") --project $Project --offline

if ($Train) {
  & $VenvPython (Join-Path $Core "train_lora.py") --project $Project
  & $VenvPython (Join-Path $Core "export_ollama.py") --project $Project --skip-merge
}

Write-Host "Pipeline complete for $Project"
