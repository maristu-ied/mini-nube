import argparse

from database.db import conectar, inicializar
from database.ingesta import ingestar_fichero


def main(db_path: str, ncu_id: str, singlefile_path: str) -> None:
	"""Inicializa la base de datos e ingesta un único fichero CSV."""
	conn = conectar(db_path)
	try:
		inicializar(conn)
		ingestar_fichero(conn, singlefile_path, ncu_id)
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
		"--file-path",
		dest="singlefile_path",
		default="../data/Poggiorsini/PR6-1075-DIMAURO/2026-09-01/TCU_008_2026-09-01.csv",
		help="Ruta del fichero CSV que se va a ingestar.",
	)
	return parser.parse_args()


if __name__ == "__main__":
	args = parse_args()
	main(
		db_path=args.db_path,
		ncu_id=args.ncu_id,
		singlefile_path=args.singlefile_path,
	)
