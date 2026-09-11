use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use mini_nube_csv::parse_bool_lenient;
use mini_nube_db::{conectar, inicializar, ingestar_directorio};

/// Inicializa la base de datos e ingesta todos los CSV de una carpeta.
#[derive(Parser, Debug)]
#[command(author, version, about = "Inicializa la base de datos e ingesta todos los CSV de una carpeta", long_about = None)]
struct Args {
    /// Ruta de la base de datos SQLite.
    #[arg(long, default_value = "../data/Poggiorsini/Poggiorsini.db")]
    db_path: PathBuf,

    /// Identificador de la NCU.
    #[arg(long, default_value = "PR6-1075-DIMAURO")]
    ncu_id: String,

    /// Carpeta que contiene los CSV de la planta.
    #[arg(long, default_value = "../data/Poggiorsini/PR6-1075-DIMAURO")]
    plant_folder: PathBuf,

    /// Omite los ficheros ya registrados en la tabla de ingesta.
    #[arg(
        long,
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1,
        value_parser = parse_bool_lenient
    )]
    skip_files_already_inserted: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut conn = conectar(&args.db_path)?;
    inicializar(&conn)?;
    let count = ingestar_directorio(
        &mut conn,
        &args.plant_folder,
        &args.ncu_id,
        args.skip_files_already_inserted,
    )?;
    println!(
        "Ingesta de carpeta finalizada. Total de filas procesadas: {}",
        count
    );
    Ok(())
}
