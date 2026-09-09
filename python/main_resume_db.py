import argparse

from database.db import conectar


def main(db_path: str) -> None:
    """Muestra el número de filas de cada tabla de la base de datos."""
    conn = conectar(db_path)
    try:
        cursor = conn.cursor()
        cursor.execute("SELECT name FROM sqlite_master WHERE type='table';")
        tablas = cursor.fetchall()
        print("Resumen de la base de datos:")
        for tabla in tablas:
            tabla_nombre = tabla[0]
            cursor.execute(f"SELECT COUNT(*) FROM {tabla_nombre};")
            num_filas = cursor.fetchone()[0]
            print(f"Tabla: {tabla_nombre}, Número de filas: {num_filas}")
    finally:
        conn.close()


def parse_args() -> argparse.Namespace:
    """Obtiene los parámetros de ejecución desde la línea de comandos."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--db-path",
        default="../data/Poggiorsini/Poggiorsini.db",
        help="Ruta de la base de datos SQLite.",
    )
    return parser.parse_args()


if __name__ == "__main__":
    args = parse_args()
    main(db_path=args.db_path)
