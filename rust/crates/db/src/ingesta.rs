use std::path::Path;
use rusqlite::{params, Connection};
use walkdir::WalkDir;

use mini_nube_csv::{
    read_event_log_csv, read_ncu_csv, read_sensor_csv, read_tcu_csv, CsvFileType,
};

use crate::error::DbError;
use crate::repository::{
    contar_eventos_existentes, contar_existentes, obtener_o_crear_dispositivo, registrar_ingesta,
};

/// Ingesta un único fichero CSV según su prefijo.
pub fn ingestar_fichero(
    conn: &mut Connection,
    csv_path: impl AsRef<Path>,
    ncu_id: &str,
) -> Result<usize, DbError> {
    let path = csv_path.as_ref();
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| DbError::PathError(format!("Ruta inválida: {:?}", path)))?;

    let file_type = match CsvFileType::detect_from_filename(filename) {
        Some(ft) => ft,
        None => return Ok(0), // Si no coincide con ningún patrón conocido, no hace nada
    };

    match file_type {
        CsvFileType::NcuEventLog => ingestar_event_log(conn, path, ncu_id),
        CsvFileType::NcuSensors => ingestar_ncu_sensor(conn, path, ncu_id),
        CsvFileType::Ncu => ingestar_ncu(conn, path, ncu_id),
        CsvFileType::Hsu { hsu_id } => ingestar_hsu(conn, path, ncu_id, &hsu_id),
        CsvFileType::Tcu { tcu_id } => ingestar_tcu(conn, path, ncu_id, &tcu_id),
    }
}

/// Escanea plant_folder y sus subcarpetas recursivamente, ingestando cada CSV.
pub fn ingestar_directorio(
    conn: &mut Connection,
    plant_folder: impl AsRef<Path>,
    ncu_id: &str,
    skip_files_already_inserted: bool,
) -> Result<usize, DbError> {
    let plant_path = plant_folder.as_ref();
    println!("Escaneando carpeta: {}", plant_path.display());

    let mut csv_files: Vec<_> = WalkDir::new(plant_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("csv"))
        })
        .map(|e| e.into_path())
        .collect();

    csv_files.sort();
    println!("Archivos CSV encontrados: {}", csv_files.len());

    let mut total_insertadas = 0;

    for csv_path in csv_files {
        let nombre = csv_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        if skip_files_already_inserted {
            let mut stmt = conn.prepare_cached(
                "SELECT COUNT(*) FROM ingesta_log WHERE fichero = ? AND ncu_id = ?",
            )?;
            let ya_ingestado: i64 = stmt.query_row(params![nombre, ncu_id], |r| r.get(0))?;
            if ya_ingestado > 0 {
                continue;
            }
        }

        let filas = ingestar_fichero(conn, &csv_path, ncu_id)?;
        total_insertadas += filas;
    }

    Ok(total_insertadas)
}

/// Ingesta un fichero NCU (estado general).
pub fn ingestar_ncu(
    conn: &mut Connection,
    csv_path: impl AsRef<Path>,
    ncu_id: &str,
) -> Result<usize, DbError> {
    let path = csv_path.as_ref();
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();

    let disp_id = obtener_o_crear_dispositivo(conn, ncu_id, "NCU", ncu_id)?;
    let records = read_ncu_csv(path)?;
    if records.is_empty() {
        return Ok(0);
    }

    let timestamps: Vec<i64> = records.iter().map(|r| r.timestamp).collect();
    let ya_existentes = contar_existentes(conn, "datos_ncu", disp_id, &timestamps)?;
    let nuevos = records.len().saturating_sub(ya_existentes);

    let ts_inicio = records.first().map(|r| r.timestamp);
    let ts_fin = records.last().map(|r| r.timestamp);

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare_cached(
            r#"
            INSERT OR REPLACE INTO datos_ncu
               (dispositivo_id, timestamp, mqtt_online, gw1_online, gw2_online,
                ups_power_ok, ups_battery_low, stop_button, bluetooth_enabled)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )?;

        for r in &records {
            stmt.execute(params![
                disp_id,
                r.timestamp,
                if r.mqtt_online { 1 } else { 0 },
                if r.gw1_online { 1 } else { 0 },
                if r.gw2_online { 1 } else { 0 },
                if r.ups_power_ok { 1 } else { 0 },
                if r.ups_battery_low { 1 } else { 0 },
                if r.stop_button { 1 } else { 0 },
                if r.bluetooth_enabled { 1 } else { 0 },
            ])?;
        }
    }
    tx.commit()?;

    registrar_ingesta(
        conn,
        ncu_id,
        &filename,
        "datos_ncu",
        nuevos,
        ya_existentes,
        ts_inicio,
        ts_fin,
    )?;

    Ok(records.len())
}

