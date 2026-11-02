# Browser pool: tests, measurements and the platform smoke gate

Refs #856. Only measured values are stated here; where nothing was measured, it says so.

## Tests

`tests/browser_pool_test.rs` (real Chrome, local fixtures, all `#[ignore]`):

```bash
cargo test --test browser_pool_test -- --ignored --test-threads=1
```

| Test | Counter-case |
|---|---|
| `isolated_pages_get_their_own_context_and_it_is_destroyed_on_return` | one browser context per isolated page, disposed when the page returns |
| `shared_pages_use_no_extra_context_and_are_reused` | shared policy: no extra context, page reused |
| `permits_run_out_at_max_pages_and_come_back_on_drop` | permit accounting, `PoolTimeout`, no page beyond `max_pages` |
| `isolated_return_gives_the_slot_back_not_only_the_permit` | `pages_created` and permit both restored |
| `a_waiting_acquire_that_is_cancelled_leaks_no_permit` | cancellation while waiting for a permit |
| `an_acquire_cancelled_during_page_creation_leaks_no_slot` | cancellation during page creation (see below) |
| `a_page_that_cannot_be_reset_is_discarded_and_its_slot_restored` | failed page cleanup |
| `a_page_stuck_in_a_script_loop_frees_its_slot_within_the_return_budget` | hung renderer |
| `a_server_that_never_answers_ends_in_an_error_within_the_timeout_budget` | hung page, per-page timeout |
| `a_cancelled_audit_task_returns_its_page` | aborted task hands its slot back |
| `close_ends_the_browser_and_removes_its_profile` | shutdown: no Chrome process, no profile directory left |

### Defect found and fixed

`BrowserPool::acquire` forgot its semaphore permit and counted the slot, then awaited page creation.
A caller that gave up at that point (own timeout, aborted task) left the pool one slot smaller for
the rest of the run. Before the fix `an_acquire_cancelled_during_page_creation_leaks_no_slot` failed
(`Pool: 1/1 pages, 0 available`); it passes now. Taking or creating the page runs in its own task
since then, and a page nobody is waiting for any more is returned to the pool.

## Measurements (macOS arm64)

`scripts/bench-pool.py` runs the release CLI against a local fixture server (generated clean pages,
5 to 55 kB each), default `--interactive full`, `--format json`, and records wall time, peak RSS
of the whole process tree (auditmysite plus Chrome and helpers, sampled every 100 ms, so a lower
bound) and pages missing from the report.

Machine: Apple M4 Pro, 14 cores, 48 GB, macOS 27.2, Google Chrome 156.0.8078.80 (full browser,
headless), auditmysite 1.8.0 plus the pool fix, 3 runs per scenario. Ranges are min to max.

| Scenario | URLs | Concurrency | Wall time (s) | Peak tree RSS (MB) | Pages missing |
|---|---|---|---|---|---|
| single | 1 | 1 | 10.9 to 11.9 | 1380 to 1506 | 0 |
| small | 6 | 1 | 36.0 to 36.6 | 1840 to 2015 | 0 |
| small | 6 | 3 | 14.2 to 14.6 | 2987 to 3164 | 0 |
| large | 30 | 1 | 170.9 to 173.7 | 2058 to 2144 | 0 |
| large | 30 | 3 | 61.4 to 67.2 | 3296 to 3326 | 0 |
| large | 30 | 6 | 34.8 to 36.8 | 5122 to 5386 | 0 |

The CLI process itself peaked at 53 to 192 MB; the rest is Chrome.

What was and was not observed:

- No failures and no stalled run, including with journeys (keyboard tab walk) at concurrency 3 and 6
  on this machine and this Chrome. The macOS keyboard-journey stall that motivated the default of
  concurrency 1 (`parallel_journeys_stall`, plan 65) was **not reproduced**. One machine, one
  Chrome version and clean fixture pages do not prove it gone, so the default stays serialized on
  macOS with journeys; an explicit `--concurrency` overrides it.
- Not distinguished: CPU scheduling versus CDP connection versus page acquisition. The script
  measures end to end only; no per-phase timing exists yet.
- A renderer stuck in a script loop gave its slot back after 10.0 s, which is the page reset
  timeout (`PAGE_RESET_TIMEOUT`), not an instant recovery.
- A server that accepts and never answers ends in `PageLoadTimeout` after 26.0 s with
  `timeout_secs = 12`.
- Linux x64/ARM64 and Windows/Edge were not measured; this machine cannot.

## Platform smoke gate

The full QA (`ci.yml`, called by `release.yml` as `quality-gates`, which `build` needs) has the job
`native-browser-smoke`: one native run per supported platform, because a cross-build is not a
runtime test.

| Leg | Runner | Browser |
|---|---|---|
| macOS arm64 | `macos-15` | Google Chrome of the runner image |
| Linux x64 | `ubuntu-24.04` | Chrome via `browser-actions/setup-chrome` |
| Linux arm64 | `ubuntu-24.04-arm` | Chrome for Testing via `setup-chrome` |
| Windows x64 | `windows-2025` | Microsoft Edge, explicit path (Edge only) |

Each leg runs `browser_detection_chrome` and `browser_pool_test` (`--ignored`, one thread), audits
`tests/fixtures/perfect.html` from a local server through the CLI, checks that
`measurement_profile.browser` names the platform, a matching product and the runner's browser
version, and writes OS/architecture, browser version and commit to the job summary.

Findings from the first runs: Chrome for Testing unpacked by `setup-chrome` on macOS is a bare
`Contents` folder outside an app bundle; its helper processes fail the Mach port rendezvous and no
page can be created, so that leg uses the preinstalled Chrome. On Windows `--version` of Edge gives
no answer, so the version comes from the version folder next to `msedge.exe`.
