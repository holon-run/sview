# sview Dogfooding

Use `sview` before broad text reads when a Markdown or Rust file may be large enough that an outline can guide a targeted follow-up range.

## When to use

- Before reading an unfamiliar Markdown or Rust file end-to-end.
- Before editing a symbol, section, or test when only its approximate location is known.
- After parser changes to inspect `sview`'s own `src/lib.rs` output.

## Commands

```bash
cargo run --quiet -- README.md --depth 2
cargo run --quiet -- src/lib.rs --depth 1 --max-nodes 40
cargo run --quiet -- path/to/file.rs --json --depth 2
```

Use the reported `start_line` / `end_line` ranges to choose the next focused `sed -n '<start>,<end>p'` or patch target.

## Expectations

- Treat `sview` output as a navigation map, not as a replacement for reading the exact target range before editing.
- If dogfooding exposes incorrect ranges or missing important structures, fix `sview` first when that bug would mislead agent workflows.
