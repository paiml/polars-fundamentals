// Lesson 1.1 — What is Polars and why use it with Rust?
// Demonstrates: adding polars to a project, verifying the setup, and printing
// a small in-memory DataFrame to confirm the Arrow-backed engine is working.

use polars::prelude::*;

fn main() -> PolarsResult<()> {
    // Build a tiny DataFrame from Rust vecs — no file needed to get started.
    // Each Series maps to one column; Polars stores them contiguously in Arrow arrays.
    let df = DataFrame::new(vec![
        Column::new("wine".into(), &["Malbec", "Chardonnay", "Pinot Noir"]),
        Column::new("rating".into(), &[92.0_f64, 88.0, 95.0]),
        Column::new("region".into(), &["Mendoza", "Burgundy", "Oregon"]),
    ])?;

    println!("DataFrame shape: {:?}", df.shape()); // (rows, cols)
    println!("{}", df);

    // Polars version — useful to confirm the dependency resolved correctly.
    println!("\nPolars version: {}", polars::VERSION);

    Ok(())
}
