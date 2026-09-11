use rusqlite::Connection;

use crate::error::DbError;

pub const SQL_CREATE_DISPOSITIVOS: &str = r#"
CREATE TABLE IF NOT EXISTS dispositivos (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    ncu_id          TEXT    NOT NULL,
    tipo            TEXT    NOT NULL CHECK (tipo IN ('NCU', 'HSU', 'TCU')),
    device_id       TEXT    NOT NULL,
    descripcion     TEXT,
    created_at      INTEGER NOT NULL DEFAULT (unixepoch('now')),

    UNIQUE (ncu_id, tipo, device_id)
);
"#;

pub const SQL_CREATE_DATOS_NCU: &str = r#"
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
"#;

pub const SQL_CREATE_IDX_DATOS_NCU: &str = r#"
CREATE UNIQUE INDEX IF NOT EXISTS idx_datos_ncu_ts
    ON datos_ncu (dispositivo_id, timestamp);
"#;

pub const SQL_CREATE_DATOS_NCU_SENSOR: &str = r#"
CREATE TABLE IF NOT EXISTS datos_ncu_sensor (
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
);
"#;

pub const SQL_CREATE_IDX_DATOS_NCU_SENSOR: &str = r#"
CREATE UNIQUE INDEX IF NOT EXISTS idx_datos_ncu_sensor_ts
    ON datos_ncu_sensor (dispositivo_id, timestamp);
"#;

pub const SQL_CREATE_DATOS_HSU: &str = r#"
CREATE TABLE IF NOT EXISTS datos_hsu (
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
);
"#;

pub const SQL_CREATE_IDX_DATOS_HSU: &str = r#"
CREATE UNIQUE INDEX IF NOT EXISTS idx_datos_hsu_ts
    ON datos_hsu (dispositivo_id, timestamp);
"#;

pub const SQL_CREATE_DATOS_TCU: &str = r#"
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
"#;

pub const SQL_CREATE_IDX_DATOS_TCU: &str = r#"
CREATE UNIQUE INDEX IF NOT EXISTS idx_datos_tcu_ts
    ON datos_tcu (dispositivo_id, timestamp);
"#;

pub const SQL_CREATE_NCU_EVENT_LOG: &str = r#"
CREATE TABLE IF NOT EXISTS ncu_event_log (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    dispositivo_id      INTEGER NOT NULL REFERENCES dispositivos(id),
    timestamp           INTEGER NOT NULL,
    evento              TEXT    NOT NULL
);
"#;

pub const SQL_CREATE_IDX_NCU_EVENT_LOG: &str = r#"
CREATE UNIQUE INDEX IF NOT EXISTS idx_ncu_event_log_ts
    ON ncu_event_log (dispositivo_id, timestamp, evento);
"#;

pub const SQL_CREATE_INGESTA_LOG: &str = r#"
CREATE TABLE IF NOT EXISTS ingesta_log (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    ncu_id          TEXT    NOT NULL,
    fichero         TEXT    NOT NULL,
    tipo_datos      TEXT    NOT NULL,
    filas_insertadas INTEGER NOT NULL,
    filas_nuevas      INTEGER NOT NULL,
    filas_actualizadas INTEGER NOT NULL,
    timestamp_inicio INTEGER,
    timestamp_fin    INTEGER,
    ingested_at     INTEGER NOT NULL DEFAULT (unixepoch('now'))
);
"#;

pub const ALL_CREATE_STATEMENTS: &[&str] = &[
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
];

/// Crea todas las tablas e índices si no existen.
pub fn inicializar(conn: &Connection) -> Result<(), DbError> {
    for sql in ALL_CREATE_STATEMENTS {
        conn.execute(sql, [])?;
    }
    Ok(())
}
