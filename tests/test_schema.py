"""Tests del esquema de base de datos."""

from mini_nube.db import conectar, inicializar, obtener_o_crear_dispositivo


def test_crear_tablas(db):
    """Verifica que se crean todas las tablas esperadas."""
    tablas = {
        row[0]
        for row in db.execute(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        ).fetchall()
    }
    esperadas = {
        "dispositivos",
        "datos_ncu",
        "datos_ncu_sensor",
        "datos_hsu",
        "datos_tcu",
        "ncu_event_log",
        "ingesta_log",
    }
    assert esperadas == tablas


def test_crear_indices(db):
    """Verifica que se crean los índices de timestamp."""
    indices = {
        row[0]
        for row in db.execute(
            "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'"
        ).fetchall()
    }
    esperados = {
        "idx_datos_ncu_ts",
        "idx_datos_ncu_sensor_ts",
        "idx_datos_hsu_ts",
        "idx_datos_tcu_ts",
        "idx_ncu_event_log_ts",
    }
    assert esperados == indices


def test_dispositivo_unico(db):
    """Un mismo dispositivo no se duplica."""
    id1 = obtener_o_crear_dispositivo(db, "NCU_001", "HSU", "HSU_230")
    id2 = obtener_o_crear_dispositivo(db, "NCU_001", "HSU", "HSU_230")
    assert id1 == id2


def test_dispositivos_distintos(db):
    """Dispositivos con distinto tipo/id son diferentes."""
    id1 = obtener_o_crear_dispositivo(db, "NCU_001", "HSU", "HSU_230")
    id2 = obtener_o_crear_dispositivo(db, "NCU_001", "TCU", "TCU_001")
    id3 = obtener_o_crear_dispositivo(db, "NCU_002", "HSU", "HSU_230")
    assert id1 != id2
    assert id1 != id3


def test_wal_mode(db):
    """Verifica que WAL está activo."""
    mode = db.execute("PRAGMA journal_mode").fetchone()[0]
    assert mode == "wal"


def test_foreign_keys(db):
    """Verifica que las foreign keys están activas."""
    fk = db.execute("PRAGMA foreign_keys").fetchone()[0]
    assert fk == 1
