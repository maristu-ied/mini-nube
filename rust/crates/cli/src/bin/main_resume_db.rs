use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use mini_nube_db::{conectar, obtener_resumen_tablas};

/// Muestra el número de filas de cada tabla de la base de datos.
#[derive(Parser, Debug)]
#[command(author, version, about = "Muestra el número de filas de cada tabla de la base de datos", long_about = None)]
struct Args {
    /// Ruta de la base de datos SQLite.
    #[arg(long, default_value = "../data/Poggiorsini/Poggiorsini.db")]
    db_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let conn = conectar(&args.db_path)?;
    let summaries = obtener_resumen_tablas(&conn)?;
    println!("Resumen de la base de datos:");
    for s in summaries {
        println!("Tabla: {}, Número de filas: {}", s.name, s.row_count);
    }
    Ok(())
}
