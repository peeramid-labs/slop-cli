# Detection categories

Every category the engine knows. Categories live in the server's
embedded corpus (`global.yml`), are loaded once at process start, and
match against added lines of every diff.

Fixability is the contract `slop apply` honours:

- **SafeDelete** — apply removes the line outright
- **SafeReplace** — apply rewrites the line via a typed substitution
- **Todo** — apply splices a `// TODO(slop): …` above; code stays
- **Flag** — surface only, no automatic mutation

## Language-agnostic

| Category              | Fixability  | What it catches |
|-----------------------|-------------|-----------------|
| `self_congrats`       | SafeDelete  | LLM editorialising ("comprehensive", "robust", "production-ready") |
| `filler`              | SafeDelete  | Wordy filler ("in order to", "at the end of the day") |
| `prompt_leak`         | SafeDelete  | Chat-turn residue leaked into source |
| `ai_scaffolding`      | SafeDelete  | "Let me", "I'll now", first-person agent narration |
| `hedges`              | SafeDelete  | "essentially", "arguably", filler that softens nothing |
| `listy`               | SafeDelete  | Essay scaffolding leaked from the model |
| `co_authored_ai`      | SafeDelete  | AI co-authorship trailers committed to source |
| `meta_reference`      | SafeDelete  | "see above", "used by X" that rot as code moves |
| `narrative_opener`    | SafeDelete  | "First", "Then" comments narrating code shape |
| `ai_tells`            | Todo        | Competitive / market framing words ("landscape", "tapestry") |
| `template_placeholder`| Todo        | `[Your name]`, `[Insert price]`, `<YOUR_…>` left unfilled |
| `defensive_crud`      | Todo        | `if x is None: x = None` and friends |
| `boolean_redundant`   | Todo        | `if foo == True` |
| `useless_promise_wrap`| Todo        | `new Promise(resolve => resolve(x))` |
| `boundary_off_by_one` | Todo        | Loop bound that looks inclusive/exclusive-swapped |
| `catch_rethrow`       | Todo        | `catch (e) { throw e; }` |
| `null_check_redundant`| Todo        | Overlapping `x !== null && x !== undefined` |
| `naming_slop`         | Todo        | `tmp`, `data`, `result` as the actual function name |
| `branch_no_test`      | Todo        | New control-flow branch without a paired test |
| `error_no_message`    | Todo        | `throw new Error()` with no string |
| `unused_generic`      | Todo        | Generic type parameter declared but never used |
| `double_cast_unknown` | Todo        | TS `as unknown as T` escape hatch |
| `emoji_in_code`       | Todo        | Emoji in non-prose source |
| `hardcoded_url`       | Todo        | `localhost`, `example.com`, `YOUR_…` in source |
| `hardcoded_id`        | Todo        | UUID / API-key shaped literal pasted in |
| `noop_console_log`    | SafeDelete  | `console.log()` with no argument |
| `triple_negation`     | SafeReplace | `!!!x` |
| `redundant_string_cast`| SafeReplace| `String(String(x))` |
| `todo_implement`      | Flag        | `// TODO: implement` style markers |
| `fixme_hack_xxx_comment` | Todo     | `FIXME` / `HACK` / `XXX` markers — fix or track |

## Python

| Category                     | Fixability | What it catches |
|------------------------------|------------|-----------------|
| `python_mutable_default`     | Todo       | `def f(x=[])` |
| `python_print_debug`         | Todo       | `print(...)` in non-test code |
| `python_range_len_loop`      | Todo       | `for i in range(len(x))` |
| `python_pass_placeholder`    | Todo       | `def foo(): pass` body |
| `python_ellipsis_placeholder`| Todo       | `def foo(): ...` body |
| `python_bare_except`         | Todo       | `except:` with no exception class |
| `python_empty_except`        | Todo       | `except X: pass` |
| `python_star_import`         | Todo       | `from x import *` |
| `python_global_statement`    | Todo       | `global x` inside a function |

## Rust

| Category               | Fixability | What it catches |
|------------------------|------------|-----------------|
| `rust_non_test_unwrap` | Todo       | `.unwrap()` / `.expect()` outside `#[cfg(test)]` |
| `rust_pub_no_doc`      | Todo       | `pub fn` / `struct` / `trait` with no `///` doc |

## C# / .NET

| Category                          | Fixability | What it catches |
|-----------------------------------|------------|-----------------|
| `csharp_async_void`               | Todo       | `async void` outside event handlers |
| `csharp_task_blocking_get`        | Todo       | `.Result` / `.Wait()` / `.GetAwaiter().GetResult()` |
| `csharp_count_any_anti_pattern`   | Todo       | `.Count() > 0` instead of `.Any()` |
| `csharp_tolist_foreach`           | Todo       | `.ToList().ForEach(...)` |
| `csharp_throw_generic_exception`  | Todo       | `throw new Exception(...)` |
| `csharp_not_implemented`          | Flag       | `throw new NotImplementedException()` |

## SQL

| Category                  | Fixability | What it catches |
|---------------------------|------------|-----------------|
| `sql_select_star`         | Todo       | `SELECT * FROM …` |
| `sql_where_1_eq_1`        | SafeDelete | `WHERE 1=1` |
| `sql_leading_wildcard_like`| Todo      | `LIKE '%foo%'` (full-scan) |
| `sql_destructive_no_where`| Flag       | `DELETE FROM x` / `UPDATE x SET …` with no WHERE |

## Context gating

A category may declare itself ignored in certain contexts:

- File-path tests (`tests/`, `*_test.go`, `*.spec.ts`) suppress the
  language-specific "no panic / no print / placeholder" rules.
- Inline `#[cfg(test)] mod tests` blocks suppress the same rules from
  the boundary line onward.
- `.md` / glossary string contexts suppress every prose-tuned
  category.

The full gating matrix is enforced inline in
`crates/slop-detector/src/lib.rs` — search for `is_in_test_ctx`.