/// Ingesta genérica para datos de sensores (HSU y NCU_SENSORS comparten esquema).
fn ingestar_sensor_interno(
    conn: &mut Connection,
    csv_path: impl AsRef<Path>,
    ncu_id: &str,
    tipo: &str,
    device_id: &str,
    tabla: &str,
) -> Result<usize, DbError> {
    let path = csv_path.as_ref();
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();

    let disp_id = obtener_o_crear_dispositivo(conn, ncu_id, tipo, device_id)?;
    let records = read_sensor_csv(path)?;
    if records.is_empty() {
        return Ok(0);
    }

    let timestamps: Vec<i64> = records.iter().map(|r| r.timestamp).collect();
    let ya_existentes = contar_existentes(conn, tabla, disp_id, &timestamps)?;
    let nuevos = records.len().saturating_sub(ya_existentes);

    let ts_inicio = records.first().map(|r| r.timestamp);
    let ts_fin = records.last().map(|r| r.timestamp);

    let sql = format!(
        r#"
        INSERT OR REPLACE INTO {}
            (dispositivo_id, timestamp, main_battery, internal_temp,
             wind_speed, wind_direction, wind_level, snow_level, irradiance,
             wind_alarm, gusty_wind_alarm, snow_alarm, snow_sensor_com_error)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        tabla
    );

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare_cached(&sql)?;
        for r in &records {
            stmt.execute(params![
                disp_id,
                r.timestamp,
                r.main_battery,
                r.internal_temp,
                r.wind_speed,
                r.wind_direction,
                r.wind_level,
                r.snow_level,
                r.irradiance,
                if r.wind_alarm { 1 } else { 0 },
                if r.gusty_wind_alarm { 1 } else { 0 },
                if r.snow_alarm { 1 } else { 0 },
                if r.snow_sensor_com_error { 1 } else { 0 },
            ])?;
        }
    }
    tx.commit()?;

    registrar_ingesta(
        conn,
        ncu_id,
        &filename,
        tabla,
        nuevos,
        ya_existentes,
        ts_inicio,
        ts_fin,
    )?;

    Ok(records.len())
}

/// Ingesta un fichero HSU.
pub fn ingestar_hsu(
    conn: &mut Connection,
    csv_path: impl AsRef<Path>,
    ncu_id: &str,
    hsu_id: &str,
) -> Result<usize, DbError> {
    ingestar_sensor_interno(conn, csv_path, ncu_id, "HSU", hsu_id, "datos_hsu")
}

/// Ingesta un fichero NCU_SENSORS.
pub fn ingestar_ncu_sensor(
    conn: &mut Connection,
    csv_path: impl AsRef<Path>,
    ncu_id: &str,
) -> Result<usize, DbError> {
    ingestar_sensor_interno(conn, csv_path, ncu_id, "NCU", ncu_id, "datos_ncu_sensor")
}

