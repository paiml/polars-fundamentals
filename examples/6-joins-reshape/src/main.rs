// Lesson 2.3 — Joining and Reshaping Data
// Demonstrates: left join to enrich the wine DataFrame, melt for wide-to-long,
// and writing output to CSV and Parquet.

use polars::prelude::*;

fn main() -> anyhow::Result<()> {
    // Main wine DataFrame
    let wine_csv = r#"name,variety,region,rating
Achaval Ferrer Malbec,Malbec,Mendoza,94
Catena Zapata,Malbec,Mendoza,96
Louis Jadot Beaune,Pinot Noir,Burgundy,91
Elk Cove,Pinot Noir,Oregon,89
Cloudy Bay,Sauvignon Blanc,Marlborough,90
"#;

    // Lookup table: variety → wine family
    let family_csv = r#"variety,family
Malbec,Red
Pinot Noir,Red
Sauvignon Blanc,White
Chardonnay,White
Riesling,White
"#;

    let wines = CsvReadOptions::default()
        .with_has_header(true)
        .into_reader_with_file_handle(std::io::Cursor::new(wine_csv))
        .finish()?;

    let families = CsvReadOptions::default()
        .with_has_header(true)
        .into_reader_with_file_handle(std::io::Cursor::new(family_csv))
        .finish()?;

    // --- Left join: enrich wines with wine family ---
    println!("=== Wines enriched with family ===");
    let enriched = wines
        .lazy()
        .join(
            families.lazy(),
            [col("variety")],
            [col("variety")],
            JoinArgs::new(JoinType::Left),
        )
        .collect()?;
    println!("{}\n", enriched);

    // --- Melt: wide-to-long ---
    // unpivot keeps 'name' and 'family' as identifier columns and turns
    // 'rating' into a value row — useful for reporting tools that expect long format.
    println!("=== Wide-to-long with melt ===");
    let enriched_f64 = enriched
        .clone()
        .lazy()
        .with_column(col("rating").cast(DataType::Float64))
        .collect()?;
    let melted = enriched_f64.unpivot(["rating"], ["name", "family"])?;
    println!("{}\n", melted);

    // --- Write cleaned DataFrame to CSV ---
    println!("=== CSV export ===");
    let mut csv_file = std::fs::File::create("enriched_wines.csv")?;
    CsvWriter::new(&mut csv_file).finish(&mut enriched.clone())?;
    println!("Wrote enriched_wines.csv");

    // --- Write to Parquet (in-memory bytes to show the API) ---
    let mut parquet_buf: Vec<u8> = Vec::new();
    ParquetWriter::new(&mut parquet_buf).finish(&mut enriched.clone())?;
    println!("\nParquet bytes written: {}", parquet_buf.len());

    Ok(())
}
