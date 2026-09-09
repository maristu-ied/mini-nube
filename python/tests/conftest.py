"""Fixtures compartidos para los tests."""

import sqlite3
from pathlib import Path

import pytest

from database.db import conectar, inicializar

DATA_DIR = Path(__file__).parent / "data"
NCU_ID = "NCU_TEST_001"


@pytest.fixture
def db(tmp_path) -> sqlite3.Connection:
    """Crea una BD temporal inicializada."""
    conn = conectar(tmp_path / "test.db")
    inicializar(conn)
    yield conn
    conn.close()
