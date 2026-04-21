// Lesson 2.1 — Data Cleaning
// Demonstrates: detecting nulls, drop-vs-fill strategy, casting rating strings
// to f64, normalizing text fields, and filtering invalid rows.

use polars::prelude::*;

fn main() -> PolarsResult<()> {
    // Raw data as it might come from wine-ratings.csv — messy on purpose.
    let csv = r#"name,variety,region,rating,notes
Achaval Ferrer Malbec,malbec ,Mendoza,94,Dark fruit
Catena Zapata, MALBEC,Mendoza,,
Louis Jadot Beaune,Pinot Noir,Burgundy,91,Cherry
Elk Cove,pinot noir,Oregon,999,Red berry
Cloudy Bay,Sauvignon Blanc,Marlborough,90,Citrus
,Riesling,Alsace,93,Honey
"#;

    let df = CsvReader::new(std::io::Cursor::new(csv))
        .has_header(true)
        .finish()?;

    println!("=== Raw data ===");
    println!("{}\n", df);

    println!("=== Null counts (raw) ===");
    println!("{}\n", df.null_count());

    // Save the original count before the cleaning pipeline consumes `df`.
    let raw_rows = df.height();

    // --- Cleaning pipeline as a lazy chain ---
    let cleaned = df
        .lazy()
        // Drop rows where 'name' or 'rating' is null — these are unrecoverable.
        .drop_nulls(Some(vec![col("name"), col("rating")]))
        // Cast rating to f64 for numeric operations.
        .with_column(col("rating").cast(DataType::Float64))
        // Filter out-of-range ratings (valid: 80–100).
        .filter(col("rating").gt_eq(lit(80.0_f64)).and(col("rating").lt_eq(lit(100.0_f64))))
        // Normalize variety: trim whitespace, then uppercase.
        .with_column(
            col("variety")
                .str()
                .strip_chars(lit(" "))
                .str()
                .to_uppercase()
                .alias("variety"),
        )
        // Fill missing notes with an empty string.
        .with_column(col("notes").fill_null(lit("")))
        .collect()?;

    println!("=== Cleaned data ===");
    println!("{}\n", cleaned);

    println!("Rows dropped: {}", raw_rows - cleaned.height());

    Ok(())
}
