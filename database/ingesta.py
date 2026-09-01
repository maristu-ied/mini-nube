"""Ingesta de ficheros CSV en la base de datos."""

import csv
import sqlite3
from calendar import timegm
from pathlib import Path
from time import strptime

from .db import obtener_o_crear_dispositivo


def _parse_ts(val: str) -> int:
    """Convierte 'YYYY-MM-DD HH:MM:SS' (UTC) a Unix epoch (segundos)."""
    return timegm(strptime(val.strip(), "%Y-%m-%d %H:%M:%S"))


def _parse_bool(val: str) -> int:
    """Convierte 'true'/'false' a 1/0."""
    return 1 if val.strip().lower() == "true" else 0


def _parse_int(val: str) -> int | None:
    """Convierte a int, None si vacío."""
    val = val.strip()
    return int(val) if val else None


def _parse_float(val: str) -> float | None:
    """Convierte a float, None si vacío."""
    val = val.strip()
    return float(val) if val else None


# ---------------------------------------------------------------------------
# Ingesta de cada tipo de fichero
# ---------------------------------------------------------------------------

def ingestar_fichero(conn: sqlite3.Connection, csv_path: str | Path, ncu_id: str) -> None:
    """Ingesta un único CSV según el prefijo de su nombre."""
    csv_path = Path(csv_path)
    nombre = csv_path.name

    # El orden importa: NCU_EVENT_LOG_ y NCU_SENSORS_ deben comprobarse antes que NCU_.
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


def ingestar_directorio(conn: sqlite3.Connection, plant_folder: str | Path, ncu_id: str, skip_files_already_inserted: bool) -> None:
    """Escanea plant_folder y sus subcarpetas, ingestando cada CSV según su nombre."""
    print(f"Escaneando carpeta: {plant_folder}")
    csv_files = list(Path(plant_folder).rglob("*.csv"))

    print(f"Archivos CSV encontrados: {len(csv_files)}")

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

        ingestar_fichero(conn, csv_path, ncu_id)


