// wine-pipeline — Graded Project
//
// A CLI tool implementing the Bronze–Silver–Gold medallion architecture over
// the wine ratings dataset:
//   bronze  — read wine-ratings.csv and load raw rows into SQLite raw_wines
//   silver  — clean raw_wines and write a validated clean_wines table
//   gold    — filter by --min-rating, aggregate, and export CSV + JSON
//   serve   — answer HTTP requests from the gold LazyFrame (Module 4)

mod pipeline;
mod serve;

use clap::{Parser, Subcommand};
use polars::prelude::ChunkAgg;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "wine-pipeline",
    about = "Medallion data pipeline for wine ratings",
    version
)]
struct Cli {
    /// Path to the SQLite database file
    #[arg(long, env = "WINE_DB", default_value = "wine.db")]
    db: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest raw wine-ratings.csv into the bronze layer
    Bronze {
        /// Path to wine-ratings.csv
        #[arg(long, default_value = "wine-ratings.csv")]
        input: PathBuf,
    },
    /// Clean bronze data and write the silver layer
    Silver,
    /// Apply business logic to silver and export gold results
    Gold {
        /// Minimum rating threshold (inclusive)
        #[arg(long, default_value_t = 90.0)]
        min_rating: f64,

        /// Scope results to a single region
        #[arg(long)]
        region: Option<String>,
    },
    /// Print a Markdown summary table of gold-layer aggregates to stdout
    Report {
        /// Minimum rating threshold used when producing the report
        #[arg(long, default_value_t = 90.0)]
        min_rating: f64,
    },
    /// Serve the gold layer over HTTP (Module 4)
    Serve {
        /// Minimum rating that defines the gold layer (inclusive)
        #[arg(long, default_value_t = 90.0)]
        min_rating: f64,

        /// Address to bind
        #[arg(long, default_value = "127.0.0.1")]
        bind: std::net::IpAddr,

        /// Port to listen on
        #[arg(long, default_value_t = 3000)]
        port: u16,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let conn = pipeline::open_db(&cli.db)?;
    pipeline::init_schema(&conn)?;

    match cli.command {
        Commands::Bronze { input } => {
            let n = pipeline::bronze(&conn, &input)?;
            println!("Bronze: ingested {n} rows from {}", input.display());
        }
        Commands::Silver => {
            let (total, kept) = pipeline::silver(&conn)?;
            println!(
                "Silver: {kept} clean rows written ({} dropped)",
                total - kept
            );
        }
        Commands::Gold { min_rating, region } => {
            let (csv, json) = pipeline::gold(&conn, min_rating, region.as_deref())?;
            println!("Gold: exported {csv} and {json}");
        }
        Commands::Report { min_rating } => {
            pipeline::report(&conn, min_rating)?;
        }
        Commands::Serve {
            min_rating,
            bind,
            port,
        } => {
            let gold = pipeline::gold_lazy(&conn, min_rating)?.collect()?;
            if gold.height() == 0 {
                anyhow::bail!(
                    "gold layer is empty at --min-rating {min_rating}; run bronze and silver first"
                );
            }
            // Provable contract: gold floor -- every wine the API can return is
            // rated at or above the threshold that defined the gold layer.
            let lowest = gold.column("rating")?.f64()?.min().unwrap_or(f64::NAN);
            assert!(
                lowest >= min_rating,
                "gold floor violated: {lowest} < {min_rating}"
            );
            println!("contract: gold floor (min rating {lowest} >= {min_rating}) OK");

            let addr = std::net::SocketAddr::new(bind, port);
            tokio::runtime::Runtime::new()?.block_on(serve::run(gold, addr))?;
        }
    }

    Ok(())
}
