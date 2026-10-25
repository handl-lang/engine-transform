# Explanation

## Why this crate exists

Desugaring, lowering and rewrite passes all want the same thing: walk an arena
bottom-up, remap changed nodes, hand back a new root. Extracting that shape once
(ADR-0094) turned the legacy `desugar_pass` into a reusable engine and made
passes registerable data.

## Where it sits

This crate belongs to the `engine-*` family. Plugins, kits and hosts reach it
through the kit's operation slot, so the engine can be swapped without touching a
language definition.

See the [reference](/engine-transform/reference/) for the API and the
[how-to guides](/engine-transform/how-to/) for tasks.
