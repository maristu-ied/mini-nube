use std::path::Path;

/// Tipo de archivo CSV detectado a partir del nombre de fichero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CsvFileType {
    NcuEventLog,
    NcuSensors,
    Ncu,
    Hsu { hsu_id: String },
    Tcu { tcu_id: String },
}

impl CsvFileType {
    /// Detecta el tipo de archivo según las convenciones de prefijos:
    /// - `NCU_EVENT_LOG_...` -> `NcuEventLog`
    /// - `NCU_SENSORS_...`   -> `NcuSensors`
    /// - `NCU_...`           -> `Ncu`
    /// - `HSU_...`           -> `Hsu` (con hsu_id = primeros 2 componentes separados por `_`)
    /// - `TCU_...`           -> `Tcu` (con tcu_id = primeros 2 componentes separados por `_`)
    pub fn detect_from_filename(filename: &str) -> Option<Self> {
        let name = Path::new(filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(filename);

        if name.starts_with("NCU_EVENT_LOG_") {
            Some(CsvFileType::NcuEventLog)
        } else if name.starts_with("NCU_SENSORS_") {
            Some(CsvFileType::NcuSensors)
        } else if name.starts_with("NCU_") {
            Some(CsvFileType::Ncu)
        } else if name.starts_with("HSU_") {
            let parts: Vec<&str> = name.split('_').collect();
            let hsu_id = if parts.len() >= 2 {
                format!("{}_{}", parts[0], parts[1])
            } else {
                parts[0].to_string()
            };
            Some(CsvFileType::Hsu { hsu_id })
        } else if name.starts_with("TCU_") {
            let parts: Vec<&str> = name.split('_').collect();
            let tcu_id = if parts.len() >= 2 {
                format!("{}_{}", parts[0], parts[1])
            } else {
                parts[0].to_string()
            };
            Some(CsvFileType::Tcu { tcu_id })
        } else {
            None
        }
    }

    /// Nombre de la tabla de datos correspondiente en la BD
    pub fn table_name(&self) -> &'static str {
        match self {
            CsvFileType::Ncu => "datos_ncu",
            CsvFileType::NcuSensors => "datos_ncu_sensor",
            CsvFileType::Hsu { .. } => "datos_hsu",
            CsvFileType::Tcu { .. } => "datos_tcu",
            CsvFileType::NcuEventLog => "ncu_event_log",
        }
    }
}

/// Registro para `datos_ncu`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NcuRecord {
    pub timestamp: i64,
    pub mqtt_online: bool,
    pub gw1_online: bool,
    pub gw2_online: bool,
    pub ups_power_ok: bool,
    pub ups_battery_low: bool,
    pub stop_button: bool,
    pub bluetooth_enabled: bool,
}

/// Registro para `datos_ncu_sensor` y `datos_hsu`
#[derive(Debug, Clone, PartialEq)]
pub struct SensorRecord {
    pub timestamp: i64,
    pub main_battery: Option<i64>,
    pub internal_temp: Option<f64>,
    pub wind_speed: Option<f64>,
    pub wind_direction: Option<f64>,
    pub wind_level: Option<i64>,
    pub snow_level: Option<f64>,
    pub irradiance: Option<i64>,
    pub wind_alarm: bool,
    pub gusty_wind_alarm: bool,
    pub snow_alarm: bool,
    pub snow_sensor_com_error: bool,
}

/// Registro para `datos_tcu`
#[derive(Debug, Clone, PartialEq)]
pub struct TcuRecord {
    pub timestamp: i64,
    pub main_state: Option<String>,
    pub backtracking: bool,
    pub wind_from_east: bool,
    pub active_security_position: Option<i64>,
    pub angle: Option<f64>,
    pub target_angle: Option<f64>,
    pub soc: Option<i64>,
    pub remaining_capacity: Option<i64>,
    pub ps_voltage: Option<i64>,
    pub ps_current: Option<i64>,
    pub voltage: Option<i64>,
    pub current: Option<i64>,
    pub motor_voltage: Option<i64>,
    pub motor_current: Option<i64>,
    pub motor_current_peak: Option<i64>,
    pub motor_state: Option<String>,
    pub motor_pwm: Option<f64>,
    pub daily_motor_power_consumption: Option<i64>,
    pub pcb_temp: Option<f64>,
    pub battery_temp: Option<f64>,
    pub alarms_1: Option<i64>,
    pub alarms_2: Option<i64>,
    pub hw_alarms: Option<i64>,
    pub system_monitor_status: Option<i64>,
    pub system_monitor_flags: Option<i64>,
    pub power_section_alarms: Option<i64>,
}

/// Registro para `ncu_event_log`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventLogRecord {
    pub timestamp: i64,
    pub event: String,
}
