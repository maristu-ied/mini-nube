"""
Esquema de base de datos SQLite para Mini Nube.

Jerarquía de dispositivos:
    NCU (Network Control Unit)
    ├── HSU (sensores meteorológicos, mismas columnas que ncu_sensors)
    └── TCU (Tracker Control Unit)

Tablas de datos:
    - dispositivos: registro de NCUs, HSUs y TCUs
    - datos_ncu: estado general de la NCU (conectividad, UPS, etc.)
    - datos_ncu_sensor: lecturas de sensores de la NCU
    - datos_hsu: lecturas de sensores de la HSU (mismo esquema que ncu_sensor)
    - datos_tcu: telemetría del tracker (ángulo, motor, batería, alarmas)
    - ncu_event_log: eventos con timestamp y texto libre

Todos los timestamps se almacenan en UTC como INTEGER (Unix epoch, segundos desde 1970-01-01).
"""

# -- Tabla de dispositivos (registro) -----------------------------------------

SQL_CREATE_DISPOSITIVOS = """
CREATE TABLE IF NOT EXISTS dispositivos (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    ncu_id          TEXT    NOT NULL,
    tipo            TEXT    NOT NULL CHECK (tipo IN ('NCU', 'HSU', 'TCU')),
    device_id       TEXT    NOT NULL,
    descripcion     TEXT,
    created_at      INTEGER NOT NULL DEFAULT (unixepoch('now')),

    UNIQUE (ncu_id, tipo, device_id)
);
"""

# -- Datos NCU (estado general) ------------------------------------------------

SQL_CREATE_DATOS_NCU = """
CREATE TABLE IF NOT EXISTS datos_ncu (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    dispositivo_id      INTEGER NOT NULL REFERENCES dispositivos(id),
    timestamp           INTEGER NOT NULL,
    mqtt_online         INTEGER NOT NULL,
    gw1_online          INTEGER NOT NULL,
    gw2_online          INTEGER NOT NULL,
    ups_power_ok        INTEGER NOT NULL,
    ups_battery_low     INTEGER NOT NULL,
    stop_button         INTEGER NOT NULL,
    bluetooth_enabled   INTEGER NOT NULL
);
"""

SQL_CREATE_IDX_DATOS_NCU = """
CREATE INDEX IF NOT EXISTS idx_datos_ncu_ts
    ON datos_ncu (dispositivo_id, timestamp);
"""

# -- Datos NCU Sensor / HSU (mismas columnas) ----------------------------------

_SQL_SENSOR_COLUMNS = """
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    dispositivo_id          INTEGER NOT NULL REFERENCES dispositivos(id),
    timestamp               INTEGER NOT NULL,
    main_battery            INTEGER,
    internal_temp           REAL,
    wind_speed              REAL,
    wind_direction          REAL,
    wind_level              INTEGER,
    snow_level              REAL,
    irradiance              INTEGER,
    wind_alarm              INTEGER NOT NULL,
    gusty_wind_alarm        INTEGER NOT NULL,
    snow_alarm              INTEGER NOT NULL,
    snow_sensor_com_error   INTEGER NOT NULL
"""

SQL_CREATE_DATOS_NCU_SENSOR = f"""
CREATE TABLE IF NOT EXISTS datos_ncu_sensor (
{_SQL_SENSOR_COLUMNS}
);
"""

SQL_CREATE_IDX_DATOS_NCU_SENSOR = """
CREATE INDEX IF NOT EXISTS idx_datos_ncu_sensor_ts
    ON datos_ncu_sensor (dispositivo_id, timestamp);
"""

SQL_CREATE_DATOS_HSU = f"""
CREATE TABLE IF NOT EXISTS datos_hsu (
{_SQL_SENSOR_COLUMNS}
);
"""

SQL_CREATE_IDX_DATOS_HSU = """
CREATE INDEX IF NOT EXISTS idx_datos_hsu_ts
    ON datos_hsu (dispositivo_id, timestamp);
"""

# -- Datos TCU -----------------------------------------------------------------

SQL_CREATE_DATOS_TCU = """
CREATE TABLE IF NOT EXISTS datos_tcu (
    id                              INTEGER PRIMARY KEY AUTOINCREMENT,
    dispositivo_id                  INTEGER NOT NULL REFERENCES dispositivos(id),
    timestamp                       INTEGER NOT NULL,
    main_state                      TEXT,
    backtracking                    INTEGER NOT NULL,
    wind_from_east                  INTEGER NOT NULL,
    active_security_position        INTEGER,
    angle                           REAL,
    target_angle                    REAL,
    soc                             INTEGER,
    remaining_capacity              INTEGER,
    ps_voltage                      INTEGER,
    ps_current                      INTEGER,
    voltage                         INTEGER,
    current                         INTEGER,
    motor_voltage                   INTEGER,
    motor_current                   INTEGER,
    motor_current_peak              INTEGER,
    motor_state                     TEXT,
    motor_pwm                       REAL,
    daily_motor_power_consumption   INTEGER,
    pcb_temp                        REAL,
    battery_temp                    REAL,
    alarms_1                        INTEGER,
    alarms_2                        INTEGER,
    hw_alarms                       INTEGER,
    system_monitor_status           INTEGER,
    system_monitor_flags            INTEGER,
    power_section_alarms            INTEGER
);
"""

SQL_CREATE_IDX_DATOS_TCU = """
CREATE INDEX IF NOT EXISTS idx_datos_tcu_ts
    ON datos_tcu (dispositivo_id, timestamp);
"""

# -- NCU Event Log -------------------------------------------------------------

SQL_CREATE_NCU_EVENT_LOG = """
CREATE TABLE IF NOT EXISTS ncu_event_log (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    dispositivo_id      INTEGER NOT NULL REFERENCES dispositivos(id),
    timestamp           INTEGER NOT NULL,
    evento              TEXT    NOT NULL
);
"""

SQL_CREATE_IDX_NCU_EVENT_LOG = """
CREATE INDEX IF NOT EXISTS idx_ncu_event_log_ts
    ON ncu_event_log (dispositivo_id, timestamp);
"""

# -- Metadata de ingesta ------------------------------------------------------

SQL_CREATE_INGESTA_LOG = """
CREATE TABLE IF NOT EXISTS ingesta_log (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    ncu_id          TEXT    NOT NULL,
    fichero         TEXT    NOT NULL,
    tipo_datos      TEXT    NOT NULL,
    filas_insertadas INTEGER NOT NULL,
    timestamp_inicio INTEGER,
    timestamp_fin    INTEGER,
    ingested_at     INTEGER NOT NULL DEFAULT (unixepoch('now'))
);
"""

# -- Orden de creación --------------------------------------------------------

ALL_CREATE_STATEMENTS = [
    SQL_CREATE_DISPOSITIVOS,
    SQL_CREATE_DATOS_NCU,
    SQL_CREATE_IDX_DATOS_NCU,
    SQL_CREATE_DATOS_NCU_SENSOR,
    SQL_CREATE_IDX_DATOS_NCU_SENSOR,
    SQL_CREATE_DATOS_HSU,
    SQL_CREATE_IDX_DATOS_HSU,
    SQL_CREATE_DATOS_TCU,
    SQL_CREATE_IDX_DATOS_TCU,
    SQL_CREATE_NCU_EVENT_LOG,
    SQL_CREATE_IDX_NCU_EVENT_LOG,
    SQL_CREATE_INGESTA_LOG,
]
