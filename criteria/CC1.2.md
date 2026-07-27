---
id: CC1.2
family: CC1 — Control Environment
category: Security (Common Criteria)
coso: COSO Principle 2
title: Board Independence and Oversight
weight: 1
automatable: partial
nature: document
---

# CC1.2 — Board Independence and Oversight

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 2: The board of directors demonstrates independence from management and exercises oversight of the development and performance of internal control.

## What it means
This is the criterion tiny startups worry about most, because most of them have no independent board — the "board" is the two founders. Auditors know this. The criterion does not require a Fortune-500 board; it requires that *someone with a degree of independence from day-to-day management* periodically looks at how internal control is developing and performing, and that this oversight is documented.

Acceptable tiny-startup equivalents, in rough order of strength: (1) an actual board with at least one investor or independent director who receives security/compliance updates; (2) a documented external advisor (fractional CISO, technical advisor, or counsel) who reviews the security program quarterly; (3) a founder-led "management oversight" structure with documented quarterly self-reviews, explicitly disclosed in the system description. Option 3 is common and auditors typically accept it for early-stage companies, but they will describe the limitation; options 1–2 make the report cleaner.

The substance the auditor is looking for: recurring, minuted review of risk, incidents, control monitoring results, and audit findings — by someone who can challenge management. Cadence (quarterly is the norm), agenda, and minutes matter more than the reviewer's title.

## Points of focus (2022 revision, summarized)
Guidance from COSO as mapped in the 2022 TSC — illustrative, not required:
- **Establishes oversight responsibilities** — the board identifies and accepts its oversight responsibilities relative to established requirements and expectations.
- **Applies relevant expertise** — the board defines and periodically evaluates the skills and expertise needed among its members, including (per TSC supplemental guidance) supplementing board expertise on security/technology matters where needed.
- **Operates independently** — the board has sufficient members who are independent from management and objective in evaluations and decision making.
- **Provides oversight of the system of internal control** — the board retains oversight responsibility for management's design, implementation, and conduct of internal control, including tone at the top, control monitoring, and remediation of deficiencies.

## What the auditor will ask for
- Description of governance structure: board composition, or named external advisor(s) and their role, or documented founder-oversight model.
- Board/advisor meeting minutes or quarterly security-review notes covering the audit period (sampled).
- Evidence that security topics were actually on the agenda: risk assessment results, incident summary, monitoring/pen-test findings, remediation status.
- Charter or short document defining oversight responsibilities and cadence (can be one page).
- Bio/qualifications of the advisor or board member providing security oversight (relevant expertise).
- Evidence of follow-up: decisions or action items from oversight reviews tracked to closure.

## How a tiny AI-first startup satisfies it
- Write `policies/governance.md` (one page): who provides oversight, what they review (risk register, incidents, monitor results, access reviews), how often (quarterly), and how it's recorded.
- If you have an investor board seat or an advisor: send them a quarterly security summary (auto-generated from your shadow-tool dashboard is fine) and record their response. An email thread saved to evidence is acceptable minutes.
- If founders-only: hold a documented quarterly internal-control review. Create a recurring Linear ticket "Q_ security & internal control review"; the closing comment includes the agenda, findings reviewed (compliance monitor results, open vulnerabilities, incidents, access-review outcome), and decisions. Both founders comment/approve. Disclose this structure honestly in the system description.
- Track oversight action items as Linear issues linked from the review ticket, so closure is demonstrable.
- Store each quarter's review packet (dashboard export + minutes) in `evidence/oversight/2026-Q1/` on the compliance-archives branch.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| `governance.md` exists and defines oversight body + cadence | GitHub | `gh api repos/{org}/policies/contents/governance.md`; grep for cadence keyword (`quarterly`) |
| Quarterly oversight review ticket exists and is closed for each quarter in period | Linear API | Query issues labeled `oversight-review`, verify one Done per quarter with closing comment length > threshold |
| Review ticket references required inputs (risk, incidents, monitors, access review) | Linear API | Grep ticket body/comments for required section headings |
| Oversight action items tracked to closure | Linear API | Issues linked from review ticket are Done or have active status with owner |
| Quarterly evidence packet archived | GitHub | File existence check `evidence/oversight/<quarter>/` on `compliance-archives` branch via `gh api` |
| Governance doc reviewed in last 12 months | GitHub | Latest commit date on `governance.md` < 365 days |
| Independence and substance of oversight (did the reviewer actually challenge management?) | MANUAL | Auditor inquiry/interview with advisor or founders |
| Advisor/board-member expertise | MANUAL | Bio review by auditor |

## Evidence artifacts
- `policies/governance.md` — oversight charter (one page), version-controlled.
- `evidence/oversight/<YYYY-QN>/minutes.md` — quarterly review minutes or exported email thread with advisor.
- `evidence/oversight/<YYYY-QN>/security-summary.pdf` — the dashboard/monitor summary presented at the review.
- Linear export of `oversight-review` tickets and their linked action items, archived to `compliance-archives`.
- Advisor agreement or board consent naming security oversight responsibility (if applicable), stored in `evidence/governance/`.
