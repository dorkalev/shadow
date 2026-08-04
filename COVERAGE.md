# Coverage ratchet

The CI workflow measures line coverage with `cargo llvm-cov` on every change.
It is a merge gate: coverage may rise but must not fall below the current
baseline. The baseline was measured on PR #1, run `30893816032`:

| Crate | Line coverage | Enforced floor |
|---|---:|---:|
| `actions/shadow-ci` | 20.63% | 20% |
| `website/app` | 7.48% | 7% |

The floors are deliberately honest—not an arbitrary 100% badge. Raise them in
the same pull request that adds tests. The completion target is 100% line and
branch coverage for owned decision logic. CLI/network adapters may be excluded
only when a test seam cannot exercise them; each exclusion requires a short
comment naming the integration test that covers the behavior instead.

## Order of work

1. `shadow-ci`: isolate `gh`, `git`, `curl`, and filesystem access behind test
   interfaces; table-test every pass, fail, and unknown result for `check`,
   `archive`, `attest`, rituals, and `verify`.
2. Website: separate parsing, persistence, ingest authorization, and HTML view
   models from Axum handlers; add SQLite fixture tests for every route and
   status transition.
3. Add process-level tests with fake command binaries for the adapters. They
   exercise exit codes, malformed JSON, permission failures, retries, and
   evidence writes without live GitHub or GCP credentials.
4. Raise each crate's floor until it reaches 100%; remove a temporary floor
   only when the crate is at 100%.
