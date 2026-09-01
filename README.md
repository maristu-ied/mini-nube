# Mini Nube

Sistema de almacenamiento local para datos de plantas solares. Ingesta datos CSV de dispositivos NCU, HSU y TCU en una base de datos SQLite.

## Estructura

```
mini_nube/
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

## Uso básico

```python
from mini_nube.db import conectar, inicializar
from mini_nube.ingesta import (
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

- [Esquema de base de datos](docs/esquema-bbdd.md)
