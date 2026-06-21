# sview

Use `sview` to generate structured outlines for source code and Markdown files, helping agents quickly navigate projects before reading or editing.

Use `sview` when a compact structure map can reduce uncertainty before reading or editing. Prefer it before broad text reads of Markdown, Rust, JavaScript, or TypeScript files; skip it for tiny edits where the exact range is already known.

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

- Before reading an unfamiliar Markdown, Rust, JavaScript, or TypeScript file end-to-end.
- Before editing a symbol, section, or test when only its approximate location is known.
- When several candidate files need quick triage before choosing exact ranges.
- After parser changes, to inspect representative project files and confirm the outline remains useful.

## When not to use

- When `rg` directly answers the question and no structural map is needed.
- When the exact small range to read or patch is already known.
- When the file type is unsupported and the outline would not guide a better next read.

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
- If the outline exposes incorrect ranges or missing important structures, fix `sview` first when that bug would mislead agent workflows.
