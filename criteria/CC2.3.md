---
id: CC2.3
family: CC2 — Communication and Information
category: Security (Common Criteria)
coso: COSO Principle 15
title: Communication with External Parties
weight: 2
automatable: partial
nature: document
---

# CC2.3 — Communication with External Parties

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 15: The entity communicates with external parties regarding matters affecting the functioning of internal control.

## What it means
This is the outward-facing twin of CC2.2. The auditor checks that the company tells customers, vendors, and other outsiders what they need to know about the system — its commitments (security terms in contracts/ToS, SLAs, privacy policy), its boundaries and their responsibilities (what the customer must do, e.g., manage their own users' access) — and that outsiders can reach the company about security matters: a security contact channel, a way to report vulnerabilities, and a defined process for notifying affected parties about incidents and breaches.

For a tiny SaaS startup the artifact set is small and concrete: Terms of Service / MSA security clauses, a privacy policy, a public `security@` address (plus `security.txt` and a trust/security page if you want to look grown-up), the customer-notification section of the incident response plan, and status/uptime communication (a status page or a stated email process). Inbound matters as much as outbound — auditors like to see that a vulnerability report or customer security question would land somewhere monitored and get tracked.

Vendors are external parties too: communicating your security expectations to subprocessors (DPAs, reviewing their SOC 2 reports) is usually evidenced here and under CC9.2, so keep the vendor-review trail consistent.

## Points of focus (2022 revision, summarized)
Guidance from COSO as mapped in the 2022 TSC — illustrative, not required:
- **Communicates to external parties** — processes communicate relevant, timely information to shareholders, customers, vendors, regulators, and others.
- **Enables inbound communications** — open channels allow input from customers, vendors, external auditors, and others, giving management relevant information.
- **Communicates with the board of directors** — relevant information from external assessments/parties reaches the board (oversight body).
- **Provides separate communication lines** — external parties have channels (e.g., whistleblower-type) for confidential communication when normal channels don't work.
- **Selects relevant method of communication** — medium and timing fit the audience and legal/regulatory requirements.
- **Communicates system objectives, responsibilities, and incident information to external users** (TSC supplemental) — commitments, system boundaries, user responsibilities, changes, and security incidents/breaches are communicated to affected external parties, including how to report failures and concerns.

## What the auditor will ask for
- Customer-facing commitments: Terms of Service/MSA security sections, SLA, privacy policy — with effective dates covering the period.
- Public security contact evidence: `security@` mailbox existence and monitoring, `security.txt`, vulnerability-disclosure statement, trust/security page.
- Incident response plan section on external/customer and regulator notification, including breach-notification timelines.
- Evidence of external notifications sent during the period (incident emails, status-page posts), or attestation none were required.
- Records of inbound security communications (customer questionnaires, vulnerability reports) and how they were handled.
- Vendor/subprocessor communication evidence: DPAs, security expectations conveyed, subprocessor list published to customers if applicable.
- Support-channel description: how customers report problems and how reports are tracked.

## How a tiny AI-first startup satisfies it
- Publish the basics on the website: privacy policy, ToS with a security-commitments clause, a `/security` page stating commitments, boundaries, customer responsibilities, and how to report vulnerabilities, plus `/.well-known/security.txt` pointing at `security@yourco.com`.
- `security@` and `support@` as Google Workspace groups containing both founders — inbound reports can't die in one person's inbox. Every inbound security report becomes a Linear ticket (`inbound-security` label).
- IR plan external section: `runbooks/incident-response.md` defines when and how customers/regulators are notified (who drafts, who approves, target timelines). If AI subprocessors (OpenAI/Anthropic) handle customer data, list them on a subprocessor page and commit to notice-of-change.
- Status communication: a hosted status page (Instatus/BetterStack free tier) or a documented email-blast process — either is fine if written down.
- Vendor side: signed DPAs with subprocessors stored in the evidence store; annual review of key vendors' SOC 2 reports tracked as Linear tickets (shared evidence with CC9.2).
- Route external assessment results (pen test summary, the SOC 2 report itself) into the quarterly oversight review — that closes the "external info reaches the board" loop.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| `security.txt` published and unexpired | Website | HTTP GET `https://{domain}/.well-known/security.txt`; parse `Expires` field |
| Privacy policy, ToS, and security page reachable | Website | HTTP GET returns 200 for `/privacy`, `/terms`, `/security` (paths from config) |
| `security@` group exists with ≥2 members | Workspace Admin API | `GET admin/directory/v1/groups/{security@domain}/members` — count ≥ 2 |
| Inbound security reports tracked | Linear API | Issues labeled `inbound-security` exist and are triaged (assignee + state) within SLA |
| IR plan contains external-notification section with timelines | GitHub | Fetch `runbooks/incident-response.md`, grep for customer/regulator notification headings |
| Subprocessor list published and current | Website + GitHub | HTTP GET subprocessor page; compare list against `inventory/systems.md` vendor entries |
| Vendor DPA/SOC 2 review tickets completed annually | Linear API | `vendor-review` labeled issues, one per key vendor per year, Done |
| Status page live | Website | HTTP GET status-page URL returns 200 |
| Adequacy of contractual commitments language | MANUAL | Auditor/legal review of ToS/MSA |
| Whether actual notifications met obligations | MANUAL | Auditor reviews any incident notifications against contracts/law |

## Evidence artifacts
- Live URLs (snapshotted quarterly as PDFs to `evidence/external-comms/`): privacy policy, ToS, `/security` page, `security.txt`, subprocessor list, status page.
- Workspace export of `security@` group membership in `evidence/workspace/groups-<date>.json`.
- Linear exports: `inbound-security` tickets and `vendor-review` tickets for the period, archived to `compliance-archives`.
- `runbooks/incident-response.md` external-notification section (version-controlled).
- Signed DPAs and vendor SOC 2 review notes in the evidence store.
- Copies of any customer incident notifications or status-page incident posts sent during the period, or a signed attestation that none were required, in `evidence/external-comms/`.
