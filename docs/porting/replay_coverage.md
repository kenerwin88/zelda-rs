# Replay Coverage

`scripts/replay_coverage.py` treats the replay-save route as an integration
test and records which C and Rust functions execute.

Coverage is a blind-spot tool, not a parity proof. A function can be covered and
still behave differently. Use coverage to choose missing route windows, then use
`scripts/replay_bisect.py` and checkpoint comparisons to prove identical state.

## Standard Route

The canonical full-playthrough input is:

```bash
saves/zelda3-combined-route.sav
```

Run coverage for the full standard route:

```bash
python3 scripts/replay_coverage.py --rebuild-c
```

This writes:

```text
target/replay-coverage/c/html/index.html
target/replay-coverage/rust/html/index.html
target/replay-coverage/c-functions.tsv
target/replay-coverage/rust-functions.tsv
target/replay-coverage/parity_matrix.md
```

`parity_matrix.md` normalizes C and Rust function names and highlights:

- mapped functions where only one side was covered
- covered C functions without a normalized Rust match
- covered Rust functions without a normalized C match

The name matching is intentionally approximate. It is a triage report for
finding blind spots, not an authoritative porting map.

## Windowed Coverage

Prefer named windows when checking whether specific systems are covered:

```bash
python3 scripts/replay_coverage.py --rebuild-c \
    --window file-select:42998 \
    --window overworld:248779 \
    --window late-route:1073092
```

For parity work, coverage by window is usually more useful than one merged total
because it shows which gameplay area exercised each subsystem.

## Requirements

The script uses LLVM coverage directly:

- `clang`
- `llvm-profdata`
- `llvm-cov`

On macOS, the script also resolves LLVM tools through `xcrun` when they are not
on `PATH`.
