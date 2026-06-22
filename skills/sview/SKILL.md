---
name: sview
description: "Use sview to navigate code and Markdown structure with compact outlines and line ranges before broad reads."
---

# sview

Use `sview` to generate structured outlines for source code and Markdown files, helping agents quickly navigate unfamiliar or structurally broad areas before reading or editing.

Use `sview` when a compact structure map is likely to reduce more uncertainty than it adds overhead. Do not use it reflexively: for small files, known ranges, or direct text lookups, prefer `rg` / focused `sed` first.

## Install

Prefer the released binary when using this skill outside the `sview` repository:

```bash
brew tap holon-run/tap
brew install sview
```

Or download a prebuilt archive from GitHub Releases:

```bash
curl -L https://github.com/holon-run/sview/releases/latest/download/sview-linux-amd64.tar.gz | tar -xz
chmod +x sview
./sview --help
```

Use `sview-darwin-amd64.tar.gz` or `sview-darwin-arm64.tar.gz` on macOS.

Other install paths:

```bash
cargo install sview
cargo install --path .
```

## When to use

- Before reading an unfamiliar Markdown, Rust, JavaScript, or TypeScript file mostly end-to-end, especially when it is large enough that a structural map can avoid broad reads.
- Before editing a symbol, section, or test when only its approximate location is known and text search does not identify a tight range.
- When several candidate files or symbols need quick triage before choosing exact ranges.
- After parser changes, to inspect representative project files and confirm the outline remains useful.

## When not to use

- When `rg` directly answers the question and no structural map is needed.
- When the exact small range to read or patch is already known.
- When the file is small enough to inspect directly with one focused read.
- When there are only one or two obvious candidate files and their relevant functions are easy to locate by name.
- When the file type is unsupported and the outline would not guide a better next read.

## Decision flow

1. Try the cheapest locator first: file names, `rg`, or existing compiler/test output.
2. If that gives a precise file and line range, read that range directly; skip `sview`.
3. If the target is still approximate, spans multiple candidates, or would require broad reads, run `sview` with a shallow `--depth` / `--max-nodes` limit.
4. Use the reported line ranges for the next focused read before editing.

## Commands

```bash
sview README.md --depth 2
sview README.md src/lib.rs --depth 1
sview src/lib.rs --depth 1 --max-nodes 40
sview tests/fixtures/typescript_sample.ts --depth 2
sview tests/fixtures/javascript_sample.js tests/fixtures/tsx_sample.tsx --json
sview path/to/file.rs --json --depth 2
```

Default text output is a compact tree outline, for example:

```text
src/main.rs (rust)
├─ struct Cli L8-31 — struct Cli {
├─ enum OutputFormat L34-37 — enum OutputFormat {
└─ function main L39-54 — fn main() -> Result<()> {
```

TypeScript output follows the same tree shape:

```text
tests/fixtures/typescript_sample.ts (typescript)
├─ interface User L1-3 — export interface User {
├─ type UserId L5-5 — type UserId = string;
├─ enum Mode L7-10 — enum Mode {
└─ class Service L12-16 — export class Service {
   └─ method load L13-15 — async load(id: UserId): Promise<User> {
```

Use the reported `start_line` / `end_line` ranges to choose the next focused `sed -n '<start>,<end>p'` or patch target.

## Expectations

- Treat `sview` output as a navigation map, not as a replacement for reading the exact target range before editing.
- `sview` should lower navigation cost. If using it would add a separate planning pass without narrowing the next read, skip it.
- If the outline exposes incorrect ranges or missing important structures, fix `sview` first when that bug would mislead agent workflows.
