// Lesson 1.2 — DataFrames and Series
// Demonstrates: reading wine-ratings.csv with CsvReader, inspecting schema,
// selecting columns, slicing rows, and examining null counts.

use polars::prelude::*;

fn main() -> PolarsResult<()> {
    // --- Load CSV ---
    // CsvReader infers column types from the first rows.
    // Pass a real path to wine-ratings.csv when running outside this example.
    let csv = r#"name,variety,region,rating,notes
Achaval Ferrer Malbec,Malbec,Mendoza,94,"Dark fruit, tobacco"
Catena Zapata,Malbec,Mendoza,,
Louis Jadot Beaune,Pinot Noir,Burgundy,91,"Cherry, earthy"
Elk Cove,Pinot Noir,Oregon,89,"Red berry, silky"
Cloudy Bay,Sauvignon Blanc,Marlborough,90,"Citrus, herbaceous"
"#;

    let df = CsvReadOptions::default()
        .with_has_header(true)
        .into_reader_with_file_handle(std::io::Cursor::new(csv))
        .finish()?;

    // --- Schema inspection ---
    println!("=== Schema ===");
    println!("{:?}", df.schema());

    println!("\n=== Column names ===");
    println!("{:?}", df.get_column_names());

    println!("\n=== Null counts ===");
    println!("{}", df.null_count());

    // --- Shape ---
    let (rows, cols) = df.shape();
    println!("\nRows: {rows}  Cols: {cols}");

    // --- Column selection ---
    println!("\n=== 'name' and 'rating' columns ===");
    let subset = df.select(["name", "rating"])?;
    println!("{}", subset);

    // --- Row slicing ---
    println!("\n=== First 3 rows ===");
    println!("{}", df.slice(0, 3));

    Ok(())
}
