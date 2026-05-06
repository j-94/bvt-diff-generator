# BVT Diff Generator

Product one-liner: BVT Diff Generator compiles typed edit operations into fast candidate diffs, then preserves the transition as a receipt.

Live site: https://site-pxiew51dc-jasedgws-projects.vercel.app

Current release benchmark: 320,000 typed ops -> 10,000 unified patch artifacts in roughly 0.22-0.24s, or about 41k-45k patches/sec at ~776 bytes/patch.

This repo is a small Rust CLI/library for turning structured edit ops into unified diffs:

```text
S + ops(Delta) + C -> candidate diff + receipt
```

The first version keeps the optimizer/judge boundary clean:

| Layer | Job |
| --- | --- |
| `EditOp` | typed insert, replace, delete, and write-file operations |
| `build_bundle` | compile ops into per-file before/after patches |
| `render_unified_diff` | emit a standard unified diff |
| `check` | validate ranges and path safety without writing |
| `apply` | write admitted candidate output and optional receipt |
| `bench` | measure local op-to-diff throughput |
| `compile-dsl` | lower high-level intent actions into typed edit ops |

## Install

```bash
cargo build --release
```

## Plan Format

```json
{
  "intent": "tighten a config default",
  "base_dir": "/path/to/workspace",
  "ops": [
    {
      "kind": "replace",
      "path": "example.txt",
      "start": 2,
      "end": 2,
      "lines": ["new line"]
    }
  ]
}
```

Line numbers are 1-based. Paths must be relative and may not contain `..`.

## Commands

```bash
bvt-diff-generator generate examples/plan.json
bvt-diff-generator generate examples/plan.json --out runs/example.diff --receipt runs/receipt.json
bvt-diff-generator check examples/plan.json
bvt-diff-generator apply examples/plan.json --receipt runs/apply-receipt.json
bvt-diff-generator compile-dsl examples/dsl-plan.json --out runs/compiled-plan.json
bvt-diff-generator bench --iterations 10000 --ops 32
```

## ICL + DSL Orchestration

The LM should spend its intelligence on choosing the transition, not manually counting line numbers.

```text
user intent
-> ICL examples teach the LM the house style and claim boundary
-> LM emits small DSL actions
-> deterministic compiler lowers DSL to typed ops
-> diff engine generates patch + receipt
-> tests/BVT gate decide admission
```

That makes the LM operate at full capacity: semantic planning, abstraction, and tradeoff choice. The local engine handles high-rate mechanical edits.

DSL example:

```json
{
  "intent": "make the example text more explicit",
  "base_dir": "/Users/jobs/Desktop/bvt-diff-generator/examples/workspace",
  "actions": [
    {
      "kind": "replace_line_containing",
      "path": "example.txt",
      "needle": "beta",
      "line": "BETA"
    }
  ]
}
```

## Claim Boundary

The engine measures structured operation throughput, not LLM reasoning quality. High ops/sec becomes useful only when clean apply rate, tests, rollback, and BVT admission stay attached.
