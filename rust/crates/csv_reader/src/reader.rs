use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use chrono::NaiveDateTime;
use csv::ReaderBuilder;

use crate::error::CsvError;
use crate::models::{EventLogRecord, NcuRecord, SensorRecord, TcuRecord};

/// Convierte una cadena "YYYY-MM-DD HH:MM:SS" (UTC) a Unix epoch (segundos).
pub fn parse_timestamp(val: &str) -> Result<i64, CsvError> {
    let trimmed = val.trim();
    let dt = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| CsvError::DateTimeParse(trimmed.to_string(), e))?;
    Ok(dt.and_utc().timestamp())
}

/// Convierte "true"/"false" a bool (case-insensitive).
pub fn parse_bool(val: &str) -> bool {
    val.trim().eq_ignore_ascii_case("true")
}

/// Parsea un valor booleano tolerante a mayúsculas/minúsculas ("true", "True", "TRUE", "false", "False", "1", "0", etc.).
pub fn parse_bool_lenient(val: &str) -> Result<bool, String> {
    match val.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "y" | "t" => Ok(true),
        "false" | "0" | "no" | "n" | "f" => Ok(false),
        other => Err(format!(
            "Valor booleano inválido: '{other}'. Se esperaba true/false, True/False, 1/0."
        )),
    }
}

/// Convierte a i64, None si está vacío o no es un entero válido.
pub fn parse_int(val: &str) -> Result<Option<i64>, CsvError> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        trimmed
            .parse::<i64>()
            .map(Some)
            .map_err(|e| CsvError::NumberParse(trimmed.to_string(), e.to_string()))
    }
}

/// Convierte a f64, None si está vacío o no es un número válido.
pub fn parse_float(val: &str) -> Result<Option<f64>, CsvError> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        trimmed
            .parse::<f64>()
            .map(Some)
            .map_err(|e| CsvError::NumberParse(trimmed.to_string(), e.to_string()))
    }
}

/// Convierte una cadena de texto a Option<String>, None si está vacía.
pub fn parse_opt_string(val: &str) -> Option<String> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Lee un archivo CSV de tipo NCU (estado general).
pub fn read_ncu_csv(path: impl AsRef<Path>) -> Result<Vec<NcuRecord>, CsvError> {
    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_reader(BufReader::new(file));

    let headers = reader.headers()?.clone();
    let get_idx = |name: &str| -> usize {
        headers
            .iter()
            .position(|h| h.trim() == name)
            .unwrap_or_else(|| panic!("Columna obligatoria faltante: {}", name))
    };

    let idx_dt = get_idx("datetime");
    let idx_mqtt = get_idx("mqtt_online");
    let idx_gw1 = get_idx("gw1_online");
    let idx_gw2 = get_idx("gw2_online");
    let idx_ups_ok = get_idx("ups_power_ok");
    let idx_ups_low = get_idx("ups_battery_low");
    let idx_stop = get_idx("stop_button");
    let idx_bt = get_idx("bluetooth_enabled");

    let mut records = Vec::new();
    for result in reader.records() {
        let record = result?;
        let dt_str = record.get(idx_dt).unwrap_or_default();
        let ts = parse_timestamp(dt_str)?;

        records.push(NcuRecord {
            timestamp: ts,
            mqtt_online: parse_bool(record.get(idx_mqtt).unwrap_or_default()),
            gw1_online: parse_bool(record.get(idx_gw1).unwrap_or_default()),
            gw2_online: parse_bool(record.get(idx_gw2).unwrap_or_default()),
            ups_power_ok: parse_bool(record.get(idx_ups_ok).unwrap_or_default()),
            ups_battery_low: parse_bool(record.get(idx_ups_low).unwrap_or_default()),
            stop_button: parse_bool(record.get(idx_stop).unwrap_or_default()),
            bluetooth_enabled: parse_bool(record.get(idx_bt).unwrap_or_default()),
        });
    }

    Ok(records)
}

