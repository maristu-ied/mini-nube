"""Gestión de la base de datos SQLite."""

import sqlite3
from pathlib import Path

from .schema import ALL_CREATE_STATEMENTS


def conectar(db_path: str | Path) -> sqlite3.Connection:
    """Abre (o crea) la base de datos y activa WAL + foreign keys."""
    conn = sqlite3.connect(str(db_path))
    conn.execute("PRAGMA journal_mode = WAL")
    conn.execute("PRAGMA foreign_keys = ON")
    conn.row_factory = sqlite3.Row
    return conn


def inicializar(conn: sqlite3.Connection) -> None:
    """Crea todas las tablas e índices si no existen."""
    for sql in ALL_CREATE_STATEMENTS:
        conn.execute(sql)
    conn.commit()


def obtener_o_crear_dispositivo(
    conn: sqlite3.Connection,
    ncu_id: str,
    tipo: str,
    device_id: str,
) -> int:
    """Devuelve el id del dispositivo, creándolo si no existe."""
    row = conn.execute(
        "SELECT id FROM dispositivos WHERE ncu_id = ? AND tipo = ? AND device_id = ?",
        (ncu_id, tipo, device_id),
    ).fetchone()
    if row:
        return row["id"]

    cur = conn.execute(
        "INSERT INTO dispositivos (ncu_id, tipo, device_id) VALUES (?, ?, ?)",
        (ncu_id, tipo, device_id),
    )
    return cur.lastrowid
