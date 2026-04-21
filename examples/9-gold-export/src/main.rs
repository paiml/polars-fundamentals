// Lesson 3.3 — Gold: Business Logic and Export
// Demonstrates: reading clean_wines from SQLite into Polars, filtering by a
// configurable min-rating, computing top-variety aggregations, and exporting
// results to CSV and JSON.

use anyhow::Result;
use clap::Parser;
use polars::prelude::*;
use rusqlite::Connection;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gold", about = "Apply business logic and export gold-layer results")]
struct Cli {
    /// Minimum rating threshold (inclusive)
    #[arg(long, default_value_t = 90.0)]
    min_rating: f64,

    /// Path to the SQLite database file
    #[arg(long, env = "WINE_DB", default_value = "wine.db")]
    db: PathBuf,
}

fn load_clean(conn: &Connection) -> Result<DataFrame> {
    let mut stmt = conn.prepare(
        "SELECT name, variety, region, rating FROM clean_wines",
    )?;

    let mut names:   Vec<String>        = vec![];
    let mut variety: Vec<String>        = vec![];
    let mut regions: Vec<Option<String>>= vec![];
    let mut ratings: Vec<f64>           = vec![];

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, f64>(3)?,
        ))
    })?;

    for row in rows {
        let (n, v, r, rt) = row?;
        names.push(n);
        variety.push(v);
        regions.push(r);
        ratings.push(rt);
    }

    Ok(DataFrame::new(vec![
        Series::new("name".into(),    names),
        Series::new("variety".into(), variety),
        Series::new("region".into(),  regions),
        Series::new("rating".into(),  ratings),
    ])?)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let min_rating = cli.min_rating;
    let conn = Connection::open(&cli.db)?;
    let df = load_clean(&conn)?;

    // --- Filter by min-rating ---
    let filtered = df
        .lazy()
        .filter(col("rating").gt_eq(lit(min_rating)))
        .collect()?;
    println!("Wines above {min_rating}: {}", filtered.height());

    // --- Top-10 varieties by average rating ---
    println!("\n=== Top varieties by avg rating ===");
    let top_varieties = filtered
        .lazy()
        .group_by([col("variety")])
        .agg([
            col("rating").mean().alias("avg_rating"),
            col("rating").count().alias("count"),
        ])
        .sort(["avg_rating"], SortMultipleOptions::default().with_order_descending(true))
        .limit(10)
        .collect()?;
    println!("{}", top_varieties);

    // --- Highest-rated regions ---
    println!("\n=== Highest-rated regions ===");
    let top_regions = filtered
        .lazy()
        .group_by([col("region")])
        .agg([col("rating").mean().alias("avg_rating")])
        .sort(["avg_rating"], SortMultipleOptions::default().with_order_descending(true))
        .collect()?;
    println!("{}", top_regions);

    // --- Export to CSV ---
    let csv_path = PathBuf::from("gold_wines.csv");
    let mut csv_file = std::fs::File::create(&csv_path)?;
    CsvWriter::new(&mut csv_file).finish(&mut filtered.clone())?;
    println!("\nExported CSV: {}", csv_path.display());

    // --- Export to JSON ---
    let json_path = PathBuf::from("gold_wines.json");
    let mut json_file = std::fs::File::create(&json_path)?;
    JsonWriter::new(&mut json_file)
        .with_json_format(JsonFormat::Json)
        .finish(&mut filtered.clone())?;
    println!("Exported JSON: {}", json_path.display());

    Ok(())
}
