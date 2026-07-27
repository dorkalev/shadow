---
description: Onboarding with evidence — least-privilege grant set confirmed by a human, checklist ticket with provisioning, attestation, and day-14 verification (CC6.2/CC1.4)
---
# /shadow:onboard — Grant Access, Leave a Trail

Onboards a new person (CC6.2 provisioning, CC1.4 policy attestation) so the
evidence exists before the first grant is made. **Args:** name and role
(`/shadow:onboard "Jane Doe" backend-engineer`).

Three phases: **gather** (zero questions), **interview** (judgment only),
**file** (zero questions). This command does not execute grants — it produces
the approved grant set and the checklist ticket that drives provisioning; each
grant carries its exact command so execution is copy-paste with a paper trail.

**Segregation of duties:** the grant set is approved by a human, named on the
record. The agent never approves its own proposal. No approval → the ticket is
filed with the grant set marked PROPOSED/OPEN and nothing should be
provisioned from it.

## Phase 1: GATHER (no questions asked)

**Step 1 — the role→grants mapping.** Read the access register in the
policies repo (look for `access-register`, `registers/access*`, or the
register referenced by the policy pack). If it maps the given role to a grant
set, that is the baseline proposal.

**Step 2 — no mapping? Propose from a comparable person.** Pick an existing
teammate in the same role (or nearest), pull their live grants:
```bash
USER_FILTER={comparable-user} .shadow/ci/target/release/shadow-ci access-review
```
and propose that set MINUS anything that looks person-specific (admin/owner
roles, personal service-account keys, repos unrelated to the role). Mark each
proposed grant with its provenance (register entry vs. copied-from-{user}).

**Step 3 — policy pack version.** Read the current policy pack version from
the policies repo (for the attestation step).

## Phase 2: INTERVIEW (judgment only)

One batched exchange, evidence inline:

> **Proposed grant set for {name} ({role})** — source: {register / modeled on
> {comparable}, whose full set is shown for comparison}:
> - GitHub org member; write on {repos}
> - GCP: {role} on {project}
> - Tracker seat, Workspace account
>
> Least privilege check: {comparable} also has {extra grants} which I have
> already trimmed — restore any? And what else should be **trimmed** from the
> proposal? Then: confirm start date and equipment (laptop issued? company or
> BYOD — disk encryption + screen lock required either way).
>
> Who approves this grant set?

Collect: final grant set, start date, equipment answer, approver's name. Cuts
are accepted silently; **additions** beyond the register/comparable baseline
need a one-line justification recorded next to the grant. Unanswered →
everything stays PROPOSED/OPEN.

## Phase 3: FILE (no questions asked)

**Create the onboarding checklist ticket** — Linear MCP (`save_issue`) when
available, otherwise `gh issue create`. Title: `Onboarding: {name} ({role}) —
starts {date}`. Body: approver's name, grant-set source, then the checklist:

1. **One checkbox per approved grant**, each with its exact provisioning
   command:
   ```bash
   gh api -X PUT /orgs/{org}/memberships/{login} -f role=member
   gh api -X PUT /repos/{owner}/{repo}/collaborators/{login} -f permission=push
   gcloud projects add-iam-policy-binding {project} \
     --member="user:{email}" --role="{role}"
   ```
   (Workspace/tracker seats: the manual console steps, spelled out.)
2. **MFA verification** — checkbox: confirm 2FA enforced on the GitHub
   account and 2SV on Workspace before any production-adjacent grant is made;
   note how it was verified.
3. **Policy attestation** — open a separate personal attestation issue
   assigned to {name}: body instructs *"comment «I have read and agree to the
   policy pack v{X}» to sign"*, linking the policy pack at that version. The
   comment (their identity, timestamped) is the CC1.4 evidence. Link it from
   the checklist.
4. **Day-14 verification** — checkbox dated start+14: run
   ```bash
   USER_FILTER={login-or-email} .shadow/ci/target/release/shadow-ci access-review
   ```
   and diff against the approved set in this ticket — the scan must match
   **exactly, no more and no less**; any drift gets a revocation or a recorded
   approval, never a shrug.

If the register had no entry for this role, note in the summary that the
approved set should be added to the access register via PR (its own ticketed
change) so the next hire starts from a register, not archaeology.

**Report a one-screen summary:** approved grant set (with trims/additions),
approver, start date, equipment note, ticket ID, attestation issue ID, day-14
date, OPEN items.

## STOP conditions
- No approver responds → file the ticket with the set marked PROPOSED, state
  clearly that nothing may be provisioned until a human approves on the ticket.
- Role is unknown and no comparable person exists → ask only that irreducible
  question (what should this role access?) — that answer is the interview.