def ingestar_ncu(
    conn: sqlite3.Connection,
    csv_path: str | Path,
    ncu_id: str,
) -> int:
    """Ingesta un fichero NCU (estado general). Devuelve filas procesadas."""
    csv_path = Path(csv_path)
    disp_id = obtener_o_crear_dispositivo(conn, ncu_id, "NCU", ncu_id)

    with open(csv_path, newline="") as f:
        reader = csv.DictReader(f, delimiter=";")
        rows = []
        for row in reader:
            rows.append((
                disp_id,
                _parse_ts(row["datetime"]),
                _parse_bool(row["mqtt_online"]),
                _parse_bool(row["gw1_online"]),
                _parse_bool(row["gw2_online"]),
                _parse_bool(row["ups_power_ok"]),
                _parse_bool(row["ups_battery_low"]),
                _parse_bool(row["stop_button"]),
                _parse_bool(row["bluetooth_enabled"]),
            ))

    ya_existentes = _contar_existentes(conn, "datos_ncu", disp_id, [r[1] for r in rows])
    nuevos = len(rows) - ya_existentes

    conn.executemany(
        """INSERT OR REPLACE INTO datos_ncu
           (dispositivo_id, timestamp, mqtt_online, gw1_online, gw2_online,
            ups_power_ok, ups_battery_low, stop_button, bluetooth_enabled)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
        rows,
    )
    conn.commit()

    _registrar_ingesta(conn, ncu_id, csv_path.name, "datos_ncu", nuevos, ya_existentes, rows)
    return len(rows)


def _contar_existentes(conn: sqlite3.Connection, tabla: str, disp_id: int, timestamps: list[int]) -> int:
    """Cuenta cuántos de los (dispositivo_id, timestamp) dados ya existen en la tabla."""
    total = 0
    for i in range(0, len(timestamps), 900):  # límite de parámetros de SQLite
        chunk = timestamps[i:i + 900]
        placeholders = ",".join("?" * len(chunk))
        total += conn.execute(
            f"SELECT COUNT(*) FROM {tabla} WHERE dispositivo_id = ? AND timestamp IN ({placeholders})",
            (disp_id, *chunk),
        ).fetchone()[0]
    return total


def _ingestar_sensor(
    conn: sqlite3.Connection,
    csv_path: str | Path,
    ncu_id: str,
    tipo: str,
    device_id: str,
    tabla: str,
) -> int:
    """Ingesta genérica para datos de sensores (HSU y NCU_SENSOR comparten esquema)."""
    csv_path = Path(csv_path)
    disp_id = obtener_o_crear_dispositivo(conn, ncu_id, tipo, device_id)

    with open(csv_path, newline="") as f:
        reader = csv.DictReader(f, delimiter=";")
        rows = []
        for row in reader:
            rows.append((
                disp_id,
                _parse_ts(row["datetime"]),
                _parse_int(row["main_battery"]),
                _parse_float(row["internal_temp"]),
                _parse_float(row["wind_speed"]),
                _parse_float(row["wind_direction"]),
                _parse_int(row["wind_level"]),
                _parse_float(row["snow_level"]),
                _parse_int(row["irradiance"]),
                _parse_bool(row["wind_alarm"]),
                _parse_bool(row["gusty_wind_alarm"]),
                _parse_bool(row["snow_alarm"]),
                _parse_bool(row["snow_sensor_com_error"]),
            ))

    ya_existentes = _contar_existentes(conn, tabla, disp_id, [r[1] for r in rows])
    nuevos = len(rows) - ya_existentes

    conn.executemany(
        f"""INSERT OR REPLACE INTO {tabla}
            (dispositivo_id, timestamp, main_battery, internal_temp,
             wind_speed, wind_direction, wind_level, snow_level, irradiance,
             wind_alarm, gusty_wind_alarm, snow_alarm, snow_sensor_com_error)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
        rows,
    )
    conn.commit()

    _registrar_ingesta(conn, ncu_id, csv_path.name, tabla, nuevos, ya_existentes, rows)
    return len(rows)


def ingestar_hsu(
    conn: sqlite3.Connection,
    csv_path: str | Path,
    ncu_id: str,
    hsu_id: str,
) -> int:
    """Ingesta un fichero HSU. Devuelve filas insertadas."""
    return _ingestar_sensor(conn, csv_path, ncu_id, "HSU", hsu_id, "datos_hsu")


def ingestar_ncu_sensor(
    conn: sqlite3.Connection,
    csv_path: str | Path,
    ncu_id: str,
) -> int:
    """Ingesta un fichero NCU_SENSORS. Devuelve filas insertadas."""
    return _ingestar_sensor(conn, csv_path, ncu_id, "NCU", ncu_id, "datos_ncu_sensor")


