// Lesson 1.3 — Expressions and the Lazy API
// Demonstrates: building a LazyFrame, writing col/lit expressions, chaining
// transforms, and calling collect() to materialise results.

use polars::prelude::*;

fn main() -> PolarsResult<()> {
    let csv = r#"name,variety,region,rating
Achaval Ferrer Malbec,Malbec,Mendoza,94
Catena Zapata,Malbec,Mendoza,96
Louis Jadot Beaune,Pinot Noir,Burgundy,91
Elk Cove,Pinot Noir,Oregon,89
Cloudy Bay,Sauvignon Blanc,Marlborough,90
Domaine Weinbach,Riesling,Alsace,93
"#;

    // Convert the eager DataFrame to a LazyFrame — no work happens yet.
    let lf = CsvReadOptions::default()
        .with_has_header(true)
        .into_reader_with_file_handle(std::io::Cursor::new(csv))
        .finish()?
        .lazy();

    // --- Expressions: col, lit, alias ---
    // Polars builds a logical plan; nothing executes until collect().
    let result = lf
        .filter(col("rating").gt(lit(90i32)))
        .with_column((col("rating") - lit(88i32)).alias("points_above_88"))
        .select([
            col("name"),
            col("variety"),
            col("rating"),
            col("points_above_88"),
        ])
        .sort(
            ["rating"],
            SortMultipleOptions::default().with_order_descending(true),
        )
        .collect()?;

    println!("=== Wines rated above 90 ===");
    println!("{}", result);

    // --- Inspect the query plan before collecting ---
    let lf2 = CsvReadOptions::default()
        .with_has_header(true)
        .into_reader_with_file_handle(std::io::Cursor::new(csv))
        .finish()?
        .lazy()
        .filter(col("region").eq(lit("Mendoza")))
        .select([col("name"), col("rating")]);

    println!("\n=== Logical plan ===");
    println!("{}", lf2.describe_plan()?);

    println!("\n=== Collected result ===");
    println!("{}", lf2.collect()?);

    Ok(())
}
