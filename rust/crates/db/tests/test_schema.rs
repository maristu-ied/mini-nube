use mini_nube_db::{conectar, inicializar, obtener_o_crear_dispositivo};
use rusqlite::Connection;
use std::collections::HashSet;
use tempfile::NamedTempFile;

fn setup_db() -> Connection {
    let tmp = NamedTempFile::new().unwrap();
    let conn = conectar(tmp.path()).unwrap();
    inicializar(&conn).unwrap();
    conn
}

#[test]
fn test_crear_tablas() {
    let conn = setup_db();
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
        .unwrap();
    let tables: HashSet<String> = stmt
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    let esperadas: HashSet<String> = [
        "dispositivos",
        "datos_ncu",
        "datos_ncu_sensor",
        "datos_hsu",
        "datos_tcu",
        "ncu_event_log",
        "ingesta_log",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    assert_eq!(esperadas, tables);
}

#[test]
fn test_crear_indices_unique() {
    let conn = setup_db();
    let mut stmt = conn
        .prepare("SELECT name, sql FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
        .unwrap();

    let indices: Vec<(String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    let names: HashSet<String> = indices.iter().map(|(n, _)| n.clone()).collect();
    let esperados: HashSet<String> = [
        "idx_datos_ncu_ts",
        "idx_datos_ncu_sensor_ts",
        "idx_datos_hsu_ts",
        "idx_datos_tcu_ts",
        "idx_ncu_event_log_ts",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    assert_eq!(esperados, names);

    for (name, sql) in indices {
        assert!(
            sql.to_uppercase().contains("UNIQUE"),
            "{} debería ser UNIQUE",
            name
        );
    }
}

#[test]
fn test_dispositivo_unico() {
    let conn = setup_db();
    let id1 = obtener_o_crear_dispositivo(&conn, "NCU_001", "HSU", "HSU_230").unwrap();
    let id2 = obtener_o_crear_dispositivo(&conn, "NCU_001", "HSU", "HSU_230").unwrap();
    assert_eq!(id1, id2);
}

#[test]
fn test_dispositivos_distintos() {
    let conn = setup_db();
    let id1 = obtener_o_crear_dispositivo(&conn, "NCU_001", "HSU", "HSU_230").unwrap();
    let id2 = obtener_o_crear_dispositivo(&conn, "NCU_001", "TCU", "TCU_001").unwrap();
    let id3 = obtener_o_crear_dispositivo(&conn, "NCU_002", "HSU", "HSU_230").unwrap();
    assert_ne!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_wal_mode() {
    let conn = setup_db();
    let mode: String = conn
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap();
    assert_eq!(mode.to_lowercase(), "wal");
}

#[test]
fn test_foreign_keys() {
    let conn = setup_db();
    let fk: i32 = conn
        .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
        .unwrap();
    assert_eq!(fk, 1);
}