/// Ingesta un fichero TCU.
pub fn ingestar_tcu(
    conn: &mut Connection,
    csv_path: impl AsRef<Path>,
    ncu_id: &str,
    tcu_id: &str,
) -> Result<usize, DbError> {
    let path = csv_path.as_ref();
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();

    let disp_id = obtener_o_crear_dispositivo(conn, ncu_id, "TCU", tcu_id)?;
    let records = read_tcu_csv(path)?;
    if records.is_empty() {
        return Ok(0);
    }

    let timestamps: Vec<i64> = records.iter().map(|r| r.timestamp).collect();
    let ya_existentes = contar_existentes(conn, "datos_tcu", disp_id, &timestamps)?;
    let nuevos = records.len().saturating_sub(ya_existentes);

    let ts_inicio = records.first().map(|r| r.timestamp);
    let ts_fin = records.last().map(|r| r.timestamp);

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare_cached(
            r#"
            INSERT OR REPLACE INTO datos_tcu
               (dispositivo_id, timestamp, main_state, backtracking, wind_from_east,
                active_security_position, angle, target_angle, soc, remaining_capacity,
                ps_voltage, ps_current, voltage, current, motor_voltage, motor_current,
                motor_current_peak, motor_state, motor_pwm, daily_motor_power_consumption,
                pcb_temp, battery_temp, alarms_1, alarms_2, hw_alarms,
                system_monitor_status, system_monitor_flags, power_section_alarms)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )?;

        for r in &records {
            stmt.execute(params![
                disp_id,
                r.timestamp,
                r.main_state,
                if r.backtracking { 1 } else { 0 },
                if r.wind_from_east { 1 } else { 0 },
                r.active_security_position,
                r.angle,
                r.target_angle,
                r.soc,
                r.remaining_capacity,
                r.ps_voltage,
                r.ps_current,
                r.voltage,
                r.current,
                r.motor_voltage,
                r.motor_current,
                r.motor_current_peak,
                r.motor_state,
                r.motor_pwm,
                r.daily_motor_power_consumption,
                r.pcb_temp,
                r.battery_temp,
                r.alarms_1,
                r.alarms_2,
                r.hw_alarms,
                r.system_monitor_status,
                r.system_monitor_flags,
                r.power_section_alarms,
            ])?;
        }
    }
    tx.commit()?;

    registrar_ingesta(
        conn,
        ncu_id,
        &filename,
        "datos_tcu",
        nuevos,
        ya_existentes,
        ts_inicio,
        ts_fin,
    )?;

    Ok(records.len())
}

/// Ingesta un fichero NCU_EVENT_LOG (sin cabecera).
pub fn ingestar_event_log(
    conn: &mut Connection,
    csv_path: impl AsRef<Path>,
    ncu_id: &str,
) -> Result<usize, DbError> {
    let path = csv_path.as_ref();
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();

    let disp_id = obtener_o_crear_dispositivo(conn, ncu_id, "NCU", ncu_id)?;
    let records = read_event_log_csv(path)?;
    if records.is_empty() {
        return Ok(0);
    }

    let eventos_refs: Vec<(i64, &str)> = records
        .iter()
        .map(|r| (r.timestamp, r.event.as_str()))
        .collect();
    let ya_existentes = contar_eventos_existentes(conn, disp_id, &eventos_refs)?;
    let nuevos = records.len().saturating_sub(ya_existentes);

    let ts_inicio = records.first().map(|r| r.timestamp);
    let ts_fin = records.last().map(|r| r.timestamp);

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare_cached(
            r#"
            INSERT OR REPLACE INTO ncu_event_log (dispositivo_id, timestamp, evento)
            VALUES (?, ?, ?)
            "#,
        )?;

        for r in &records {
            stmt.execute(params![disp_id, r.timestamp, r.event])?;
        }
    }
    tx.commit()?;

    registrar_ingesta(
        conn,
        ncu_id,
        &filename,
        "ncu_event_log",
        nuevos,
        ya_existentes,
        ts_inicio,
        ts_fin,
    )?;

    Ok(records.len())
}
