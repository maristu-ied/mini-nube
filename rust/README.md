# Mini Nube en Rust

Implementación en Rust del sistema de almacenamiento e ingesta local para datos de plantas solares (NCU, HSU, TCU, Event Logs) en SQLite.

## Estructura del Workspace

El proyecto está organizado como un Cargo Workspace modular con 3 crates:

```text
rust/
├── Cargo.toml                  # Configuración raíz del workspace
├── crates/
│   ├── csv_reader/             # Crate para lectura y parseo de ficheros CSV (mini-nube-csv)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models.rs       # Structs de registros (Ncu, Sensor, Tcu, EventLog) y detección de tipos
│   │       ├── reader.rs       # Lectores optimizados para cada formato CSV
│   │       └── error.rs
│   ├── db/                     # Crate para gestión de SQLite e ingesta (mini-nube-db)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── schema.rs       # DDL de tablas e índices UNIQUE
│   │   │   ├── connection.rs   # Conexión con WAL y foreign keys
│   │   │   ├── repository.rs   # Gestión de dispositivos, idempotencia y resumen
│   │   │   ├── ingesta.rs      # Ingesta individual o por lotes/carpetas con transacciones
│   │   │   └── error.rs
│   │   └── tests/              # Tests de integración (esquema e ingesta con datos reales)
│   └── cli/                    # Crate binario / interfaz de línea de comandos (mini-nube-cli)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs         # CLI unificado (mini-nube) con subcomandos
│           └── bin/
│               ├── main_ingest_file.rs    # Réplica 1:1 de main_ingest_file.py
│               ├── main_ingest_folder.rs  # Réplica 1:1 de main_ingest_folder.py
│               └── main_resume_db.rs      # Réplica 1:1 de main_resume_db.py
```

## Ejecución

### 1. Ingestar un único fichero

```bash
cargo run --bin main_ingest_file -- --db-path ../data/Poggiorsini/Poggiorsini_Rust.db --ncu-id PR6-1075-DIMAURO --file-path ../data/Poggiorsini/PR6-1075-DIMAURO/2026-09-01/TCU_008_2026-09-01.csv
```

### 2. Ingestar una carpeta completa

```bash
cargo run --release --bin main_ingest_folder -- --db-path ../data/Poggiorsini/Poggiorsini_Rust.db --ncu-id PR6-1075-DIMAURO --plant-folder ../data/Poggiorsini/PR6-1075-DIMAURO
```

### 3. Ver resumen de la base de datos

```bash
cargo run --bin main_resume_db -- --db-path ../data/Poggiorsini/Poggiorsini_Rust.db
```

### 4. CLI Unificado

```bash
cargo run --bin mini-nube -- ingest-file --db-path ... --ncu-id ... --file-path ...
cargo run --bin mini-nube -- ingest-folder --db-path ... --ncu-id ... --plant-folder ...
cargo run --bin mini-nube -- resume-db --db-path ...
```

## Pruebas

Para ejecutar la suite de tests completa:

```bash
cargo test
```
