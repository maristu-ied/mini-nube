from pathlib import Path
from database.db import conectar, inicializar
from database.ingesta import ingestar_directorio, ingestar_fichero

# Definición de constantes
DB_PATH = "data/Poggiorsini/Poggiorsini.db"
REGENERATE_DB = False  # True para borrar y crear la base de datos desde cero
# REGENERATE_DB = True  # True para borrar y crear la base de datos desde cero

NCU_ID = "PR6-1075-DIMAURO"

PLANT_FOLDER = "data/Poggiorsini/PR6-1075-DIMAURO"
SKIP_FILES_ALREADY_INSERTED = True

SINGLEFILE_PATH = "data/Poggiorsini/PR6-1075-DIMAURO/2026-09-01/TCU_008_2026-09-01.csv"


if REGENERATE_DB:
        Path(DB_PATH).unlink(missing_ok=True)

conn = conectar(DB_PATH)
inicializar(conn)

# ingestar_directorio(conn, PLANT_FOLDER, NCU_ID, skip_files_already_inserted=SKIP_FILES_ALREADY_INSERTED)
ingestar_fichero(conn, Path(SINGLEFILE_PATH), NCU_ID)

# Resumen
cursor = conn.cursor()
cursor.execute("SELECT name FROM sqlite_master WHERE type='table';")
tablas = cursor.fetchall()
print("Resumen de la base de datos:")
for tabla in tablas:
    tabla_nombre = tabla[0]
    cursor.execute(f"SELECT COUNT(*) FROM {tabla_nombre};")
    num_filas = cursor.fetchone()[0]
    print(f"Tabla: {tabla_nombre}, Número de filas: {num_filas}")
conn.close()
