use std::path::PathBuf;
use mini_nube_db::{
    conectar, inicializar, ingestar_event_log, ingestar_hsu, ingestar_ncu, ingestar_ncu_sensor,
    ingestar_tcu,
};
use rusqlite::Connection;
use tempfile::NamedTempFile;

fn get_data_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("../../../python/tests/data")
}

fn setup_db() -> Connection {
    let tmp = NamedTempFile::new().unwrap();
    let conn = conectar(tmp.path()).unwrap();
    inicializar(&conn).unwrap();
    conn
}

const NCU_ID: &str = "NCU_TEST_001";

#[test]
fn test_ingesta_ncu() {
    let mut conn = setup_db();
    let data_dir = get_data_dir();
    let csv_file = data_dir.join("NCU_20260729.csv");

    let filas = ingestar_ncu(&mut conn, &csv_file, NCU_ID).unwrap();
    assert!(filas > 0);

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM datos_ncu", [], |r| r.get(0))
        .unwrap();
    assert!(count > 0);
    assert!(count as usize <= filas);

    let (ts, mqtt, gw1): (i64, i32, i32) = conn
        .query_row(
            "SELECT timestamp, mqtt_online, gw1_online FROM datos_ncu ORDER BY timestamp LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();

    assert_eq!(ts, 1785283200); // 2026-07-29 00:00:00 UTC
    assert_eq!(mqtt, 0);
    assert_eq!(gw1, 1);
}

#[test]
fn test_ingesta_hsu() {
    let mut conn = setup_db();
    let data_dir = get_data_dir();
    let csv_file = data_dir.join("HSU_230_20260729.csv");

    let filas = ingestar_hsu(&mut conn, &csv_file, NCU_ID, "HSU_230").unwrap();
    assert!(filas > 0);

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM datos_hsu", [], |r| r.get(0))
        .unwrap();
    assert!(count > 0);
    assert!(count as usize <= filas);

    let (ts, battery, temp, wind_alarm): (i64, i64, f64, i32) = conn
        .query_row(
            "SELECT timestamp, main_battery, internal_temp, wind_alarm FROM datos_hsu ORDER BY timestamp LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();

    assert_eq!(ts, 1785283200);
    assert_eq!(battery, 14847);
    assert!((temp - 19.85).abs() < 1e-4);
    assert_eq!(wind_alarm, 0);
}

#[test]
fn test_ingesta_ncu_sensor() {
    let mut conn = setup_db();
    let data_dir = get_data_dir();
    let csv_file = data_dir.join("NCU_SENSORS_20260901.csv");

    let filas = ingestar_ncu_sensor(&mut conn, &csv_file, NCU_ID).unwrap();
    assert!(filas > 0);

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM datos_ncu_sensor", [], |r| r.get(0))
        .unwrap();
    assert!(count > 0);
    assert!(count as usize <= filas);

    let (battery, temp): (i64, f64) = conn
        .query_row(
            "SELECT main_battery, internal_temp FROM datos_ncu_sensor ORDER BY timestamp LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();

    assert_eq!(battery, 18299);
    assert!((temp - 33.85).abs() < 1e-4);
}

#[test]
fn test_ingesta_tcu() {
    let mut conn = setup_db();
    let data_dir = get_data_dir();
    let csv_file = data_dir.join("TCU_001_20260729.csv");

    let filas = ingestar_tcu(&mut conn, &csv_file, NCU_ID, "TCU_001").unwrap();
    assert!(filas > 0);

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM datos_tcu", [], |r| r.get(0))
        .unwrap();
    assert!(count > 0);
    assert!(count as usize <= filas);

    let (ts, main_state, backtracking, angle, soc, motor_state): (
        i64,
        String,
        i32,
        f64,
        i64,
        String,
    ) = conn
        .query_row(
            "SELECT timestamp, main_state, backtracking, angle, soc, motor_state FROM datos_tcu ORDER BY timestamp LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
        )
        .unwrap();

    assert_eq!(ts, 1785283208); // 2026-07-29 00:00:08 UTC
    assert_eq!(main_state, "AUTO");
    assert_eq!(backtracking, 0);
    assert!((angle - 5.20).abs() < 1e-4);
    assert_eq!(soc, 84);
    assert_eq!(motor_state, "OFF");
}

#[test]
fn test_ingesta_event_log() {
    let mut conn = setup_db();
    let data_dir = get_data_dir();
    let csv_file = data_dir.join("NCU_EVENT_LOG_20260729.csv");

    let filas = ingestar_event_log(&mut conn, &csv_file, NCU_ID).unwrap();
    assert!(filas > 0);

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM ncu_event_log", [], |r| r.get(0))
        .unwrap();
    assert!(count > 0);
    assert!(count as usize <= filas);

    let (ts, evento): (i64, String) = conn
        .query_row(
            "SELECT timestamp, evento FROM ncu_event_log ORDER BY timestamp LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();

    assert_eq!(ts, 1785307707); // 2026-07-29 06:48:27 UTC
    assert!(evento.contains("OTA performed"));
}

#[test]
fn test_registro_ingesta() {
    let mut conn = setup_db();
    let data_dir = get_data_dir();
    let csv_file = data_dir.join("NCU_20260729.csv");

    ingestar_ncu(&mut conn, &csv_file, NCU_ID).unwrap();

    let (ncu_id, tipo_datos, filas_ins, ts_inicio, ts_fin): (
        String,
        String,
        i64,
        Option<i64>,
        Option<i64>,
    ) = conn
        .query_row(
            "SELECT ncu_id, tipo_datos, filas_insertadas, timestamp_inicio, timestamp_fin FROM ingesta_log LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .unwrap();

    assert_eq!(ncu_id, NCU_ID);
    assert_eq!(tipo_datos, "datos_ncu");
    assert!(filas_ins > 0);
    assert!(ts_inicio.is_some());
    assert!(ts_fin.is_some());
}

#[test]
fn test_dispositivos_tras_ingesta_completa() {
    let mut conn = setup_db();
    let data_dir = get_data_dir();

    ingestar_ncu(&mut conn, data_dir.join("NCU_20260729.csv"), NCU_ID).unwrap();
    ingestar_ncu_sensor(&mut conn, data_dir.join("NCU_SENSORS_20260901.csv"), NCU_ID).unwrap();
    ingestar_hsu(&mut conn, data_dir.join("HSU_230_20260729.csv"), NCU_ID, "HSU_230").unwrap();
    ingestar_tcu(&mut conn, data_dir.join("TCU_001_20260729.csv"), NCU_ID, "TCU_001").unwrap();
    ingestar_event_log(&mut conn, data_dir.join("NCU_EVENT_LOG_20260729.csv"), NCU_ID).unwrap();

    let mut stmt = conn
        .prepare("SELECT ncu_id, tipo, device_id FROM dispositivos ORDER BY tipo, device_id")
        .unwrap();
    let rows: Vec<(String, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    let tipos: std::collections::HashSet<(String, String)> =
        rows.into_iter().map(|(_, t, d)| (t, d)).collect();

    assert!(tipos.contains(&("NCU".to_string(), NCU_ID.to_string())));
    assert!(tipos.contains(&("HSU".to_string(), "HSU_230".to_string())));
    assert!(tipos.contains(&("TCU".to_string(), "TCU_001".to_string())));
}

#[test]
fn test_multiples_ncus() {
    let mut conn = setup_db();
    let data_dir = get_data_dir();
    let csv_file = data_dir.join("NCU_20260729.csv");

    ingestar_ncu(&mut conn, &csv_file, "NCU_A").unwrap();
    ingestar_ncu(&mut conn, &csv_file, "NCU_B").unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT ncu_id) FROM dispositivos",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 2);

    for ncu in &["NCU_A", "NCU_B"] {
        let disp_id: i64 = conn
            .query_row(
                "SELECT id FROM dispositivos WHERE ncu_id = ?",
                [ncu],
                |r| r.get(0),
            )
            .unwrap();
        let filas: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM datos_ncu WHERE dispositivo_id = ?",
                [disp_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(filas > 0);
    }
}

#[test]
fn test_doble_ingesta_ncu_no_duplica() {
    let mut conn = setup_db();
    let data_dir = get_data_dir();
    let csv_file = data_dir.join("NCU_20260729.csv");

    ingestar_ncu(&mut conn, &csv_file, NCU_ID).unwrap();
    let count1: i64 = conn
        .query_row("SELECT COUNT(*) FROM datos_ncu", [], |r| r.get(0))
        .unwrap();

    ingestar_ncu(&mut conn, &csv_file, NCU_ID).unwrap();
    let count2: i64 = conn
        .query_row("SELECT COUNT(*) FROM datos_ncu", [], |r| r.get(0))
        .unwrap();

    assert_eq!(count1, count2);
}

#[test]
fn test_doble_ingesta_hsu_no_duplica() {
    let mut conn = setup_db();
    let data_dir = get_data_dir();
    let csv_file = data_dir.join("HSU_230_20260729.csv");

    ingestar_hsu(&mut conn, &csv_file, NCU_ID, "HSU_230").unwrap();
    let count1: i64 = conn
        .query_row("SELECT COUNT(*) FROM datos_hsu", [], |r| r.get(0))
        .unwrap();

    ingestar_hsu(&mut conn, &csv_file, NCU_ID, "HSU_230").unwrap();
    let count2: i64 = conn
        .query_row("SELECT COUNT(*) FROM datos_hsu", [], |r| r.get(0))
        .unwrap();

    assert_eq!(count1, count2);
}

#[test]
fn test_doble_ingesta_tcu_no_duplica() {
    let mut conn = setup_db();
    let data_dir = get_data_dir();
    let csv_file = data_dir.join("TCU_001_20260729.csv");

    ingestar_tcu(&mut conn, &csv_file, NCU_ID, "TCU_001").unwrap();
    let count1: i64 = conn
        .query_row("SELECT COUNT(*) FROM datos_tcu", [], |r| r.get(0))
        .unwrap();

    ingestar_tcu(&mut conn, &csv_file, NCU_ID, "TCU_001").unwrap();
    let count2: i64 = conn
        .query_row("SELECT COUNT(*) FROM datos_tcu", [], |r| r.get(0))
        .unwrap();

    assert_eq!(count1, count2);
}
