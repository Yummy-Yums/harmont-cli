# C# (.NET) example

Solution with one classlib + xunit test project, .NET 8. Pipeline runs `dotnet build / test / format --verify-no-changes`.

## Run the pipeline

```sh
cd examples/csharp
hm run ci --local
```

See `.harmont/pipeline.py` for the definition; `examples/README.md` for the full index.
