use anyhow::Result;
use chrono::Utc;
use polars::prelude::*;
use rusqlite::{params, Connection};
use std::path::Path;

pub fn open_db(path: &Path) -> Result<Connection> {
    Ok(Connection::open(path)?)
}

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS raw_wines (
            id           INTEGER PRIMARY KEY,
            name         TEXT,
            variety      TEXT,
            region       TEXT,
            rating       TEXT,
            notes        TEXT,
            ingested_at  TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS clean_wines (
            id      INTEGER PRIMARY KEY,
            name    TEXT    NOT NULL,
            variety TEXT    NOT NULL,
            region  TEXT,
            rating  REAL    NOT NULL,
            notes   TEXT    NOT NULL DEFAULT ''
        );",
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Bronze
// ---------------------------------------------------------------------------

pub fn bronze(conn: &Connection, csv_path: &Path) -> Result<usize> {
    let df = CsvReadOptions::default()
        .try_into_reader_with_file_path(Some((csv_path).into()))?
        .finish()?;
    let ingested_at = Utc::now().to_rfc3339();

    let names = df.column("name")?.str()?;
    let variety = df.column("variety")?.str()?;
    let region = df.column("region")?.str()?;
    let rating = df.column("rating")?.cast(&DataType::String)?;
    let rating = rating.str()?;
    let notes = df.column("notes")?.str()?;

    conn.execute("DELETE FROM raw_wines;", [])?;
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

// ---------------------------------------------------------------------------
// Silver
// ---------------------------------------------------------------------------

fn load_raw(conn: &Connection) -> Result<DataFrame> {
    let mut stmt = conn.prepare("SELECT name, variety, region, rating, notes FROM raw_wines")?;

    let mut names: Vec<Option<String>> = vec![];
    let mut variety: Vec<Option<String>> = vec![];
    let mut regions: Vec<Option<String>> = vec![];
    let mut ratings: Vec<Option<String>> = vec![];
    let mut notes: Vec<Option<String>> = vec![];

    for row in stmt.query_map([], |row| {
        Ok((
            row.get::<_, Option<String>>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
        ))
    })? {
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

pub fn silver(conn: &Connection) -> Result<(usize, usize)> {
    let raw = load_raw(conn)?;
    let raw_count = raw.height();

    let clean = raw
        .lazy()
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
        .collect()?;

    let clean_count = clean.height();

    conn.execute("DELETE FROM clean_wines;", [])?;
    let names = clean.column("name")?.str()?;
    let variety = clean.column("variety")?.str()?;
    let region = clean.column("region")?.str()?;
    let rating = clean.column("rating")?.f64()?;
    let notes = clean.column("notes")?.str()?;

    let tx = conn.unchecked_transaction()?;
    for i in 0..clean_count {
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
    Ok((raw_count, clean_count))
}

// ---------------------------------------------------------------------------
// Gold
// ---------------------------------------------------------------------------

fn load_clean(conn: &Connection) -> Result<DataFrame> {
    let mut stmt = conn.prepare("SELECT name, variety, region, rating, notes FROM clean_wines")?;

    let mut names: Vec<String> = vec![];
    let mut variety: Vec<String> = vec![];
    let mut regions: Vec<Option<String>> = vec![];
    let mut ratings: Vec<f64> = vec![];
    let mut notes: Vec<String> = vec![];

    for row in stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, f64>(3)?,
            row.get::<_, String>(4)?,
        ))
    })? {
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

/// The gold layer as a LazyFrame: every clean wine rated at or above
/// `min_rating`. `gold` exports it; `serve` answers HTTP requests from it.
pub fn gold_lazy(conn: &Connection, min_rating: f64) -> Result<LazyFrame> {
    Ok(load_clean(conn)?
        .lazy()
        .filter(col("rating").gt_eq(lit(min_rating))))
}

pub fn gold(
    conn: &Connection,
    min_rating: f64,
    region: Option<&str>,
) -> Result<(&'static str, &'static str)> {
    let mut lf = gold_lazy(conn, min_rating)?.select([
        col("name"),
        col("variety"),
        col("region"),
        col("rating"),
    ]);

    if let Some(r) = region {
        lf = lf.filter(col("region").eq(lit(r)));
    }

    let mut filtered = lf.collect()?;

    let csv_path = "gold_wines.csv";
    let json_path = "gold_wines.json";

    CsvWriter::new(&mut std::fs::File::create(csv_path)?).finish(&mut filtered)?;

    JsonWriter::new(&mut std::fs::File::create(json_path)?)
        .with_json_format(JsonFormat::Json)
        .finish(&mut filtered)?;

    Ok((csv_path, json_path))
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

pub fn report(conn: &Connection, min_rating: f64) -> Result<()> {
    let df = load_clean(conn)?;

    let top = df
        .lazy()
        .filter(col("rating").gt_eq(lit(min_rating)))
        .group_by([col("variety")])
        .agg([
            col("rating").mean().alias("avg_rating"),
            col("rating").count().alias("count"),
        ])
        .sort(
            ["avg_rating"],
            SortMultipleOptions::default().with_order_descending(true),
        )
        .limit(10)
        .collect()?;

    println!("## Top Grape Varieties (min rating: {min_rating})\n");
    println!("| Variety | Avg Rating | Count |");
    println!("|---------|-----------|-------|");

    let variety = top.column("variety")?.str()?;
    let avg = top.column("avg_rating")?.f64()?;
    let count = top.column("count")?.u32()?;

    for i in 0..top.height() {
        println!(
            "| {} | {:.1} | {} |",
            variety.get(i).unwrap_or(""),
            avg.get(i).unwrap_or(0.0),
            count.get(i).unwrap_or(0),
        );
    }
    Ok(())
}
