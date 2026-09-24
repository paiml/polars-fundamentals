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
| Lab 1: What is Polars and Why Use It with Rust? | Arrow memory model, Polars vs. pandas, project setup | [examples/1-polars-intro](./examples/1-polars-intro/) |
| Lab 2: DataFrames and Series | CsvReader, schema inspection, null counts, slicing | [examples/2-dataframes-series](./examples/2-dataframes-series/) |
| Lab 3: Expressions and the Lazy API | col, lit, LazyFrame, query plans, collect | [examples/3-lazy-api](./examples/3-lazy-api/) |
| Lab 4: Data Cleaning | Nulls, casting, text normalization, invalid rows | [examples/4-data-cleaning](./examples/4-data-cleaning/) |
| Lab 5: Sorting, Filtering, and Aggregation | filter, sort, group_by, aggregations | [examples/5-filtering-aggregation](./examples/5-filtering-aggregation/) |
| Lab 6: Joining and Reshaping Data | Left joins, melt/unpivot, CSV and Parquet export | [examples/6-joins-reshape](./examples/6-joins-reshape/) |
| Lab 7: Bronze — Ingesting Raw Data | clap CLI, CsvReader to SQLite, ingested_at timestamp | [examples/7-bronze-ingestion](./examples/7-bronze-ingestion/) |
| Lab 8: Silver — Cleaning and Standardizing | SQLite to LazyFrame, cleaning pipeline, clean_wines | [examples/8-silver-cleaning](./examples/8-silver-cleaning/) |
| Lab 9: Gold — Business Logic and Export | min-rating filter, top varieties, CSV and JSON export | [examples/9-gold-export](./examples/9-gold-export/) |
| [Lab 10: Serve the Gold Layer](./labs/lab-10.md) | Axum server over the gold LazyFrame, startup contract | [wine-pipeline](./wine-pipeline/) |
| [Lab 11: Filter Wines over HTTP](./labs/lab-11.md) | GET /wines, filter()/col() from query parameters | [wine-pipeline](./wine-pipeline/) |
| [Lab 12: Aggregations as Endpoints](./labs/lab-12.md) | GET /regions, GET /varieties with group_by | [wine-pipeline](./wine-pipeline/) |
| [Lab 13: Search and Region Routes](./labs/lab-13.md) | /wines/search, /wines/region/{region} | [wine-pipeline](./wine-pipeline/) |

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

### Module 4: Serving the Gold Layer

#### Lesson 4.1 — [An Axum server over the gold layer](./docs/module-4/4.1-an-axum-server-over-the-gold-layer.md)
- Reusing the gold LazyFrame from Module 3 instead of re-implementing it
- A startup contract: the gold floor, and refusing to serve an empty layer

#### Lesson 4.2 — [GET /wines: query parameters as Polars expressions](./docs/module-4/4.2-get-wines-filters-as-expressions.md)
- region / variety / min_rating / max_rating mapped onto filter() and col()
- Why a parameter must reach Polars through lit(), never through a string of code

#### Lesson 4.3 — [GET /regions and GET /varieties](./docs/module-4/4.3-regions-and-varieties-with-group-by.md)
- group_by as an endpoint, with stable output order

#### Lesson 4.4 — [/wines/search and /wines/region/{region}](./docs/module-4/4.4-search-and-region-routes.md)
- Substring search versus exact path matching

## Graded Project: wine-pipeline

Build **wine-pipeline** — a Rust CLI tool implementing the Bronze–Silver–Gold medallion architecture over the wine ratings dataset:

- `bronze` — read `wine-ratings.csv` and load all rows as-is into a SQLite `raw_wines` table, adding an `ingested_at` timestamp
- `silver` — read `raw_wines`, apply cleaning rules (drop nulls, normalize text, cast rating to `f64`), and write a validated `clean_wines` table with a printed summary of changes
- `gold` — read `clean_wines`, filter by `--min-rating` (default 90), compute top grape varieties by average rating, and export results to `gold_wines.csv` and `gold_wines.json`
- `report` — print a Markdown summary table of gold-layer aggregates to stdout
- `serve` — answer HTTP requests from the gold layer (Module 4; see [the project spec](./docs/module-4/project-serve.md))

A starter implementation is in [wine-pipeline/](./wine-pipeline/).

```bash
# Build
cargo build -p wine-pipeline

# Download the dataset (13.5 MB, not stored in this repository)
curl -L -o wine-ratings.csv https://raw.githubusercontent.com/paiml/wine-ratings/main/wine-ratings.csv

# Ingest raw CSV
cargo run -p wine-pipeline -- bronze --input wine-ratings.csv

# Clean and standardize
cargo run -p wine-pipeline -- silver

# Export gold results (wines rated 92+)
cargo run -p wine-pipeline -- gold --min-rating 92

# Print a Markdown report
cargo run -p wine-pipeline -- report

# Serve the gold layer over HTTP
cargo run -p wine-pipeline -- serve --min-rating 90 --port 3000
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
