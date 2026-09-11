use std::path::Path;
use rusqlite::Connection;

use crate::error::DbError;

/// Abre (o crea) la base de datos y activa WAL y foreign keys.
pub fn conectar(db_path: impl AsRef<Path>) -> Result<Connection, DbError> {
    let path = db_path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let conn = Connection::open(path)?;
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;
        "#,
    )?;
    Ok(conn)
}
