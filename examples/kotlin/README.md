# Kotlin example

Gradle + kotlin.test starter, JDK 21. Pipeline uses `hm.gradle(kotlin=True)` to label steps `:kotlin:` and runs `./gradlew build / test / check`.

## Bootstrap the Gradle wrapper

```sh
cd examples/kotlin
gradle wrapper --gradle-version 8.10
```

## Run the pipeline

```sh
hm run ci --local
```

See `.harmont/pipeline.py` for the definition; `examples/README.md` for the full index.
