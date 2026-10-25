# How-to guides

Task recipes. Each assumes sibling checkouts of this repo and of the platform
repos it patches.

## Write a transform pass

Implement `TransformEngine`, or reuse `engine-transform-rewrite`'s rule loop; return `declined` when nothing matched so the caller can skip re-running.

## Replace the AST safely

Use `install_transform_overlay`: it keeps the old arena alive for nodes the pass did not rewrite.
