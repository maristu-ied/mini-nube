use thiserror::Error;

#[derive(Debug, Error)]
pub enum CsvError {
    #[error("Error de E/S: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error de formato CSV: {0}")]
    Csv(#[from] csv::Error),

    #[error("Error al parsear fecha/hora '{0}': {1}")]
    DateTimeParse(String, chrono::ParseError),

    #[error("Error al parsear número '{0}': {1}")]
    NumberParse(String, String),

    #[error("Línea de log de eventos no válida: '{0}'")]
    InvalidEventLogLine(String),

    #[error("Tipo de archivo no reconocido por su nombre: '{0}'")]
    UnrecognizedFileType(String),
}
