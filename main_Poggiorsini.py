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


def ingestar_planta(plant_folder: str, db_path: str) -> None:
    """Escanea plant_folder y sus subcarpetas, ingestando cada CSV según su nombre."""
    Path(db_path).unlink(missing_ok=True)

    conn = conectar(db_path)
    inicializar(conn)

    # El orden importa: NCU_EVENT_LOG_ y NCU_SENSORS_ deben comprobarse antes que NCU_.
    for csv_path in sorted(Path(plant_folder).rglob("*.csv")):
        nombre = csv_path.name

        if nombre.startswith("NCU_EVENT_LOG_"):
            ingestar_event_log(conn, csv_path, NCU_ID)
        elif nombre.startswith("NCU_SENSORS_"):
            ingestar_ncu_sensor(conn, csv_path, NCU_ID)
        elif nombre.startswith("NCU_"):
            ingestar_ncu(conn, csv_path, NCU_ID)
        elif nombre.startswith("HSU_"):
            hsu_id = "_".join(nombre.split("_")[:2])
            ingestar_hsu(conn, csv_path, NCU_ID, hsu_id)
        elif nombre.startswith("TCU_"):
            tcu_id = "_".join(nombre.split("_")[:2])
            ingestar_tcu(conn, csv_path, NCU_ID, tcu_id)

    conn.close()


ingestar_planta(PLANT_FOLDER, DB_PATH)
