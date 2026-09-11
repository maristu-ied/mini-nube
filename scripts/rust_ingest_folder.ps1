$ErrorActionPreference = "Stop"

# Parámetros de ejecución
$PROJECT_ROOT = Split-Path $PSScriptRoot -Parent
$RUST_ROOT = Join-Path $PROJECT_ROOT "rust"
$DB_PATH = Join-Path $PROJECT_ROOT "data\Poggiorsini\Poggiorsini_Rust.db"
$NCU_ID = "PR6-1075-DIMAURO"
$PLANT_FOLDER = Join-Path $PROJECT_ROOT "data\Poggiorsini\PR6-1075-DIMAURO"
$SKIP_FILES_ALREADY_INSERTED = $true
$DELETE_DATABASE = $true

Push-Location $RUST_ROOT
try {
    if ($DELETE_DATABASE) {
        foreach ($databaseFile in @($DB_PATH, "$DB_PATH-wal", "$DB_PATH-shm")) {
            if (Test-Path -LiteralPath $databaseFile) {
                Write-Host "Borrando: $databaseFile"
                Remove-Item -LiteralPath $databaseFile -Force
            }
        }
    }

    $ingestArguments = @(
        "run", "--release", "--bin", "main_ingest_folder", "--",
        "--db-path", $DB_PATH,
        "--ncu-id", $NCU_ID,
        "--plant-folder", $PLANT_FOLDER,
        "--skip-files-already-inserted", "$SKIP_FILES_ALREADY_INSERTED"
    )

    Write-Host "Ingestando la carpeta: $PLANT_FOLDER"
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    & cargo @ingestArguments
    $stopwatch.Stop()

    if ($LASTEXITCODE -ne 0) {
        throw "La ingesta de la carpeta ha fallado con código $LASTEXITCODE."
    }

    $elapsed = $stopwatch.Elapsed
    Write-Host ("Tiempo de ingesta: {0:mm\:ss\.fff} ({1:N2} segundos)" -f $elapsed, $elapsed.TotalSeconds)

    Write-Host "Resumen de la base de datos:"
    & cargo run --release --bin main_resume_db -- --db-path $DB_PATH
    if ($LASTEXITCODE -ne 0) {
        throw "El resumen de la base de datos ha fallado con código $LASTEXITCODE."
    }
}
finally {
    Pop-Location
}