/// Lee un archivo CSV de tipo Sensor (HSU o NCU_SENSORS).
pub fn read_sensor_csv(path: impl AsRef<Path>) -> Result<Vec<SensorRecord>, CsvError> {
    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_reader(BufReader::new(file));

    let headers = reader.headers()?.clone();
    let get_idx = |name: &str| -> usize {
        headers
            .iter()
            .position(|h| h.trim() == name)
            .unwrap_or_else(|| panic!("Columna obligatoria faltante: {}", name))
    };

    let idx_dt = get_idx("datetime");
    let idx_battery = get_idx("main_battery");
    let idx_temp = get_idx("internal_temp");
    let idx_ws = get_idx("wind_speed");
    let idx_wd = get_idx("wind_direction");
    let idx_wl = get_idx("wind_level");
    let idx_sl = get_idx("snow_level");
    let idx_irr = get_idx("irradiance");
    let idx_wa = get_idx("wind_alarm");
    let idx_gwa = get_idx("gusty_wind_alarm");
    let idx_sa = get_idx("snow_alarm");
    let idx_sce = get_idx("snow_sensor_com_error");

    let mut records = Vec::new();
    for result in reader.records() {
        let record = result?;
        let dt_str = record.get(idx_dt).unwrap_or_default();
        let ts = parse_timestamp(dt_str)?;

        records.push(SensorRecord {
            timestamp: ts,
            main_battery: parse_int(record.get(idx_battery).unwrap_or_default())?,
            internal_temp: parse_float(record.get(idx_temp).unwrap_or_default())?,
            wind_speed: parse_float(record.get(idx_ws).unwrap_or_default())?,
            wind_direction: parse_float(record.get(idx_wd).unwrap_or_default())?,
            wind_level: parse_int(record.get(idx_wl).unwrap_or_default())?,
            snow_level: parse_float(record.get(idx_sl).unwrap_or_default())?,
            irradiance: parse_int(record.get(idx_irr).unwrap_or_default())?,
            wind_alarm: parse_bool(record.get(idx_wa).unwrap_or_default()),
            gusty_wind_alarm: parse_bool(record.get(idx_gwa).unwrap_or_default()),
            snow_alarm: parse_bool(record.get(idx_sa).unwrap_or_default()),
            snow_sensor_com_error: parse_bool(record.get(idx_sce).unwrap_or_default()),
        });
    }

    Ok(records)
}

/// Lee un archivo CSV de tipo TCU.
pub fn read_tcu_csv(path: impl AsRef<Path>) -> Result<Vec<TcuRecord>, CsvError> {
    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_reader(BufReader::new(file));

    let headers = reader.headers()?.clone();
    let get_idx = |name: &str| -> usize {
        headers
            .iter()
            .position(|h| h.trim() == name)
            .unwrap_or_else(|| panic!("Columna obligatoria faltante: {}", name))
    };

    let idx_dt = get_idx("datetime");
    let idx_main_state = get_idx("main_state");
    let idx_backtracking = get_idx("backtracking");
    let idx_wind_east = get_idx("wind_from_east");
    let idx_sec_pos = get_idx("active_security_position");
    let idx_angle = get_idx("angle");
    let idx_target_angle = get_idx("target_angle");
    let idx_soc = get_idx("soc");
    let idx_rem_cap = get_idx("remaining_capacity");
    let idx_ps_volt = get_idx("ps_voltage");
    let idx_ps_curr = get_idx("ps_current");
    let idx_volt = get_idx("voltage");
    let idx_curr = get_idx("current");
    let idx_mot_volt = get_idx("motor_voltage");
    let idx_mot_curr = get_idx("motor_current");
    let idx_mot_curr_peak = get_idx("motor_current_peak");
    let idx_mot_state = get_idx("motor_state");
    let idx_mot_pwm = get_idx("motor_pwm");
    let idx_daily_cons = get_idx("daily_motor_power_consumption");
    let idx_pcb_temp = get_idx("pcb_temp");
    let idx_bat_temp = get_idx("battery_temp");
    let idx_alarms1 = get_idx("alarms_1");
    let idx_alarms2 = get_idx("alarms_2");
    let idx_hw_alarms = get_idx("hw_alarms");
    let idx_sys_mon_stat = get_idx("system_monitor_status");
    let idx_sys_mon_flags = get_idx("system_monitor_flags");
    let idx_ps_alarms = get_idx("power_section_alarms");

    let mut records = Vec::new();
    for result in reader.records() {
        let record = result?;
        let dt_str = record.get(idx_dt).unwrap_or_default();
        let ts = parse_timestamp(dt_str)?;

        records.push(TcuRecord {
            timestamp: ts,
            main_state: parse_opt_string(record.get(idx_main_state).unwrap_or_default()),
            backtracking: parse_bool(record.get(idx_backtracking).unwrap_or_default()),
            wind_from_east: parse_bool(record.get(idx_wind_east).unwrap_or_default()),
            active_security_position: parse_int(record.get(idx_sec_pos).unwrap_or_default())?,
            angle: parse_float(record.get(idx_angle).unwrap_or_default())?,
            target_angle: parse_float(record.get(idx_target_angle).unwrap_or_default())?,
            soc: parse_int(record.get(idx_soc).unwrap_or_default())?,
            remaining_capacity: parse_int(record.get(idx_rem_cap).unwrap_or_default())?,
            ps_voltage: parse_int(record.get(idx_ps_volt).unwrap_or_default())?,
            ps_current: parse_int(record.get(idx_ps_curr).unwrap_or_default())?,
            voltage: parse_int(record.get(idx_volt).unwrap_or_default())?,
            current: parse_int(record.get(idx_curr).unwrap_or_default())?,
            motor_voltage: parse_int(record.get(idx_mot_volt).unwrap_or_default())?,
            motor_current: parse_int(record.get(idx_mot_curr).unwrap_or_default())?,
            motor_current_peak: parse_int(record.get(idx_mot_curr_peak).unwrap_or_default())?,
            motor_state: parse_opt_string(record.get(idx_mot_state).unwrap_or_default()),
            motor_pwm: parse_float(record.get(idx_mot_pwm).unwrap_or_default())?,
            daily_motor_power_consumption: parse_int(record.get(idx_daily_cons).unwrap_or_default())?,
            pcb_temp: parse_float(record.get(idx_pcb_temp).unwrap_or_default())?,
            battery_temp: parse_float(record.get(idx_bat_temp).unwrap_or_default())?,
            alarms_1: parse_int(record.get(idx_alarms1).unwrap_or_default())?,
            alarms_2: parse_int(record.get(idx_alarms2).unwrap_or_default())?,
            hw_alarms: parse_int(record.get(idx_hw_alarms).unwrap_or_default())?,
            system_monitor_status: parse_int(record.get(idx_sys_mon_stat).unwrap_or_default())?,
            system_monitor_flags: parse_int(record.get(idx_sys_mon_flags).unwrap_or_default())?,
            power_section_alarms: parse_int(record.get(idx_ps_alarms).unwrap_or_default())?,
        });
    }

    Ok(records)
}

