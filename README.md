# engine-transform

`hnl-engine-transform` — The **abstraction** crate for the AST transform operation (ADR-0094).

## Development

```bash
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

`ci.sh` is the gate (fmt → clippy → test → doc). Deps resolve from git, so no
sibling checkouts are needed; a plain clone test-runs as-is.

## License

MIT.

## Documentation

Published site (Diátaxis): <https://handl-lang.github.io/docs/engine-transform/>

- [Reference](https://handl-lang.github.io/docs/engine-transform/reference/) — public API surface
- [How-to](https://handl-lang.github.io/docs/engine-transform/how-to/) — task recipes
- [Explanation](https://handl-lang.github.io/docs/engine-transform/explanation/) — design and rationale

Decision records live on the `project` branch under `project/adrs/`.
