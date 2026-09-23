# Graded Project: `wine-pipeline serve`

Extend your `wine-pipeline` from Module 3 with a `serve` subcommand that answers
HTTP requests from the gold layer. The reference implementation is
[`wine-pipeline/src/serve.rs`](../../wine-pipeline/src/serve.rs). Build your own
version before you read it.

## Requirements

1. `wine-pipeline serve --min-rating <f64> --bind <ip> --port <u16>`: defaults `90`, `127.0.0.1`, `3000`.
2. The server builds the gold layer with the **same** function the `gold` subcommand uses.
   No second copy of the filter.
3. Before binding, it asserts the **gold floor** (every rating ≥ `--min-rating`) and prints a
   `contract: … OK` line. On an empty gold layer it exits non-zero and does not bind.
4. Five routes, all JSON:

| Route | Behaviour |
|---|---|
| `GET /wines` | optional `region`, `variety` (case-insensitive substring), `min_rating`, `max_rating` (inclusive), `limit` (default 100, max 1000); sorted by rating descending |
| `GET /regions` | `{ region: count }`, wines with no region excluded, stable key order |
| `GET /varieties` | `{ variety: { count, avg_rating } }` |
| `GET /wines/search?q=` | case-insensitive substring of name OR notes; missing `q` → 400 |
| `GET /wines/region/{region}` | exact region match; unknown region → `200 []` |

5. No query parameter is ever formatted into code or SQL. Every value reaches Polars through `lit()`.
6. `cargo test -p wine-pipeline` passes with at least one test per route, including one
   well-formed request that must return an empty list.

## How it is graded

The grader runs these commands against your repository. Every line must behave as stated.

```bash
curl -L -o wine-ratings.csv https://raw.githubusercontent.com/paiml/wine-ratings/main/wine-ratings.csv
cargo run -p wine-pipeline -- bronze --input wine-ratings.csv
cargo run -p wine-pipeline -- silver
cargo test -p wine-pipeline                                  # exit 0
cargo run -p wine-pipeline -- serve --port 3000 &            # prints "contract: gold floor ... OK"

curl -s 'localhost:3000/wines?limit=1' | jq '.[0].rating'                     # 99.0
curl -s 'localhost:3000/wines?min_rating=99.5'                                # []
curl -s 'localhost:3000/wines?region=napa&max_rating=91&limit=1000' \
  | jq 'all(.[]; .rating <= 91 and (.region | test("napa"; "i")))'            # true
curl -s 'localhost:3000/regions' | jq 'has("Napa Valley, California")'        # true
curl -s 'localhost:3000/varieties' | jq '."RED WINE".count > 0'               # true
curl -s -o /dev/null -w '%{http_code}' 'localhost:3000/wines/search'          # 400
curl -s 'localhost:3000/wines/search?q=bourbon&limit=1000' | jq 'length > 0'  # true
curl -s 'localhost:3000/wines/region/Atlantis'                                # []
curl -s "localhost:3000/wines?region=x'%20OR%201=1"                           # []
```

## Optional: containerise it

If you took the Docker course, package `serve` in a multi-stage Dockerfile that
builds a release binary and runs it with `--bind 0.0.0.0`. The same curl checks must pass
against the container.
