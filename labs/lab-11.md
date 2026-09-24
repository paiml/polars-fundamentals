# Lab 11: Filter Wines over HTTP

**Reading:** [4.2 GET /wines](../docs/module-4/4.2-get-wines-filters-as-expressions.md)

With the server from Lab 10 running:

```bash
curl -s 'http://127.0.0.1:3000/wines?limit=3'
curl -s 'http://127.0.0.1:3000/wines?region=napa&min_rating=97&limit=5'
curl -s 'http://127.0.0.1:3000/wines?min_rating=99.5'        # expect []
curl -s "http://127.0.0.1:3000/wines?region=x'%20OR%201=1"  # expect []
```

## Tasks
1. Confirm the first response is sorted by rating, highest first.
2. Run `cargo test -p wine-pipeline wines_rating_bounds`. Then delete the `max_rating`
   branch in `wine_filter` and run it again. Read the failure message, then restore the branch.
3. Add a `notes` query parameter that does a case-insensitive substring match on tasting
   notes, and add a test for it in the same style as the existing ones.

## Check yourself
- Why does `wine_filter` start from `lit(true)`?
- What does `contains_literal` do with a `.` in the query that a regex match would not?
