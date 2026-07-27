---
description: Triage and resolve review-bot findings — fix Critical/Major, loop until clean
---
# /shadow:fix-pr — Answer the Reviewer

The independent review bot is the second identity that makes agent-authored
changes non-self-approved. Unresolved CRITICAL/MAJOR findings are a hard gate
in `shadow-ci`'s post-review phase (`<!-- shadow-ci:review-gate -->`) —
resolving them promptly is just cheaper than being blocked at merge.

## Phase 1: Fetch all review findings

```bash
PR=$(gh pr view --json number --jq .number)
# Review-thread comments (inline findings):
gh api repos/{owner}/{repo}/pulls/${PR}/comments --paginate \
  --jq '.[] | {id, path, line, user: .user.login, body: .body[0:400], in_reply_to_id}'
# Review summaries:
gh api repos/{owner}/{repo}/pulls/${PR}/reviews --paginate --jq '.[] | {user: .user.login, state, body: .body[0:400]}'
# Thread resolution state (only the GraphQL API has it):
gh api graphql -f query='query($owner:String!,$repo:String!,$pr:Int!){
  repository(owner:$owner,name:$repo){pullRequest(number:$pr){
    reviewThreads(first:100){nodes{id isResolved isOutdated path comments(first:1){nodes{body author{login}}}}}}}}' \
  -f owner={owner} -f repo={repo} -F pr=${PR}
```

Filter to bot reviewers (logins ending `[bot]`, or the configured REQUIRED_REVIEWERS) and **unresolved,
non-outdated** threads. No review posted at all → summon it
(its re-review command if it has one, or push a commit to re-trigger), then wait and re-fetch.

## Phase 2: Triage

Classify each unresolved finding by the bot's own severity tag (bots
marks `Critical`/`Major`/`Minor`/`Nitpick`; treat `potential bug`,
`security`, and data-loss findings as Major even if untagged):

| Severity | Policy |
|---|---|
| CRITICAL / MAJOR | MUST be fixed, or justified as false positive with evidence. Hard gate. |
| MINOR / NITPICK | Optional — fix if cheap and clearly right, otherwise leave a brief reply. |

Present the triage table to the user in interactive sessions.

## Phase 3: Fix or justify — never ignore

For each CRITICAL/MAJOR finding, exactly one of:

1. **Fix it.** Make the code change. If the fix grows beyond the ticket's spec,
   update the ticket first (`/shadow:load` scope rule).
2. **Justify it as a false positive.** Reply on the thread with a **concrete**
   refutation — point at the guard clause, the type, the test, or the invariant
   that disproves the finding. "Working as intended" without evidence does not
   count. Then resolve the thread:
   ```bash
   gh api graphql -f query='mutation($id:ID!){resolveReviewThread(input:{threadId:$id}){thread{isResolved}}}' -f id={thread_id}
   ```

Never resolve a thread silently, and never weaken a check, delete a test, or
suppress a linter rule to make a finding disappear.

## Phase 4: Commit, push, request re-review

```bash
git add -A
git commit -m "{IDENTIFIER}: address review findings"
git push
git commit --allow-empty -m "re-review" && git push   # re-trigger the review bot on new commits
```

If files were added/removed, update the PR body's `## Changes` section to keep
every changed file listed (or the audit phase goes red next).

## Phase 5: Loop until clean

Wait for the bot's re-review (poll every 60s, max 10 minutes, e.g.
`gh api .../pulls/${PR}/reviews` for a new review newer than your push). Then
re-run Phase 1:
- New CRITICAL/MAJOR findings → back to Phase 2.
- None unresolved → verify the gate:
  ```bash
  gh api repos/{owner}/{repo}/issues/${PR}/comments --paginate --jq '[.[] | select(.body | contains("shadow-ci:review-gate"))] | last | .body'
  gh pr checks ${PR}
  ```
  Report the final state (findings fixed / justified, checks status) to the user.

## STOP conditions
- No open PR on this branch → nothing to fix; run `/shadow:finish` first.
- A finding reveals a real security issue with exposed secrets → STOP the loop, rotate first (incident discipline), then continue.
- Bot doesn't re-review within 10 minutes → report and stop polling; the user can rerun this command.
- Disagreement with the bot you cannot concretely refute → surface it to the user rather than resolving the thread yourself.
