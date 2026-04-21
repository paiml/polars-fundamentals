// Lesson 2.2 — Sorting, Filtering, and Aggregation
// Demonstrates: filter by threshold, multi-column sort, group_by average,
// and value_counts for most-reviewed regions.

use polars::prelude::*;

fn main() -> PolarsResult<()> {
    let csv = r#"name,variety,region,rating
Achaval Ferrer Malbec,Malbec,Mendoza,94
Catena Zapata,Malbec,Mendoza,96
Luigi Bosca,Malbec,Mendoza,88
Louis Jadot Beaune,Pinot Noir,Burgundy,91
Domaine Dujac,Pinot Noir,Burgundy,95
Elk Cove,Pinot Noir,Oregon,89
Ponzi Vineyards,Pinot Noir,Oregon,90
Cloudy Bay,Sauvignon Blanc,Marlborough,90
Kim Crawford,Sauvignon Blanc,Marlborough,87
Domaine Weinbach,Riesling,Alsace,93
"#;

    let lf = CsvReader::new(std::io::Cursor::new(csv))
        .has_header(true)
        .finish()?
        .lazy();

    // --- Filter: wines rated 90 or above from a specific region ---
    println!("=== Mendoza wines rated >= 90 ===");
    let mendoza_top = lf
        .clone()
        .filter(
            col("region")
                .eq(lit("Mendoza"))
                .and(col("rating").cast(DataType::Float64).gt_eq(lit(90.0_f64))),
        )
        .collect()?;
    println!("{}\n", mendoza_top);

    // --- Sort: rating descending, name ascending ---
    // SortMultipleOptions lets you set a direction per column.
    println!("=== All wines sorted by rating desc, name asc ===");
    let sorted = lf
        .clone()
        .sort(
            ["rating", "name"],
            SortMultipleOptions::new()
                .with_order_descending(true)   // applies to "rating"
                .with_order_descending(false),  // applies to "name"
        )
        .collect()?;
    println!("{}\n", sorted);

    // --- Group-by: average rating per variety ---
    println!("=== Average rating per grape variety ===");
    let avg_by_variety = lf
        .clone()
        .with_column(col("rating").cast(DataType::Float64))
        .group_by([col("variety")])
        .agg([col("rating").mean().alias("avg_rating"), col("rating").count().alias("count")])
        .sort(["avg_rating"], SortMultipleOptions::default().with_order_descending(true))
        .collect()?;
    println!("{}\n", avg_by_variety);

    // --- value_counts: most-reviewed regions ---
    // group_by + count is the explicit equivalent of value_counts — easier to read.
    println!("=== Most-reviewed regions ===");
    let region_counts = lf
        .clone()
        .group_by([col("region")])
        .agg([col("region").count().alias("count")])
        .sort(["count"], SortMultipleOptions::default().with_order_descending(true))
        .collect()?;
    println!("{}", region_counts);

    Ok(())
}
