# Agent guide — tool-transform

`hnl-tool-transform` — the abstraction crate for the AST transform
operation: the trait/type surface for AST transforms (passes,
pipelines, plans) that concrete transform engines
(`tool-transform-rewrite`) implement. `publish = false` (git-install
only). Rust 2024.

## Layout

| path | role |
| --- | --- |
| `src/lib.rs` | crate docs + public API |
| `src/request.rs`, `src/condition.rs`, `src/validate.rs` | transform request model, conditions, validation |
| `src/source.rs`, `src/output.rs`, `src/native.rs` | source/output adapters + native-transform glue |
| `docs/` | operation doc |
| `xtask/`, `workflows/gate.yaml`, `bupha.toml`, `.cargo/` | vendored repo gate + CI wiring |

## Contract

- Deps resolve from git — `handl-language` → `handl-lang/kernel`,
  `handl-plugin-ext-diag` → `handl-lang/host`, `branch = "main"`.
- Abstraction only: pass/pipeline semantics live in the engine crate —
  this repo owns the traits and the request/condition vocabulary.

## Verification

```bash
cargo xtask gate   # fmt --check → clippy -D warnings → test → doc
```


<!-- myco:agent-awareness -->

## Agent awareness

- Session start / any work discussion: run `myco status --prompt`
  before proposing next steps — know the board first.
- User describes new work → offer to capture it (`myco todo add`,
  `issue new`, `proposal new`). Don't wait to be asked.
- Picking up a task: `myco resume <ID>` is the briefing.
- Wrapping up: `myco handoff <ID>` anything resumable; keep statuses
  honest (claimed → in-progress → blocked/done).

<!-- /myco:agent-awareness -->
