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

**What NOT to do**
- Do not refactor correct code just to be "more Rusty"
- Do not introduce new abstractions or traits
- Do not change public API signatures (unless there is a bug)
- Do not add unnecessary comments

### 3. Fix

- Only fix issues you are certain about — do not guess
- After each fix, run `cargo check -p <crate>` to verify compilation
- For core logic changes, run the relevant unit tests

### 4. Loop

After each round of fixes, re-analyze the file to confirm nothing was missed. Stop only when you can clearly state: "no logic errors, no lock issues, no worthwhile simplification remaining."

## Output Format

- Start each round by stating what problems were found
- After fixing, explain what changed and why
- On the final round, explicitly state "no issues, stopping"
- Do not output meaningless progress descriptions
