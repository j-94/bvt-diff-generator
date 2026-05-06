# AGENTS

Scope: repository-wide.

Product one-liner: BVT Diff Generator is a high-throughput delta compiler: typed edit ops become candidate diffs, and receipts preserve what happened.

## Working Contract

- Treat the system as `S + Delta + C -> S'`.
- Keep proposal and admission separate.
- Prefer typed edit operations over free-form patch text.
- Preserve receipts for generated, checked, applied, and benchmarked transitions.
- Do not claim admitted code quality from raw ops/sec. Ops/sec is only proposal throughput until tests and BVT gates pass.

## First Surface

```bash
cargo test
cargo run -- generate examples/plan.json
cargo run -- check examples/plan.json
cargo run -- bench --iterations 10000 --ops 32
```