def ingestar_tcu(
    conn: sqlite3.Connection,
    csv_path: str | Path,
    ncu_id: str,
    tcu_id: str,
) -> int:
    """Ingesta un fichero TCU. Devuelve filas insertadas."""
    csv_path = Path(csv_path)
    disp_id = obtener_o_crear_dispositivo(conn, ncu_id, "TCU", tcu_id)

    with open(csv_path, newline="") as f:
        reader = csv.DictReader(f, delimiter=";")
        rows = []
        for row in reader:
            rows.append((
                disp_id,
                _parse_ts(row["datetime"]),
                row["main_state"],
                _parse_bool(row["backtracking"]),
                _parse_bool(row["wind_from_east"]),
                _parse_int(row["active_security_position"]),
                _parse_float(row["angle"]),
                _parse_float(row["target_angle"]),
                _parse_int(row["soc"]),
                _parse_int(row["remaining_capacity"]),
                _parse_int(row["ps_voltage"]),
                _parse_int(row["ps_current"]),
                _parse_int(row["voltage"]),
                _parse_int(row["current"]),
                _parse_int(row["motor_voltage"]),
                _parse_int(row["motor_current"]),
                _parse_int(row["motor_current_peak"]),
                row["motor_state"],
                _parse_float(row["motor_pwm"]),
                _parse_int(row["daily_motor_power_consumption"]),
                _parse_float(row["pcb_temp"]),
                _parse_float(row["battery_temp"]),
                _parse_int(row["alarms_1"]),
                _parse_int(row["alarms_2"]),
                _parse_int(row["hw_alarms"]),
                _parse_int(row["system_monitor_status"]),
                _parse_int(row["system_monitor_flags"]),
                _parse_int(row["power_section_alarms"]),
            ))

    ya_existentes = _contar_existentes(conn, "datos_tcu", disp_id, [r[1] for r in rows])
    nuevos = len(rows) - ya_existentes

    conn.executemany(
        """INSERT OR REPLACE INTO datos_tcu
           (dispositivo_id, timestamp, main_state, backtracking, wind_from_east,
            active_security_position, angle, target_angle, soc, remaining_capacity,
            ps_voltage, ps_current, voltage, current, motor_voltage, motor_current,
            motor_current_peak, motor_state, motor_pwm, daily_motor_power_consumption,
            pcb_temp, battery_temp, alarms_1, alarms_2, hw_alarms,
            system_monitor_status, system_monitor_flags, power_section_alarms)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
        rows,
    )
    conn.commit()

    _registrar_ingesta(conn, ncu_id, csv_path.name, "datos_tcu", nuevos, ya_existentes, rows)
    return len(rows)


def ingestar_event_log(
    conn: sqlite3.Connection,
    csv_path: str | Path,
    ncu_id: str,
) -> int:
    """Ingesta un fichero NCU_EVENT_LOG (sin cabecera). Devuelve filas insertadas."""
    csv_path = Path(csv_path)
    disp_id = obtener_o_crear_dispositivo(conn, ncu_id, "NCU", ncu_id)

    rows = []
    with open(csv_path, newline="") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            # Formato: timestamp;evento (el evento puede contener ";")
            parts = line.split(";", maxsplit=1)
            if len(parts) == 2:
                rows.append((disp_id, _parse_ts(parts[0]), parts[1]))

    # el evento forma parte de la clave única, no basta con (dispositivo_id, timestamp)
    ya_existentes = sum(
        1 for disp_id_r, ts, evento in rows
        if conn.execute(
            "SELECT 1 FROM ncu_event_log WHERE dispositivo_id = ? AND timestamp = ? AND evento = ?",
            (disp_id_r, ts, evento),
        ).fetchone()
    )
    nuevos = len(rows) - ya_existentes

    conn.executemany(
        """INSERT OR REPLACE INTO ncu_event_log (dispositivo_id, timestamp, evento)
           VALUES (?, ?, ?)""",
        rows,
    )
    conn.commit()

    _registrar_ingesta(conn, ncu_id, csv_path.name, "ncu_event_log", nuevos, ya_existentes, rows)
    return len(rows)


def _registrar_ingesta(
    conn: sqlite3.Connection,
    ncu_id: str,
    fichero: str,
    tipo_datos: str,
    filas_nuevas: int,
    filas_actualizadas: int,
    rows: list,
) -> None:
    """Registra la ingesta en la tabla de log, con el desglose de filas nuevas y actualizadas."""
    # Extraer rango de timestamps
    ts_inicio = rows[0][1] if rows else None
    ts_fin = rows[-1][1] if rows else None

    conn.execute(
        """INSERT OR REPLACE INTO ingesta_log
           (ncu_id, fichero, tipo_datos, filas_insertadas, filas_nuevas, filas_actualizadas,
            timestamp_inicio, timestamp_fin)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
        (ncu_id, fichero, tipo_datos, filas_nuevas + filas_actualizadas, filas_nuevas, filas_actualizadas,
         ts_inicio, ts_fin),
    )
    conn.commit()
