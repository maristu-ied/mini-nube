use rusqlite::{params, Connection, OptionalExtension};

use crate::error::DbError;

/// Devuelve el id del dispositivo, creándolo si no existe.
pub fn obtener_o_crear_dispositivo(
    conn: &Connection,
    ncu_id: &str,
    tipo: &str,
    device_id: &str,
) -> Result<i64, DbError> {
    let mut stmt = conn.prepare_cached(
        "SELECT id FROM dispositivos WHERE ncu_id = ? AND tipo = ? AND device_id = ?",
    )?;
    let row: Option<i64> = stmt
        .query_row(params![ncu_id, tipo, device_id], |r| r.get(0))
        .optional()?;

    if let Some(id) = row {
        return Ok(id);
    }

    conn.execute(
        "INSERT INTO dispositivos (ncu_id, tipo, device_id) VALUES (?, ?, ?)",
        params![ncu_id, tipo, device_id],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Cuenta cuántos de los timestamps dados ya existen en la tabla para el dispositivo_id.
pub fn contar_existentes(
    conn: &Connection,
    tabla: &str,
    disp_id: i64,
    timestamps: &[i64],
) -> Result<usize, DbError> {
    if timestamps.is_empty() {
        return Ok(0);
    }

    let mut total = 0;
    // Lotes de 900 para no exceder el límite de variables bind de SQLite
    for chunk in timestamps.chunks(900) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT COUNT(*) FROM {} WHERE dispositivo_id = ? AND timestamp IN ({})",
            tabla, placeholders
        );
        let mut stmt = conn.prepare(&sql)?;

        let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(chunk.len() + 1);
        params_vec.push(&disp_id);
        for ts in chunk {
            params_vec.push(ts);
        }

        let count: usize = stmt.query_row(rusqlite::params_from_iter(params_vec), |r| r.get(0))?;
        total += count;
    }

    Ok(total)
}

/// Cuenta cuántos eventos ya existen en ncu_event_log.
pub fn contar_eventos_existentes(
    conn: &Connection,
    disp_id: i64,
    eventos: &[(i64, &str)],
) -> Result<usize, DbError> {
    if eventos.is_empty() {
        return Ok(0);
    }

    let mut stmt = conn.prepare_cached(
        "SELECT 1 FROM ncu_event_log WHERE dispositivo_id = ? AND timestamp = ? AND evento = ?",
    )?;

    let mut total = 0;
    for (ts, ev) in eventos {
        let exists: Option<i64> = stmt
            .query_row(params![disp_id, ts, ev], |r| r.get(0))
            .optional()?;
        if exists.is_some() {
            total += 1;
        }
    }

    Ok(total)
}

/// Registra la ingesta en la tabla `ingesta_log`.
pub fn registrar_ingesta(
    conn: &Connection,
    ncu_id: &str,
    fichero: &str,
    tipo_datos: &str,
    filas_nuevas: usize,
    filas_actualizadas: usize,
    timestamp_inicio: Option<i64>,
    timestamp_fin: Option<i64>,
) -> Result<(), DbError> {
    conn.execute(
        r#"
        INSERT OR REPLACE INTO ingesta_log
           (ncu_id, fichero, tipo_datos, filas_insertadas, filas_nuevas, filas_actualizadas,
            timestamp_inicio, timestamp_fin)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        params![
            ncu_id,
            fichero,
            tipo_datos,
            (filas_nuevas + filas_actualizadas) as i64,
            filas_nuevas as i64,
            filas_actualizadas as i64,
            timestamp_inicio,
            timestamp_fin,
        ],
    )?;
    Ok(())
}

/// Información de resumen de una tabla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableSummary {
    pub name: String,
    pub row_count: usize,
}

/// Obtiene el listado de tablas y su número de filas.
pub fn obtener_resumen_tablas(conn: &Connection) -> Result<Vec<TableSummary>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )?;
    let table_names: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;

    let mut summaries = Vec::new();
    for name in table_names {
        let sql = format!("SELECT COUNT(*) FROM \"{}\"", name.replace('"', "\"\""));
        let count: usize = conn.query_row(&sql, [], |r| r.get(0))?;
        summaries.push(TableSummary {
            name,
            row_count: count,
        });
    }

    Ok(summaries)
}
