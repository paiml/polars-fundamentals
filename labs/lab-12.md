# Lab 12: Aggregations as Endpoints

**Reading:** [4.3 GET /regions and GET /varieties](../docs/module-4/4.3-regions-and-varieties-with-group-by.md)

```bash
curl -s http://127.0.0.1:3000/regions   | head -c 400; echo
curl -s http://127.0.0.1:3000/varieties
```

## Tasks
1. Call `/regions` twice and compare the two responses byte for byte (`diff <(curl …) <(curl …)`).
   Which line in `serve.rs` guarantees they match?
2. Find the `""` key in `/varieties`. Trace it back: which silver-layer rule lets an empty
   variety through? Add a rule to `silver` that drops or labels it, re-run `silver`, and
   restart `serve`.
3. Add `GET /regions/top?n=5`, which returns the `n` regions with the highest mean rating, using
   `group_by`, `mean`, `sort` and `limit`. Add a test that uses the fixture.
