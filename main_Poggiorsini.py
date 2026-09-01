from pathlib import Path

from mini_nube.db import conectar, inicializar
from mini_nube.ingesta import (
    ingestar_ncu,
    ingestar_ncu_sensor,
    ingestar_hsu,
    ingestar_tcu,
    ingestar_event_log,
)

# Definición de constantes
PLANT_FOLDER = "data/Poggiorsini"
DB_PATH = f"{PLANT_FOLDER}/Poggiorsini.db"

NCU_ID = "PR6-1075-DIMAURO"


def ingestar_planta(plant_folder: str, ncu_id: str, db_path: str, skip_files_already_inserted: bool, regenerate_db: bool = False) -> None:
    """Escanea plant_folder y sus subcarpetas, ingestando cada CSV según su nombre."""
    if regenerate_db:
        Path(db_path).unlink(missing_ok=True)

    conn = conectar(db_path)
    inicializar(conn)

    # escanea todos los CSV en plant_folder y subcarpetas
    print(f"Escaneando carpeta: {plant_folder}")
    csv_files = list(Path(plant_folder).rglob("*.csv"))

    print(f"Archivos CSV encontrados: {len(csv_files)}")

    # El orden importa: NCU_EVENT_LOG_ y NCU_SENSORS_ deben comprobarse antes que NCU_.
    for csv_path in sorted(csv_files):
        nombre = csv_path.name

        # comprobar si el archivo ya ha sido insertado con el correspondiente ncu_id
        if skip_files_already_inserted:
            cursor = conn.cursor()
            cursor.execute(
                "SELECT COUNT(*) FROM ingesta_log WHERE fichero=? AND ncu_id=?",
                (nombre, ncu_id),
            )
            if cursor.fetchone()[0] > 0:
                # print(f"Archivo {nombre} ya ha sido insertado para NCU_ID {ncu_id}. Saltando.")
                continue

        # print(f"Ingestando archivo: {csv_path}")

        if nombre.startswith("NCU_EVENT_LOG_"):
            ingestar_event_log(conn, csv_path, ncu_id)
        elif nombre.startswith("NCU_SENSORS_"):
            ingestar_ncu_sensor(conn, csv_path, ncu_id)
        elif nombre.startswith("NCU_"):
            ingestar_ncu(conn, csv_path, ncu_id)
        elif nombre.startswith("HSU_"):
            hsu_id = "_".join(nombre.split("_")[:2])
            ingestar_hsu(conn, csv_path, ncu_id, hsu_id)
        elif nombre.startswith("TCU_"):
            tcu_id = "_".join(nombre.split("_")[:2])
            ingestar_tcu(conn, csv_path, ncu_id, tcu_id)

    conn.close()


ingestar_planta(PLANT_FOLDER, NCU_ID, DB_PATH, skip_files_already_inserted=True, regenerate_db=False)

# Resumen
# conectar a la bbdd ver las tablas y contar el número de filas de cada tabla
conn = conectar(DB_PATH)
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
