---
name: review-and-fix
description: Deep analysis and iterative fixing of a Rust source file. Finds logic errors, lock/concurrency issues, and simplification opportunities, then fixes them one by one until the file is clean.
---

# review-and-fix

Deep-analyze the specified file, find logic errors, memory/lock issues, and simplification opportunities, and fix them iteratively until no problems remain.

## Usage

```
/review-and-fix <file_path>
```

**Examples**

```
/review-and-fix src/mqtt-broker/src/subscribe/buckets.rs
/review-and-fix src/mqtt-broker/src/subscribe/directly_push.rs
```

## Execution Flow

Each round follows this sequence until nothing left to fix:

### 1. Read

Fully read the target file. Read related files as needed (callers, struct definitions it depends on) to understand context.

### 2. Analyze

Check in priority order:

**Logic Errors (must fix)**
- Asymmetric data structure operations: `add` writes N indexes, `remove` only cleans N-1
- offset/commit semantics: committing after push failure causes message loss
- Key collisions: separator choice produces identical keys for different inputs

**Concurrency/Lock Issues (must fix)**
- DashMap `entry()`, `get()`, `get_mut()` return `Ref`/`RefMut` that hold shard locks — not released during `.await`
- `RwLock` read lock held during `.await` blocks write lock
- Fix: `.clone()` the data to drop the guard before awaiting; or store `Arc<T>`

**Simplification (apply judiciously)**
- Repeated `get_mut` + `else { insert }` → `entry().or_default()`
- Redundant `else { return x }` → remove the else
- Two-step `let x = ...; let x = match x { Some(v) => v, None => return }` → `let Some(x) = ... else { return }`
- Nested `if !condition { ... }` → `if condition { continue }`
- Duplicate import lines → merge
- Temporary flag variables (`let mut failed = false; ... if !failed { commit() }`) → early return

**Comments (keep lean)**
- Remove redundant or obvious comments; the code should speak for itself
- Keep only comments that explain a non-obvious *why* (an invariant, a subtle ordering, a footgun)
- Do not over-comment — fewer, higher-signal comments beat many noisy ones

**Test Cases (deep review, then trim — do not just add)**
- Deep-review every existing test before adding anything: does it assert a real behavior, or just re-exercise the happy path another test already covers? Does the assertion actually fail if the logic under test is broken (mutate the code mentally and check)?
- Default action is consolidation, not addition: merge near-duplicate tests into one parametrized/table-driven case, delete tests that assert trivial defaults or that duplicate coverage another test already provides
- Only add a new test when a real gap exists: a pure decision function or bug-prone branch with zero coverage. One targeted case per gap — do not pad with variations that don't exercise a new path
- Prefer testing the pure/extractable logic directly over standing up heavy mocks for orchestration glue; if a path can only be tested by mocking a large dependency, that's usually a sign to extract the pure logic rather than write the mock
- Keep the total test count as small as possible while still covering every distinct branch/outcome once — "few, focused, high-signal" beats "thorough-looking"

**Naming (align names with behavior)**
- Function names: does the name describe what the function actually does? Rename misleading or vague names (e.g. a `get_*` that mutates, a `*_switch` that only computes)
- File / module names: does the file name match its content and responsibility? Flag/rename when it has drifted
- When renaming, update every reference (callers, imports, `mod` declarations) and re-run `cargo check`. Be conservative with widely-used public names — only rename when the current name is genuinely misleading, not for taste

**What NOT to do**
- Do not refactor correct code just to be "more Rusty"
- Do not introduce new abstractions or traits
- Do not change public API signatures (unless there is a bug, or a name is genuinely misleading — then rename and update all call sites)
- Do not add unnecessary comments

### 3. Fix

- Only fix issues you are certain about — do not guess
- After each fix, run `cargo check -p <crate>` to verify compilation
- For core logic changes, run the relevant unit tests

### 4. Loop

After each round of fixes, re-analyze the file to confirm nothing was missed. Stop only when you can clearly state: "no logic errors, no lock issues, no worthwhile simplification remaining, names match behavior, test coverage adequate and focused, comments lean."

## Output Format

- Start each round by stating what problems were found
- After fixing, explain what changed and why
- On the final round, explicitly state "no issues, stopping"
- Do not output meaningless progress descriptions