/// Lee un archivo CSV de tipo NCU_EVENT_LOG (sin cabecera).
/// Formato de cada línea: `timestamp;evento` (el evento puede contener `;`).
pub fn read_event_log_csv(path: impl AsRef<Path>) -> Result<Vec<EventLogRecord>, CsvError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut records = Vec::new();
    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((ts_str, event_str)) = trimmed.split_once(';') {
            let ts = parse_timestamp(ts_str)?;
            records.push(EventLogRecord {
                timestamp: ts,
                event: event_str.to_string(),
            });
        }
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CsvFileType;

    #[test]
    fn test_parse_timestamp() {
        assert_eq!(parse_timestamp("2026-07-29 00:00:00").unwrap(), 1785283200);
    }

    #[test]
    fn test_parse_bool() {
        assert!(parse_bool("true"));
        assert!(parse_bool("TRUE"));
        assert!(parse_bool("True"));
        assert!(!parse_bool("false"));
        assert!(!parse_bool("0"));
    }

    #[test]
    fn test_parse_bool_lenient() {
        assert_eq!(parse_bool_lenient("true").unwrap(), true);
        assert_eq!(parse_bool_lenient("True").unwrap(), true);
        assert_eq!(parse_bool_lenient("TRUE").unwrap(), true);
        assert_eq!(parse_bool_lenient("1").unwrap(), true);
        assert_eq!(parse_bool_lenient("false").unwrap(), false);
        assert_eq!(parse_bool_lenient("False").unwrap(), false);
        assert_eq!(parse_bool_lenient("FALSE").unwrap(), false);
        assert_eq!(parse_bool_lenient("0").unwrap(), false);
        assert!(parse_bool_lenient("invalid").is_err());
    }

    #[test]
    fn test_detect_file_type() {
        assert_eq!(
            CsvFileType::detect_from_filename("NCU_2026-08-26.csv"),
            Some(CsvFileType::Ncu)
        );
        assert_eq!(
            CsvFileType::detect_from_filename("NCU_EVENT_LOG_2026-08-26.csv"),
            Some(CsvFileType::NcuEventLog)
        );
        assert_eq!(
            CsvFileType::detect_from_filename("NCU_SENSORS_2026-08-26.csv"),
            Some(CsvFileType::NcuSensors)
        );
        assert_eq!(
            CsvFileType::detect_from_filename("HSU_230_2026-08-26.csv"),
            Some(CsvFileType::Hsu {
                hsu_id: "HSU_230".to_string()
            })
        );
        assert_eq!(
            CsvFileType::detect_from_filename("TCU_001_2026-08-26.csv"),
            Some(CsvFileType::Tcu {
                tcu_id: "TCU_001".to_string()
            })
        );
    }
}
