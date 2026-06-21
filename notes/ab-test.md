# sview agent A/B test notes

This note captures the reusable experiment protocol for comparing an agent that
uses the `sview` skill against an otherwise comparable agent that does not.

## Lessons from issue #1

Issue #1 was a small refactor: extract shared analyzer tree-building logic.

Measured result:

- A, with `sview` skill: 45,865 total tokens, 39 model rounds.
- B, without `sview` skill: 37,578 total tokens, 29 model rounds.
- Both sides reported roughly 12 tool calls and passed verification.
- B produced the implementation that was merged.

Interpretation:

- `sview` helped make structure and line ranges explicit, but this task was too
  small to show a token or time advantage.
- The experiment should not force use of `sview`; it should make structural
  navigation available and then measure whether the agent naturally benefits.
- Better candidate tasks are medium-sized refactors where several source files
  have similar shapes and the edit boundary is not obvious from a single `rg`.

## Reusable protocol

Use the same issue, base branch, model, and acceptance criteria for both agents.

For the `sview` side:

- Provide the `sview` skill.
- Encourage using `sview` as a navigation map before broad reads of supported
  source files.
- Do not forbid normal tools such as `rg`, `sed`, tests, or focused file reads.

For the control side:

- Do not provide the `sview` skill.
- Allow normal repository inspection and implementation workflow.

For both sides, record:

- model provider and model
- start and end time
- wall-clock duration
- total token usage, preferably split into input/output/cache if available
- model rounds
- tool-call count
- files changed
- verification commands and results
- PR URL and commit
- qualitative implementation notes

After both PRs are available, put the same A/B report on both PRs, choose the
winner based on correctness, simplicity, maintainability, verification, and
measured cost, then merge only the winning PR.
