---
description: Record a gated main deployment with a human-confirmed release record
---
# /shadow:release — Record Production

Merges to protected `main` are the only normal production path. Deployment is
keyless and automatic, so this command does not move code between long-lived
branches. It records the already-deployed `main` SHA and the founder's release
decision in `compliance-archives`.

## Phase 1: Preflight

```bash
git fetch origin main compliance-archives
MAIN_SHA=$(git rev-parse origin/main)
gh run list --branch main --limit 20 --json databaseId,workflowName,conclusion,headSha,url
```

Stop if any required workflow for `MAIN_SHA` is pending or failed. Review
recent archive records and resolve every unexplained `is_bypass: true` item
through `/shadow:hotfix` before continuing.

## Phase 2: Build the release population

Find the previous release record in `compliance-archives`; if none exists, use
the beginning of the declared audit window. Gather every merged PR and ticket
since that point:

```bash
gh pr list --base main --state merged --json number,title,url,mergedAt,mergeCommit --limit 200
```

Present `| Ticket | Title | PR | Main SHA |` and identify the exact deployed
revision. A missing ticket or archive record is an exception, not a blank to
fill silently.

## Phase 3: Founder confirmation

Generate a random confirmation word and ask the authenticated founder to type
it exactly. Record their GitHub login, the word challenge result, timestamp,
`MAIN_SHA`, workflow URLs, PRs, and tickets. An agent never supplies the answer.

## Phase 4: File the protected evidence

Preferred:

```bash
RELEASED_BY="<authenticated founder>" ARCHIVES_PUSH=1 \
  .shadow/ci/target/release/shadow-ci release-record
```

The record must be a new file under `releases/`; never edit an existing record.
The `compliance-archives` ruleset rejects force pushes and deletion, and only
the designated GitHub Actions integration may bypass its pull-request rule to
append machine-created evidence. This is tamper-evident evidence, not a claim
that GitHub storage is legally immutable.

## Phase 5: Close the loop

Create or close a release ticket containing the release record path, `MAIN_SHA`,
deployment workflow URL, and included PRs. Link that ticket from each included
change ticket. Report the release SHA, time, actor, record path, and any open
exceptions.

## STOP conditions

- Required workflow pending or failed.
- Main SHA does not match the deployed digest recorded by the deploy workflow.
- Unexplained bypass or missing post-merge archive.
- Confirmation mismatch.
- Archive push rejected.

Never weaken a rule or rewrite archive history to make a release pass.
