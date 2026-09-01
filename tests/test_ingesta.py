"""Tests de ingesta de ficheros CSV reales."""

from calendar import timegm
from pathlib import Path
from time import strptime

import pytest


def _ts(s: str) -> int:
    """Helper: convierte 'YYYY-MM-DD HH:MM:SS' UTC a Unix epoch."""
    return timegm(strptime(s, "%Y-%m-%d %H:%M:%S"))

from mini_nube.ingesta import (
    ingestar_event_log,
    ingestar_hsu,
    ingestar_ncu,
    ingestar_ncu_sensor,
    ingestar_tcu,
)

DATA_DIR = Path(__file__).parent / "data"
NCU_ID = "NCU_TEST_001"


class TestIngestaNcu:
    def test_ingesta_ncu(self, db):
        filas = ingestar_ncu(db, DATA_DIR / "NCU_20260729.csv", NCU_ID)
        assert filas > 0

        count = db.execute("SELECT COUNT(*) FROM datos_ncu").fetchone()[0]
        assert count == filas

    def test_valores_ncu(self, db):
        ingestar_ncu(db, DATA_DIR / "NCU_20260729.csv", NCU_ID)
        row = db.execute(
            "SELECT * FROM datos_ncu ORDER BY timestamp LIMIT 1"
        ).fetchone()
        assert row["timestamp"] == _ts("2026-07-29 00:00:00")
        assert row["mqtt_online"] == 0  # false -> 0
        assert row["gw1_online"] == 1   # true -> 1


class TestIngestaHsu:
    def test_ingesta_hsu(self, db):
        filas = ingestar_hsu(db, DATA_DIR / "HSU_230_20260729.csv", NCU_ID, "HSU_230")
        assert filas > 0

        count = db.execute("SELECT COUNT(*) FROM datos_hsu").fetchone()[0]
        assert count == filas

    def test_valores_hsu(self, db):
        ingestar_hsu(db, DATA_DIR / "HSU_230_20260729.csv", NCU_ID, "HSU_230")
        row = db.execute(
            "SELECT * FROM datos_hsu ORDER BY timestamp LIMIT 1"
        ).fetchone()
        assert row["timestamp"] == _ts("2026-07-29 00:00:00")
        assert row["main_battery"] == 14847
        assert row["internal_temp"] == pytest.approx(19.85)
        assert row["wind_alarm"] == 0


class TestIngestaNcuSensor:
    def test_ingesta_ncu_sensor(self, db):
        filas = ingestar_ncu_sensor(db, DATA_DIR / "NCU_SENSORS_20260901.csv", NCU_ID)
        assert filas > 0

        count = db.execute("SELECT COUNT(*) FROM datos_ncu_sensor").fetchone()[0]
        assert count == filas

    def test_valores_ncu_sensor(self, db):
        ingestar_ncu_sensor(db, DATA_DIR / "NCU_SENSORS_20260901.csv", NCU_ID)
        row = db.execute(
            "SELECT * FROM datos_ncu_sensor ORDER BY timestamp LIMIT 1"
        ).fetchone()
        assert row["main_battery"] == 18299
        assert row["internal_temp"] == pytest.approx(33.85)


class TestIngestaTcu:
    def test_ingesta_tcu(self, db):
        filas = ingestar_tcu(db, DATA_DIR / "TCU_001_20260729.csv", NCU_ID, "TCU_001")
        assert filas > 0

        count = db.execute("SELECT COUNT(*) FROM datos_tcu").fetchone()[0]
        assert count == filas

    def test_valores_tcu(self, db):
        ingestar_tcu(db, DATA_DIR / "TCU_001_20260729.csv", NCU_ID, "TCU_001")
        row = db.execute(
            "SELECT * FROM datos_tcu ORDER BY timestamp LIMIT 1"
        ).fetchone()
        assert row["timestamp"] == _ts("2026-07-29 00:00:08")
        assert row["main_state"] == "AUTO"
        assert row["backtracking"] == 0
        assert row["angle"] == pytest.approx(5.20)
        assert row["soc"] == 84
        assert row["motor_state"] == "OFF"


class TestIngestaEventLog:
    def test_ingesta_event_log(self, db):
        filas = ingestar_event_log(
            db, DATA_DIR / "NCU_EVENT_LOG_20260729.csv", NCU_ID
        )
        assert filas > 0

        count = db.execute("SELECT COUNT(*) FROM ncu_event_log").fetchone()[0]
        assert count == filas

    def test_valores_event_log(self, db):
        ingestar_event_log(db, DATA_DIR / "NCU_EVENT_LOG_20260729.csv", NCU_ID)
        row = db.execute(
            "SELECT * FROM ncu_event_log ORDER BY timestamp LIMIT 1"
        ).fetchone()
        assert row["timestamp"] == _ts("2026-07-29 06:48:27")
        assert "OTA performed" in row["evento"]


class TestIngestaLog:
    def test_registro_ingesta(self, db):
        ingestar_ncu(db, DATA_DIR / "NCU_20260729.csv", NCU_ID)
        log = db.execute("SELECT * FROM ingesta_log").fetchone()
        assert log["ncu_id"] == NCU_ID
        assert log["tipo_datos"] == "datos_ncu"
        assert log["filas_insertadas"] > 0
        assert log["timestamp_inicio"] is not None
        assert log["timestamp_fin"] is not None


class TestDispositivosCreados:
    def test_dispositivos_tras_ingesta_completa(self, db):
        """Tras ingestar todo, la tabla dispositivos tiene los registros correctos."""
        ingestar_ncu(db, DATA_DIR / "NCU_20260729.csv", NCU_ID)
        ingestar_ncu_sensor(db, DATA_DIR / "NCU_SENSORS_20260901.csv", NCU_ID)
        ingestar_hsu(db, DATA_DIR / "HSU_230_20260729.csv", NCU_ID, "HSU_230")
        ingestar_tcu(db, DATA_DIR / "TCU_001_20260729.csv", NCU_ID, "TCU_001")
        ingestar_event_log(db, DATA_DIR / "NCU_EVENT_LOG_20260729.csv", NCU_ID)

        dispositivos = db.execute(
            "SELECT ncu_id, tipo, device_id FROM dispositivos ORDER BY tipo, device_id"
        ).fetchall()

        tipos = {(d["tipo"], d["device_id"]) for d in dispositivos}
        assert ("NCU", NCU_ID) in tipos
        assert ("HSU", "HSU_230") in tipos
        assert ("TCU", "TCU_001") in tipos

    def test_multiples_ncus(self, db):
        """Datos de dos NCUs distintas coexisten sin conflicto."""
        ingestar_ncu(db, DATA_DIR / "NCU_20260729.csv", "NCU_A")
        ingestar_ncu(db, DATA_DIR / "NCU_20260729.csv", "NCU_B")

        count = db.execute("SELECT COUNT(DISTINCT ncu_id) FROM dispositivos").fetchone()[0]
        assert count == 2

        # Cada NCU tiene sus propios registros
        for ncu in ("NCU_A", "NCU_B"):
            disp = db.execute(
                "SELECT id FROM dispositivos WHERE ncu_id = ?", (ncu,)
            ).fetchone()
            filas = db.execute(
                "SELECT COUNT(*) FROM datos_ncu WHERE dispositivo_id = ?",
                (disp["id"],),
            ).fetchone()[0]
            assert filas > 0
