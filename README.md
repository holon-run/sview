# sview

`sview` is a small Rust CLI for producing agent-friendly structure views of files.

It is not an editor, refactoring engine, or IDE protocol client. Its first job is to help coding agents understand *where the important structures are* before they decide which exact ranges to read or edit.

## Why

Coding agents often work with only two primitive file operations:

1. read text;
2. apply patch.

That is enough for small files, but it becomes expensive and fragile for large source files, Markdown documents, config files, and generated-looking artifacts. The agent has to repeatedly read broad ranges, remember where symbols or sections are, and produce large text patches without a compact structural map.

`sview` fills the gap before editing:

```text
agent -> sview -> compact structure view -> targeted reads / patches
```

## Project status

Status: **MVP implementation**.

The current crate provides a working Rust CLI with Markdown and Rust structure views. Rust parsing uses `tree-sitter-rust`; Markdown parsing is still a lightweight line-oriented outline.

## Design goals

- **Agent-facing**: optimize output for downstream agents, not for human IDE UI.
- **View first**: inspect and summarize structure; do not mutate files.
- **Fast local CLI**: start quickly, work well in shell-based agent runtimes, and produce bounded output.
- **Stable ranges**: report line ranges that can guide follow-up `sed`, editor, patch, or tool calls.
- **Broad file coverage**: support code, Markdown, configuration, scripts, and other structured text files over time.
- **Machine-readable by default**: provide a stable JSON contract, with optional text formats for humans and agents.
- **Small core**: avoid becoming a full LSP client, IDE backend, or rewrite framework in the first phase.

## Non-goals

First versions should not implement:

- rename symbol;
- move symbol / move module;
- extract function / extract module;
- organize imports;
- compiler or LSP diagnostics;
- code actions;
- automatic rewrites.

Those capabilities may later belong in a sibling tool such as `sedit`, or in a higher-level agent harness that combines `sview` with parser, LSP, compiler, or codemod backends.

## Initial CLI sketch

The exact interface is still provisional, but the tool should be shaped around simple file-oriented calls:

```bash
sview README.md
sview README.md --json
sview src/lib.rs --json
sview src/lib.rs --depth 2
sview src/lib.rs --ranges
sview src/lib.rs --format agent
```

The first stable version should probably support:

- one input file per invocation;
- automatic language detection from path and content;
- JSON output;
- optional compact text output;
- maximum depth / maximum nodes / maximum preview length controls.

## Output model

A structure view is a tree of nodes with source ranges:

```json
{
  "path": "src/lib.rs",
  "language": "rust",
  "nodes": [
    {
      "kind": "function",
      "name": "run_task",
      "start_line": 120,
      "end_line": 188,
      "preview": "fn run_task(...) -> ...",
      "children": []
    }
  ]
}
```

For Markdown, the same contract can represent headings and document regions:

```json
{
  "path": "README.md",
  "language": "markdown",
  "nodes": [
    {
      "kind": "heading",
      "level": 2,
      "name": "Installation",
      "start_line": 20,
      "end_line": 42,
      "children": []
    }
  ]
}
```

The output should be deliberately boring: stable keys, predictable ranges, and enough preview text to help an agent choose the next read or edit range.

## First implementation slice

A useful MVP can stay very small:

1. Markdown outline from headings, frontmatter, code blocks, and list regions.
2. Rust outline for modules, structs, enums, traits, impl blocks, functions, and tests.
3. JSON output with line ranges and short previews.
4. Compact text output for quick terminal use.
5. Real dogfooding inside agent tasks that currently require large-file inspection.

## Dogfooding

During development, use the local binary before broad file reads:

```bash
cargo run --quiet -- README.md --depth 2
cargo run --quiet -- src/lib.rs --depth 1 --max-nodes 40
cargo run --quiet -- src/lib.rs --json --depth 2
```

Text output is a compact tree outline:

```text
src/main.rs (rust)
├─ struct Cli L8-31 — struct Cli {
├─ enum OutputFormat L34-37 — enum OutputFormat {
└─ function main L39-54 — fn main() -> Result<()> {
```

The intended workflow is:

1. run `sview` to get stable `start_line` / `end_line` ranges;
2. read only the relevant range with a focused command such as `sed -n '120,180p'`;
3. patch or inspect the exact range, then rerun `sview` or tests if structure changed.

## Possible implementation approach

Rust is the preferred implementation language because `sview` should behave like local developer tools such as `rg`, `bat`, or `ast-grep`:

- fast startup;
- single binary distribution;
- predictable file and range handling;
- good parser ecosystem;
- easy JSON output;
- suitable for repeated agent subprocess calls.

Current implementation dependencies:

- `clap` for CLI parsing;
- `serde` / `serde_json` for output contracts;
- `tree-sitter` and `tree-sitter-rust` for Rust code outlines.

Possible future dependencies include more `tree-sitter` grammars for additional languages and `pulldown-cmark` or another Markdown parser for richer document outlines.

## Development and coverage

Run the normal verification before submitting changes:

```bash
cargo fmt -- --check
cargo test
```

Coverage is tracked with [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov):

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --workspace --all-targets --summary-only
```

Initial coverage target: keep the core library line coverage at **70% or higher** while the project is small, and raise the bar once more language parsers and fixtures land.

## Relationship to other tools

`sview` is narrower than an IDE and higher-level than raw parser output:

```text
agent -> sview -> tree-sitter / markdown parser / ast-grep / LSP / compiler
```

- LSP can be a backend later, but the agent should not need to speak LSP directly.
- `ast-grep` can be a backend for structural matching or future rewrite workflows, but `sview` should first expose an outline, not a grep interface.
- IDE MCP servers and tools like Serena are closer to full code intelligence backends; `sview` starts as a lightweight CLI surface.

## Repository layout

```text
.
├── README.md
├── Cargo.toml
├── src/
│   ├── lib.rs
│   └── main.rs
├── skills/
│   └── sview-dogfooding/
└── tests/
```
