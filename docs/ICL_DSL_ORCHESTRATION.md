# ICL + DSL Orchestration

Product one-liner: ICL chooses the transition language; the DSL lowers intent into fast, receipt-backed diff operations.

## Why This Matters

Raw diff text wastes model capacity on syntax, line counting, and patch formatting. The better split is:

| Layer | Optimizes | Owner |
| --- | --- | --- |
| ICL packet | examples, taste, constraints, claim boundary | LM |
| DSL plan | semantic edit intent | LM + deterministic schema |
| typed ops | line-safe local mutations | compiler |
| diff bundle | patch artifact | engine |
| receipt/gate | admission evidence | BVT |

## Control Loop

```text
S: files, tests, requirements, memory, claim boundary
Delta: high-level DSL actions proposed by the LM
C: path safety, exact match anchors, tests, rollback, BVT rules
S': generated candidate diff, or rejected transition
Receipt: operation count, file count, elapsed time, admission surface
```

## LM Prompt Contract

Ask the LM for DSL, not patches:

```text
Given the repo state and examples, emit a minimal JSON DSL plan.
Use replace_line_containing, insert_after_containing, append_file, or ensure_file.
Do not emit raw diffs. Do not claim success. The engine will compile and gate.
```

## Pareto Frontier

| Path | Speed | Quality Risk | Best Use |
| --- | ---: | ---: | --- |
| LM writes raw diff | low | high | one-off edits |
| LM emits typed ops | medium | medium | known line ranges |
| LM emits DSL | high | lower | repeated orchestration |
| DSL plus BVT receipts | high | lowest | durable product loop |

The point is not to remove the LM. It is to move the LM up the stack where its semantic bandwidth matters, then let the local engine spend the 4k ops/sec budget.
