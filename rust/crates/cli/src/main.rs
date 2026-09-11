use std::path::PathBuf;
use anyhow::Result;
use clap::{Parser, Subcommand};
use mini_nube_csv::parse_bool_lenient;
use mini_nube_db::{conectar, inicializar, ingestar_directorio, ingestar_fichero, obtener_resumen_tablas};

#[derive(Parser, Debug)]
#[command(
    name = "mini-nube",
    author,
    version,
    about = "Mini Nube - Ingesta de telemetría y eventos solares en SQLite",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Ingesta un único fichero CSV en la base de datos
    IngestFile {
        /// Ruta de la base de datos SQLite
        #[arg(long, default_value = "../data/Poggiorsini/Poggiorsini.db")]
        db_path: PathBuf,

        /// Identificador de la NCU
        #[arg(long, default_value = "PR6-1075-DIMAURO")]
        ncu_id: String,

        /// Ruta del fichero CSV que se va a ingestar
        #[arg(long, default_value = "../data/Poggiorsini/PR6-1075-DIMAURO/2026-09-01/TCU_008_2026-09-01.csv")]
        file_path: PathBuf,
    },

    /// Ingesta todos los CSV de una carpeta en la base de datos
    IngestFolder {
        /// Ruta de la base de datos SQLite
        #[arg(long, default_value = "../data/Poggiorsini/Poggiorsini.db")]
        db_path: PathBuf,

        /// Identificador de la NCU
        #[arg(long, default_value = "PR6-1075-DIMAURO")]
        ncu_id: String,

        /// Carpeta que contiene los CSV de la planta
        #[arg(long, default_value = "../data/Poggiorsini/PR6-1075-DIMAURO")]
        plant_folder: PathBuf,

        /// Omite los ficheros ya registrados en la tabla de ingesta
        #[arg(
            long,
            default_value = "true",
            default_missing_value = "true",
            num_args = 0..=1,
            value_parser = parse_bool_lenient
        )]
        skip_files_already_inserted: bool,
    },

    /// Muestra el resumen de tablas y número de filas en la base de datos
    ResumeDb {
        /// Ruta de la base de datos SQLite
        #[arg(long, default_value = "../data/Poggiorsini/Poggiorsini.db")]
        db_path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::IngestFile {
            db_path,
            ncu_id,
            file_path,
        } => {
            let mut conn = conectar(&db_path)?;
            inicializar(&conn)?;
            let count = ingestar_fichero(&mut conn, &file_path, &ncu_id)?;
            println!("Ingestado '{}': {} filas procesadas.", file_path.display(), count);
        }
        Commands::IngestFolder {
            db_path,
            ncu_id,
            plant_folder,
            skip_files_already_inserted,
        } => {
            let mut conn = conectar(&db_path)?;
            inicializar(&conn)?;
            let count = ingestar_directorio(
                &mut conn,
                &plant_folder,
                &ncu_id,
                skip_files_already_inserted,
            )?;
            println!("Ingesta de carpeta finalizada. Total de filas procesadas: {}", count);
        }
        Commands::ResumeDb { db_path } => {
            let conn = conectar(&db_path)?;
            let summaries = obtener_resumen_tablas(&conn)?;
            println!("Resumen de la base de datos:");
            for s in summaries {
                println!("Tabla: {}, Número de filas: {}", s.name, s.row_count);
            }
        }
    }

    Ok(())
}
