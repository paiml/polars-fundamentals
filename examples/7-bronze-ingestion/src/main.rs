// Lesson 3.1 — Bronze: Ingesting Raw Data
// Demonstrates: CLI architecture with clap, reading wine-ratings.csv with
// Polars, and bulk-inserting raw rows into a SQLite raw_wines table with an
// ingested_at timestamp — no business logic applied.

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use polars::prelude::*;
use rusqlite::{params, Connection};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bronze", about = "Ingest raw wine-ratings.csv into SQLite")]
struct Cli {
    /// Path to wine-ratings.csv
    #[arg(long, default_value = "wine-ratings.csv")]
    input: PathBuf,

    /// Path to the SQLite database file
    #[arg(long, env = "WINE_DB", default_value = "wine.db")]
    db: PathBuf,
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS raw_wines (
            id           INTEGER PRIMARY KEY,
            name         TEXT,
            variety      TEXT,
            region       TEXT,
            rating       TEXT,
            notes        TEXT,
            ingested_at  TEXT NOT NULL
        );",
    )?;
    Ok(())
}

fn ingest(conn: &Connection, df: &DataFrame) -> Result<usize> {
    let ingested_at = Utc::now().to_rfc3339();

    let names = df.column("name")?.str()?;
    let variety = df.column("variety")?.str()?;
    let region = df.column("region")?.str()?;
    let rating_series = df.column("rating")?.cast(&DataType::String)?;
    let rating = rating_series.str()?;
    let notes = df.column("notes")?.str()?;

    let tx = conn.unchecked_transaction()?;
    for i in 0..df.height() {
        tx.execute(
            "INSERT INTO raw_wines (name, variety, region, rating, notes, ingested_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                names.get(i),
                variety.get(i),
                region.get(i),
                rating.get(i),
                notes.get(i),
                ingested_at,
            ],
        )?;
    }
    tx.commit()?;
    Ok(df.height())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let df = CsvReadOptions::default()
        .try_into_reader_with_file_path(Some((&cli.input).into()))?
        .finish()?;
    println!("Read {} rows from {}", df.height(), cli.input.display());

    let conn = Connection::open(&cli.db)?;
    init_schema(&conn)?;

    let n = ingest(&conn, &df)?;
    println!("Ingested {n} rows into raw_wines ({})", cli.db.display());

    Ok(())
}
