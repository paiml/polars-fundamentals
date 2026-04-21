# Polars Fundamentals: Data Engineering with Rust

This repository contains hands-on examples and labs for using Polars with Rust for data engineering tasks. A Coursera course from Pragmatic AI Labs.

## Contents

This repository has example projects in [./examples](./examples) and hands-on labs in [./labs](./labs). Make sure you have the [Rust toolchain](https://rustup.rs) installed.

This repository is *Codespaces ready* and set as a template repository. You can open it directly in a GitHub Codespace — Rust, rust-analyzer, and all extensions are pre-installed.

[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://codespaces.new?hide_repo_select=true&ref=main)

## Labs

Complete these hands-on labs to reinforce your learning:

| Lab | Topic | Example |
|-----|-------|---------|
| [Lab 1: What is Polars and Why Use It with Rust?](./labs/lab-1.md) | Arrow memory model, Polars vs. pandas, project setup | [examples/1-polars-intro](./examples/1-polars-intro/) |
| [Lab 2: DataFrames and Series](./labs/lab-2.md) | CsvReader, schema inspection, null counts, slicing | [examples/2-dataframes-series](./examples/2-dataframes-series/) |
| [Lab 3: Expressions and the Lazy API](./labs/lab-3.md) | col, lit, LazyFrame, query plans, collect | [examples/3-lazy-api](./examples/3-lazy-api/) |
| [Lab 4: Data Cleaning](./labs/lab-4.md) | Nulls, casting, text normalization, invalid rows | [examples/4-data-cleaning](./examples/4-data-cleaning/) |
| [Lab 5: Sorting, Filtering, and Aggregation](./labs/lab-5.md) | filter, sort, group_by, aggregations | [examples/5-filtering-aggregation](./examples/5-filtering-aggregation/) |
| [Lab 6: Joining and Reshaping Data](./labs/lab-6.md) | Left joins, melt/unpivot, CSV and Parquet export | [examples/6-joins-reshape](./examples/6-joins-reshape/) |
| [Lab 7: Bronze — Ingesting Raw Data](./labs/lab-7.md) | clap CLI, CsvReader to SQLite, ingested_at timestamp | [examples/7-bronze-ingestion](./examples/7-bronze-ingestion/) |
| [Lab 8: Silver — Cleaning and Standardizing](./labs/lab-8.md) | SQLite to LazyFrame, cleaning pipeline, clean_wines | [examples/8-silver-cleaning](./examples/8-silver-cleaning/) |
| [Lab 9: Gold — Business Logic and Export](./labs/lab-9.md) | min-rating filter, top varieties, CSV and JSON export | [examples/9-gold-export](./examples/9-gold-export/) |

## Course Outline

### Module 1: Polars Foundations

#### Lesson 1.1 — What is Polars and why use it with Rust?
- [Setting up a Polars project and building a first DataFrame](./examples/1-polars-intro/)
- The Apache Arrow memory model and columnar storage
- Polars vs. pandas: performance, lazy evaluation, and type safety

#### Lesson 1.2 — DataFrames and Series
- [Reading wine-ratings.csv and inspecting the schema](./examples/2-dataframes-series/)
- Series, DataFrames, and Polars data types
- Column selection, row slicing, and null counts

#### Lesson 1.3 — Expressions and the Lazy API
- [col, lit, and chained transforms with LazyFrame](./examples/3-lazy-api/)
- Eager vs. lazy evaluation — when to use each
- Collecting a LazyFrame and reading the query plan

### Module 2: Cleaning and Transforming Wine Data

#### Lesson 2.1 — Data Cleaning
- [Handling nulls, casting ratings to f64, normalizing text](./examples/4-data-cleaning/)
- Drop-vs-fill strategies for missing values
- Filtering invalid rows and validating the cleaned schema

#### Lesson 2.2 — Sorting, Filtering, and Aggregation
- [Filter by region and rating, group_by variety](./examples/5-filtering-aggregation/)
- Multi-column sort with descending and ascending order
- Counting the most-reviewed regions

#### Lesson 2.3 — Joining and Reshaping Data
- [Left join, melt/unpivot, and writing CSV and Parquet](./examples/6-joins-reshape/)
- Adding a lookup table and enriching the wine DataFrame
- Wide-to-long transformations for reporting

### Module 3: Building the Medallion Pipeline

#### Lesson 3.1 — Bronze: Ingesting Raw Data
- [clap CLI that loads wine-ratings.csv into SQLite](./examples/7-bronze-ingestion/)
- Schema design for the bronze layer: preserving raw columns
- Adding an ingested_at timestamp without business logic

#### Lesson 3.2 — Silver: Cleaning and Standardizing
- [SQLite → LazyFrame cleaning pipeline with a printed summary](./examples/8-silver-cleaning/)
- Applying reusable cleaning functions from Module 2
- Enforcing non-null constraints and deduplication

#### Lesson 3.3 — Gold: Business Logic and Export
- [min-rating filter, top-variety aggregations, CSV and JSON export](./examples/9-gold-export/)
- Configurable thresholds with clap flags
- Exporting the gold DataFrame for downstream consumers

## Graded Project: wine-pipeline

Build **wine-pipeline** — a Rust CLI tool implementing the Bronze–Silver–Gold medallion architecture over the wine ratings dataset:

- `bronze` — read `wine-ratings.csv` and load all rows as-is into a SQLite `raw_wines` table, adding an `ingested_at` timestamp
- `silver` — read `raw_wines`, apply cleaning rules (drop nulls, normalize text, cast rating to `f64`), and write a validated `clean_wines` table with a printed summary of changes
- `gold` — read `clean_wines`, filter by `--min-rating` (default 90), compute top grape varieties by average rating, and export results to `gold_wines.csv` and `gold_wines.json`
- `report` — print a Markdown summary table of gold-layer aggregates to stdout

A starter implementation is in [wine-pipeline/](./wine-pipeline/).

```bash
# Build
cargo build -p wine-pipeline

# Ingest raw CSV
cargo run -p wine-pipeline -- bronze --input wine-ratings.csv

# Clean and standardize
cargo run -p wine-pipeline -- silver

# Export gold results (wines rated 92+)
cargo run -p wine-pipeline -- gold --min-rating 92

# Print a Markdown report
cargo run -p wine-pipeline -- report
```

## Local Setup

1. Install the Rust toolchain:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone this repository:
   ```bash
   git clone https://github.com/alfredodeza/polars-fundamentals.git
   cd polars-fundamentals
   ```

3. Build the entire workspace:
   ```bash
   cargo build --workspace
   ```

4. Run an example:
   ```bash
   cargo run -p polars-intro
   ```

5. Run tests:
   ```bash
   cargo test --workspace
   ```

## Key Crates

| Crate | Purpose |
|---|---|
| [polars](https://crates.io/crates/polars) | DataFrame engine with lazy evaluation and Arrow backend |
| [rusqlite](https://crates.io/crates/rusqlite) | SQLite bindings for the bronze/silver/gold persistence layer |
| [clap](https://crates.io/crates/clap) | CLI argument parsing with derive API |
| [serde_json](https://crates.io/crates/serde_json) | JSON serialization for gold-layer export |
| [chrono](https://crates.io/crates/chrono) | Timestamps for the bronze ingestion layer |
| [anyhow](https://crates.io/crates/anyhow) | Ergonomic error handling |

## Resources

- [Polars Rust documentation](https://docs.rs/polars)
- [Polars user guide](https://pola.rs)
- [The Rust Book](https://doc.rust-lang.org/book/)
- [clap documentation](https://docs.rs/clap)

**Coursera Courses**

- [Rust for Data Engineering Specialization](https://www.coursera.org/specializations/rust-for-data-engineering)
- [MLOps Machine Learning Operations Specialization](https://www.coursera.org/specializations/mlops-machine-learning-duke)
- [Linux and Bash for Data Engineering](https://www.coursera.org/learn/linux-and-bash-for-data-engineering-duke)
