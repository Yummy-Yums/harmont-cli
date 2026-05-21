# Java example

Gradle + JUnit 5 starter, JDK 21. Pipeline runs `./gradlew build / test / check`.

## Bootstrap the Gradle wrapper

Run once to check in `gradlew`, `gradlew.bat`, and `gradle/wrapper/`:

```sh
cd examples/java
gradle wrapper --gradle-version 8.10
```

## Run the pipeline

```sh
hm run ci --local
```

See `.harmont/pipeline.py` for the definition; `examples/README.md` for the full index.
