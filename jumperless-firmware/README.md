# jumperless-firmware

Experiment for building a jumperless firmware using [Embassy](https://embassy.dev/).

Not ready for anything. Do not use.

## How to run

1. Adjust the `runner = ...` line in `.cargo/config.toml`, if you don't have a debug probe
2. Run (i.e. build & upload) the application:
   ```
   cargo run --release
   ```

Running with a debug probe is recommended, since this is currently the only way to get full debug logging.
