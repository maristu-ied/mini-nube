$ErrorActionPreference = "Stop"

# Parámetros de ejecución
$PROJECT_ROOT = Split-Path $PSScriptRoot -Parent
$RUST_ROOT = Join-Path $PROJECT_ROOT "rust"
$DB_PATH = Join-Path $PROJECT_ROOT "data\Poggiorsini\Poggiorsini_Rust.db"
$NCU_ID = "PR6-1075-DIMAURO"
$FILE_PATH = Join-Path $PROJECT_ROOT "data\Poggiorsini\PR6-1075-DIMAURO\2026-09-01\TCU_008_2026-09-01.csv"

Push-Location $RUST_ROOT
try {
    $ingestArguments = @(
        "run", "--bin", "main_ingest_file", "--",
        "--db-path", $DB_PATH,
        "--ncu-id", $NCU_ID,
        "--file-path", $FILE_PATH
    )

    Write-Host "Ingestando el fichero: $FILE_PATH"
    & cargo @ingestArguments
    if ($LASTEXITCODE -ne 0) {
        throw "La ingesta del fichero ha fallado con código $LASTEXITCODE."
    }

    Write-Host "Resumen de la base de datos:"
    & cargo run --bin main_resume_db -- --db-path $DB_PATH
    if ($LASTEXITCODE -ne 0) {
        throw "El resumen de la base de datos ha fallado con código $LASTEXITCODE."
    }
}
finally {
    Pop-Location
}
