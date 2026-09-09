# Mini Nube

Sistema de almacenamiento local para datos de plantas solares. Ingesta datos CSV de dispositivos NCU, HSU y TCU en una base de datos SQLite.

## Estructura

```
database/
  __init__.py
  schema.py      # Definición de tablas e índices
  db.py          # Conexión e inicialización de la BD
  ingesta.py     # Parseo e inserción de CSV
tests/
  test_schema.py
  test_ingesta.py
  data/          # CSV de ejemplo para tests
docs/
  esquema-bbdd.md
```

## Requisitos

- Python >= 3.11
- [uv](https://docs.astral.sh/uv/)

## Instalación

```bash
uv sync
```

## Tests

```bash
uv run pytest -v
```

## Ejecución de los scripts principales

Los comandos siguientes se ejecutan desde esta carpeta (`python/`). Todos los
parámetros son opcionales y tienen valores por defecto, pero se muestran
explícitamente para documentar la configuración de la planta.

### Ingestar una carpeta

Ingesta todos los ficheros CSV de la carpeta indicada y sus subcarpetas:

```bash
uv run python main_ingest_folder.py \
  --db-path "../data/Poggiorsini/Poggiorsini.db" \
  --ncu-id "PR6-1075-DIMAURO" \
  --plant-folder "../data/Poggiorsini/PR6-1075-DIMAURO" \
  --skip-files-already-inserted
```

Para volver a procesar también los ficheros ya registrados:

```bash
uv run python main_ingest_folder.py \
  --db-path "../data/Poggiorsini/Poggiorsini.db" \
  --ncu-id "PR6-1075-DIMAURO" \
  --plant-folder "../data/Poggiorsini/PR6-1075-DIMAURO" \
  --no-skip-files-already-inserted
```

### Ingestar un único fichero

```bash
uv run python main_ingest_file.py \
  --db-path "../data/Poggiorsini/Poggiorsini.db" \
  --ncu-id "PR6-1075-DIMAURO" \
  --file-path "../data/Poggiorsini/PR6-1075-DIMAURO/2026-09-01/TCU_008_2026-09-01.csv"
```

### Mostrar el resumen de la base de datos

Muestra el número de filas de cada tabla:

```bash
uv run python main_resume_db.py \
  --db-path "../data/Poggiorsini/Poggiorsini.db"
```

### Ejecutar la ingesta y mostrar el resumen

El script `scripts/ingest_folder.ps1` ejecuta la ingesta de toda la carpeta y, si
termina correctamente, muestra el resumen de la base de datos. Sus parámetros
se definen como variables al principio del propio fichero.

```powershell
..\scripts\ingest_folder.ps1
```

Por defecto, `$DELETE_DATABASE = $false`. Para borrar la base de datos antes
de la ingesta, cambia esa variable a `$true`. También se borran los ficheros
auxiliares de SQLite (`-wal` y `-shm`) cuando existen.

## Uso básico

```python
from database.db import conectar, inicializar
from database.ingesta import (
    ingestar_ncu,
    ingestar_ncu_sensor,
    ingestar_hsu,
    ingestar_tcu,
    ingestar_event_log,
)

conn = conectar("planta.db")
inicializar(conn)

ncu_id = "NCU_PLANTA01"

ingestar_ncu(conn, "NCU_20260729.csv", ncu_id)
ingestar_ncu_sensor(conn, "NCU_SENSORS_20260901.csv", ncu_id)
ingestar_hsu(conn, "HSU_230_20260729.csv", ncu_id, "HSU_230")
ingestar_tcu(conn, "TCU_001_20260729.csv", ncu_id, "TCU_001")
ingestar_event_log(conn, "NCU_EVENT_LOG_20260729.csv", ncu_id)

conn.close()
```

## Documentación

- [Esquema de base de datos](../docs/esquema-bbdd.md)
