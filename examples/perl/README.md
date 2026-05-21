# Perl example

Perl module with Test::More + Perl::Critic. Pipeline installs deps via cpanm and runs `prove -lv t/` + `perlcritic lib/`.

## Run the pipeline

```sh
cd examples/perl
hm run ci --local
```

See `.harmont/pipeline.py` for the definition; `examples/README.md` for the full index.
