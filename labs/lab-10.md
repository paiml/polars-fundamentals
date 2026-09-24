# Lab 10: Serve the Gold Layer

**Reading:** [4.1 An Axum Server over the Gold Layer](../docs/module-4/4.1-an-axum-server-over-the-gold-layer.md)

## Goal
Start `wine-pipeline serve` against a real gold layer and see its startup contract hold.

## Steps
1. Download the dataset into the repository root:
   ```bash
   curl -L -o wine-ratings.csv https://raw.githubusercontent.com/paiml/wine-ratings/main/wine-ratings.csv
   ```
2. Build the bronze and silver layers:
   ```bash
   cargo run -p wine-pipeline -- bronze --input wine-ratings.csv
   cargo run -p wine-pipeline -- silver
   ```
3. Start the server: `cargo run -p wine-pipeline -- serve --min-rating 90 --port 3000`
4. Record the two lines it prints: the `contract: gold floor … OK` line and the number of gold wines served.
5. Stop the server. Delete `wine.db`, run `serve` again, and read the error. Why is refusing to start better than serving `[]`?

## Check yourself
- Restart with `--min-rating 95`. Does the served count go down? By roughly how much?
- `--bind` defaults to `127.0.0.1`. What changes if you pass `--bind 0.0.0.0`, and when would you want that?
