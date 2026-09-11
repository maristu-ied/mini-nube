use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Error de SQLite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Error de CSV: {0}")]
    Csv(#[from] mini_nube_csv::CsvError),

    #[error("Error de E/S: {0}")]
    Io(#[from] std::io::Error),

    #[error("Fichero o directorio no encontrado: {0}")]
    NotFound(String),

    #[error("Error de formato en ruta: {0}")]
    PathError(String),
}
