# engine-transform

The AST-transform contract: `TransformEngine`, `TransformMode`, arena helpers.

## What it is

The **abstraction crate** for the AST transform operation (ADR-0094):
`TransformEngine`, `TransformMode`, and the arena navigation helpers (`node_ref`,
`child_ref`, `descendant_ref`, `child_is_kind`, `contains`) that implementations
share. The rule-interpreting implementation is `engine-transform-rewrite`.

## Start here

- **Reference** — [public API](/engine-transform/reference/)
- **How-to** — [task recipes](/engine-transform/how-to/)
- **Explanation** — [design and rationale](/engine-transform/explanation/)

No tutorials: this repo is a building block used from Rust. End-to-end
walkthroughs live with the product that composes it — see the
[Handl platform](/handl/).
