import argparse

from database.db import conectar, inicializar
from database.ingesta import ingestar_directorio


def main(
	db_path: str,
	ncu_id: str,
	plant_folder: str,
	skip_files_already_inserted: bool,
) -> None:
	"""Inicializa la base de datos e ingesta todos los CSV de una carpeta."""
	conn = conectar(db_path)
	try:
		inicializar(conn)
		ingestar_directorio(
			conn,
			plant_folder,
			ncu_id,
			skip_files_already_inserted=skip_files_already_inserted,
		)
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
	parser.add_argument(
		"--ncu-id",
		default="PR6-1075-DIMAURO",
		help="Identificador de la NCU.",
	)
	parser.add_argument(
		"--plant-folder",
		default="../data/Poggiorsini/PR6-1075-DIMAURO",
		help="Carpeta que contiene los CSV de la planta.",
	)
	parser.add_argument(
		"--skip-files-already-inserted",
		action=argparse.BooleanOptionalAction,
		default=True,
		help="Omite los ficheros ya registrados en la tabla de ingesta.",
	)
	return parser.parse_args()


if __name__ == "__main__":
	args = parse_args()
	main(
		db_path=args.db_path,
		ncu_id=args.ncu_id,
		plant_folder=args.plant_folder,
		skip_files_already_inserted=args.skip_files_already_inserted,
	)
