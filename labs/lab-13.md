# Lab 13: Search and Region Routes

**Reading:** [4.4 /wines/search and /wines/region/{region}](../docs/module-4/4.4-search-and-region-routes.md)

```bash
curl -s 'http://127.0.0.1:3000/wines/search?q=bourbon&limit=3'
curl -s 'http://127.0.0.1:3000/wines/region/Napa%20Valley%2C%20California?limit=3'
curl -s -o /dev/null -w '%{http_code}\n' 'http://127.0.0.1:3000/wines/search'   # expect 400
curl -s 'http://127.0.0.1:3000/wines/region/Atlantis'                          # expect []
```

## Tasks
1. Explain why `/wines/region/Mendoza` returns `[]` while `/wines?region=mendoza` does not.
2. Search for `bourbon`. Some results don't have "bourbon" in the name. Where did the match come from?
3. Run the whole suite (`cargo test -p wine-pipeline`) and make sure every test passes before you
   start the graded project.
