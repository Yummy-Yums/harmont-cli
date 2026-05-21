# PHP / Laravel example

Minimal slice with one Model + PHPUnit unit test + PHPStan config. Pipeline uses `hm.composer(laravel=True)` so `.test()` calls `php artisan test`. Run `composer install` once locally to generate `composer.lock` (commit it; vendor/ is gitignored).

## Run the pipeline

```sh
cd examples/php-laravel
hm run ci --local
```

See `.harmont/pipeline.py` for the definition; `examples/README.md` for the full index.
