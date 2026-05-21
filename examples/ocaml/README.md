# OCaml example

Dune project with a library + bin + alcotest test suite. Pipeline initializes opam with OCaml 5.1.1, then runs `dune build / runtest / build @fmt`.

## Run the pipeline

```sh
cd examples/ocaml
hm run ci --local
```

See `.harmont/pipeline.py` for the definition; `examples/README.md` for the full index.
