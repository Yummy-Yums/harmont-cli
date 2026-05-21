# Haskell example

Single cabal package with an inline test-suite. Pipeline pins GHC 9.6.7 via ghcup and runs build + test + lint (--flag werror) + fmt (fourmolu check).

## Run the pipeline

```sh
cd examples/haskell
hm run ci --local
```

See `.harmont/pipeline.py` for the definition; `examples/README.md` for the full index.
