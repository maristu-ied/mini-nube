use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use mini_nube_db::{conectar, inicializar, ingestar_fichero};

/// Inicializa la base de datos e ingesta un único fichero CSV.
#[derive(Parser, Debug)]
#[command(author, version, about = "Inicializa la base de datos e ingesta un único fichero CSV", long_about = None)]
struct Args {
    /// Ruta de la base de datos SQLite.
    #[arg(long, default_value = "../data/Poggiorsini/Poggiorsini.db")]
    db_path: PathBuf,

    /// Identificador de la NCU.
    #[arg(long, default_value = "PR6-1075-DIMAURO")]
    ncu_id: String,

    /// Ruta del fichero CSV que se va a ingestar.
    #[arg(
        long = "file-path",
        default_value = "../data/Poggiorsini/PR6-1075-DIMAURO/2026-09-01/TCU_008_2026-09-01.csv"
    )]
    singlefile_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut conn = conectar(&args.db_path)?;
    inicializar(&conn)?;
    let count = ingestar_fichero(&mut conn, &args.singlefile_path, &args.ncu_id)?;
    println!("Ingestado '{}': {} filas procesadas.", args.singlefile_path.display(), count);
    Ok(())
}
