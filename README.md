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

Java/Android source output uses the same contract for packages, imports,
classes, interfaces, enums, fields, constructors, and methods:

```text
app/src/main/java/com/example/MainActivity.java (java)
├─ package com.example L1-1 — package com.example;
├─ import android.app.Activity L3-3 — import android.app.Activity;
└─ class MainActivity L5-16 — public class MainActivity extends Activity {
   ├─ field title L6-6 — private String title;
   ├─ constructor MainActivity L8-8 — public MainActivity() {}
   └─ method onCreate L11-11 — protected void onCreate(Bundle state) {}
```

## Project status

Status: **0.1.x released CLI**.

The current crate provides a working Rust CLI with Markdown, Rust, Java, JavaScript, and TypeScript structure views. Rust, Java, and JS/TS parsing use tree-sitter grammars; Markdown parsing is still a lightweight line-oriented outline.

## Installation

Install the latest release with Homebrew on macOS:

```bash
brew tap holon-run/tap
brew install sview
```

Or download a prebuilt binary from [GitHub Releases](https://github.com/holon-run/sview/releases/latest):

```bash
curl -L https://github.com/holon-run/sview/releases/latest/download/sview-linux-amd64.tar.gz | tar -xz
chmod +x sview
./sview --help
```

Use `sview-darwin-amd64.tar.gz` or `sview-darwin-arm64.tar.gz` on macOS.

You can also install the released CLI from crates.io:

```bash
cargo install sview
```

Or install from the repository checkout:

```bash
cargo install --path .
```

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

## CLI

The CLI is shaped around simple file-oriented calls:

```bash
sview README.md
sview README.md src/lib.rs
sview README.md --json
sview README.md src/lib.rs --json
sview src/lib.rs --json
sview src/lib.rs --depth 2
sview src/lib.rs --format text
```

The 0.1.x CLI supports:

- one or more input files per invocation;
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
5. Real agent-assisted navigation inside tasks that currently require large-file inspection.

## Agent navigation

Use `sview` as a navigation tool when structure can reduce uncertainty before
reading or editing. Agents should use it when a compact structure map is likely
to save broad reads, but should not force it into small files, direct text
lookups, or already-known edits.

Good triggers:

- an unfamiliar file may need to be read mostly end-to-end, especially if it is
  large enough that a structural map can avoid broad reads;
- the target symbol, section, test, or implementation area is only approximate
  and text search does not identify a tight range;
- several candidate Rust, Java, JavaScript, TypeScript, or Markdown files or symbols
  need quick triage before choosing exact ranges;
- a patch changes parser-visible structure and the resulting outline should be
  checked.

Skip `sview` when the exact small range is already known, when `rg` directly
answers the question, when the file is small enough for one focused read, when
only one or two obvious candidate files are involved, or when the file type is
unsupported and an outline would not guide a better next read.

Typical commands:

```bash
sview README.md --depth 2
sview README.md src/lib.rs --depth 1
sview src/lib.rs --depth 1 --max-nodes 40
sview app/src/main/java/com/example/MainActivity.java --depth 2
sview tests/fixtures/typescript_sample.ts --depth 2
sview tests/fixtures/javascript_sample.js tests/fixtures/tsx_sample.tsx --json
sview src/lib.rs --json --depth 2
```

Text output is a compact tree outline:

```text
src/main.rs (rust)
├─ struct Cli L8-31 — struct Cli {
├─ enum OutputFormat L34-37 — enum OutputFormat {
└─ function main L39-54 — fn main() -> Result<()> {
```

TypeScript output uses the same shape:

```text
tests/fixtures/typescript_sample.ts (typescript)
├─ interface User L1-3 — export interface User {
├─ type UserId L5-5 — type UserId = string;
├─ enum Mode L7-10 — enum Mode {
└─ class Service L12-16 — export class Service {
   └─ method load L13-15 — async load(id: UserId): Promise<User> {
```

The intended workflow is:

1. try the cheapest locator first: file names, `rg`, or existing compiler/test output;
2. if that gives a precise file and line range, read that range directly and skip
   `sview`;
3. if the target is still approximate, spans multiple candidates, or would require
   broad reads, run `sview` with a shallow `--depth` / `--max-nodes` limit;
4. read only the relevant range with a focused command such as `sed -n '120,180p'`;
5. patch or inspect the exact range, then rerun `sview` or tests if structure changed.

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
- `tree-sitter`, `tree-sitter-rust`, `tree-sitter-java`, `tree-sitter-javascript`, and `tree-sitter-typescript` for code outlines.

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
cargo coverage
```

`cargo coverage` is a Cargo alias for `cargo llvm-cov --workspace --all-targets --summary-only`.

Initial coverage target: keep the core library line coverage at **70% or higher** while the project is small. Parser behavior should be covered with checked-in fixtures under `tests/fixtures/` for every supported language.

## Release

Release readiness checks:

```bash
cargo fmt -- --check
cargo test
cargo coverage
cargo publish --dry-run
```

The crate is licensed under Apache-2.0. Release tags use the `vMAJOR.MINOR.PATCH` form, for example `v0.1.0`.

Pushing a release tag runs the GitHub release workflow. It uploads Linux amd64,
macOS amd64, and macOS arm64 tarballs plus `checksums.txt`, and generates a
Homebrew formula for `holon-run/homebrew-tap` when the tap token is configured.

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
├── LICENSE
├── .github/
│   └── workflows/
│       └── ci.yml
├── src/
│   ├── analyzer.rs
│   ├── javascript.rs
│   ├── markdown.rs
│   ├── model.rs
│   ├── render.rs
│   ├── rust.rs
│   ├── util.rs
│   ├── lib.rs
│   └── main.rs
├── skills/
│   └── sview/
└── tests/
    └── fixtures/
```
