# BVT Diff Generator

Product one-liner: Codex Live Run is a terminal-first system surface: one high-level instruction routes through the existing BVT engine binary, emits receipts, and renders the run without product-card theatre.

Blank page reset: the live site is now a terminal-first Codex run page inspired by the wterm/Ghostty example, with artifacts and metrics instead of CTA panels.

Live site: https://site-mk0rzu4uw-jasedgws-projects.vercel.app

Realtime DX surface: `site/dx.html` shows the production wedge: DSL packet -> BVT engine -> CI state artifact -> terminal/receipt viewer. It is wterm-ready, but still uses a zero-dependency terminal replay until the package is integrated.

Routing note: the site root now redirects to `dx.html`, and `site/dx/index.html` makes local clean-url testing work without depending on Vercel routing.

Current release benchmark: 320,000 typed ops -> 10,000 unified patch artifacts in roughly 0.22-0.24s, or about 41k-45k patches/sec at ~776 bytes/patch.

UI factory claim boundary: this throughput can create many candidate UI diffs quickly, but admitted UI quality still requires render checks, design constraints, rollback, and BVT receipts.

Killer feature: Distillation Gate compresses many generated UI diffs into one selected transition: promoted diff, evidence packet, rollback packet, and receipt.

Real run: `runs/e2e-ui-run` shows `compile-dsl -> generate -> apply` turning one UI intent packet into `ui-run.diff`, receipts, and a generated `workspace/index.html` page.

DOM actuator: the live site now shows typed browser operations (`setText`, `setAttr`, `addClass`, `appendTile`) mutating real DOM nodes with a rollback snapshot and `MutationObserver` evidence.

Live-site copy demo: the site shows an authorized structure-copy workflow where typed DOM ops recreate a landing page variant and measure browser-side DOM ops/sec live.

Indie hacker wedge: use high-throughput diff generation to create many cheap market experiments, while BVT keeps receipts, rollback, and honest claim boundaries attached.

Agent/tool-call framing: the indie business generator is a workload. Agents choose transitions; tools execute `research_offer`, `compile_dsl`, `generate_diff`, `apply_dom`, `deploy_site`, and `admit_result` with receipts.

No-rebuild path: content transitions should call the existing binary once, e.g. `target/debug/bvt-diff-generator run-dsl packet.json --run-dir runs/name`, and rebuild only when Rust engine code changes.

UI correction: the live no-rebuild demo now shows receipt metrics and browser-measured DOM ops/sec instead of a fake CTA panel.

High-level surface: the live site now includes a chat-style renderer that switches system views from in-browser state instead of writing files or rebuilding per question.

DOM benchmark proof: the live page now shows counted DOM calls, elapsed ms, loop count, and the ops/sec formula so the throughput number is auditable.

Git/site lineage: the live page renders the mainline commit tree as an ICR-style graph from `1301e67` core engine through `f3e83bd` agent tool runtime, with Vercel preview leaves for each product surface.

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
bvt-diff-generator run-dsl examples/dsl-plan.json --run-dir runs/example-run
bvt-diff-generator bench --iterations 10000 --ops 32
```

## CI State Saving

GitHub Actions now owns the repetitive loop. On push or PR, `.github/workflows/ci-state.yml` builds the engine once, runs tests, checks site JavaScript, records a benchmark receipt, and uploads a `bvt-ci-state-<sha>` artifact containing `ci-state/` plus the rendered `site/`.

For a high-level transition, run the workflow manually with a DSL packet path. CI will call:

```bash
target/release/bvt-diff-generator run-dsl <packet> --run-dir ci-state/transition-run
```

That saves `plan.json`, `diff.patch`, generate/apply receipts, and a run receipt as CI state. If `VERCEL_TOKEN` is configured in repository secrets, pushes to `main` deploy the site from CI instead of requiring a local deploy command.

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
