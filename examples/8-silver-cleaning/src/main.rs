// Lesson 3.2 — Silver: Cleaning and Standardizing
// Demonstrates: loading the bronze SQLite table into a Polars LazyFrame,
// applying the cleaning pipeline as reusable functions, writing clean_wines,
// and printing a summary of changes to stdout.

use anyhow::Result;
use clap::Parser;
use polars::prelude::*;
use rusqlite::{params, Connection};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "silver",
    about = "Clean bronze data and write the silver layer"
)]
struct Cli {
    /// Path to the SQLite database file
    #[arg(long, env = "WINE_DB", default_value = "wine.db")]
    db: PathBuf,
}

fn load_raw(conn: &Connection) -> Result<DataFrame> {
    let mut stmt = conn.prepare("SELECT name, variety, region, rating, notes FROM raw_wines")?;

    let mut names: Vec<Option<String>> = vec![];
    let mut variety: Vec<Option<String>> = vec![];
    let mut regions: Vec<Option<String>> = vec![];
    let mut ratings: Vec<Option<String>> = vec![];
    let mut notes: Vec<Option<String>> = vec![];

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, Option<String>>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
        ))
    })?;

    for row in rows {
        let (n, v, r, rt, nt) = row?;
        names.push(n);
        variety.push(v);
        regions.push(r);
        ratings.push(rt);
        notes.push(nt);
    }

    Ok(DataFrame::new(vec![
        Column::new("name".into(), names),
        Column::new("variety".into(), variety),
        Column::new("region".into(), regions),
        Column::new("rating".into(), ratings),
        Column::new("notes".into(), notes),
    ])?)
}

fn clean(df: DataFrame) -> PolarsResult<DataFrame> {
    df.lazy()
        .drop_nulls(Some(vec![col("name"), col("rating")]))
        .with_column(col("rating").cast(DataType::Float64))
        .filter(
            col("rating")
                .gt_eq(lit(80.0_f64))
                .and(col("rating").lt_eq(lit(100.0_f64))),
        )
        .with_column(
            col("variety")
                .str()
                .strip_chars(lit(" "))
                .str()
                .to_uppercase(),
        )
        .with_column(col("notes").fill_null(lit("")))
        .unique(None, UniqueKeepStrategy::First)
        .collect()
}

fn write_silver(conn: &Connection, df: &DataFrame) -> Result<()> {
    conn.execute_batch(
        "DROP TABLE IF EXISTS clean_wines;
         CREATE TABLE clean_wines (
             id      INTEGER PRIMARY KEY,
             name    TEXT    NOT NULL,
             variety TEXT    NOT NULL,
             region  TEXT,
             rating  REAL    NOT NULL,
             notes   TEXT    NOT NULL DEFAULT ''
         );",
    )?;

    let names = df.column("name")?.str()?;
    let variety = df.column("variety")?.str()?;
    let region = df.column("region")?.str()?;
    let rating = df.column("rating")?.f64()?;
    let notes = df.column("notes")?.str()?;

    let tx = conn.unchecked_transaction()?;
    for i in 0..df.height() {
        tx.execute(
            "INSERT INTO clean_wines (name, variety, region, rating, notes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                names.get(i).unwrap_or(""),
                variety.get(i).unwrap_or(""),
                region.get(i),
                rating.get(i),
                notes.get(i).unwrap_or(""),
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conn = Connection::open(&cli.db)?;

    let raw = load_raw(&conn)?;
    let raw_count = raw.height();
    println!("Loaded {raw_count} rows from raw_wines");

    let clean = clean(raw)?;
    let clean_count = clean.height();

    write_silver(&conn, &clean)?;

    println!("Wrote {clean_count} rows to clean_wines");
    println!(
        "Summary: {} rows dropped, {} nulls in rating handled",
        raw_count - clean_count,
        raw_count - clean_count
    );

    Ok(())
}
