---
id: CC6.5
family: CC6 — Logical and Physical Access Controls
category: Security (Common Criteria)
coso: null
title: Secure Asset Disposal
weight: 2
automatable: manual
nature: document
---

# CC6.5 — Secure Asset Disposal

## Criterion (AICPA TSP Section 100, verbatim)
> The entity discontinues logical and physical protections over physical assets only after the ability to read or recover data and software from those assets has been diminished and is no longer required to meet the entity's objectives.

## What it means

CC6.5 is the disposal criterion: before a physical asset (laptop, phone, drive, server) leaves your protection — sold, recycled, returned, thrown away — the data on it must be unrecoverable. You cannot stop protecting something that still holds readable customer data or secrets.

For a cloud-native micro-startup, the server-side half is again inherited: when you delete a disk or a Cloud Storage object, GCP/AWS handle media sanitization and eventual physical destruction, and their SOC 2 reports cover it. Your system description points to the subservice organization, same as CC6.4. What is genuinely yours is endpoint disposal: the laptop of a departed engineer, a dead MacBook being traded in, a founder's old phone that had Slack and Gmail on it. Full-disk encryption makes this tractable — cryptographic erasure (wiping the key via "Erase All Content and Settings" or an MDM wipe) is an accepted sanitization method for FileVault-encrypted APFS volumes, so the control is: encrypt from day one, wipe before disposal, record it.

Auditors test this by asking for the disposal log. If no assets were disposed of during the period, that is a fine answer — say so, and show the procedure exists for when it happens. The failure mode is having sold an old laptop on eBay with no record of a wipe.

## Points of focus (2022 revision, summarized)

Summaries of AICPA points of focus — guidance, not requirements:

- **Identifies data and software for disposal** — assets/media containing protected data are identified when they are to be disposed of, resold, or repurposed.
- **Removes data and software from physical assets** — data and software are erased/sanitized (or the media destroyed) using appropriate techniques before protections are discontinued.

## What the auditor will ask for

- The asset disposal procedure (usually a section of the endpoint/physical-security policy).
- Disposal log for the period: which devices were wiped/destroyed, when, by whom, method used — or a written statement that no disposals occurred.
- For sampled disposals: corroborating evidence (MDM wipe command log, Apple trade-in confirmation, recycler certificate of destruction).
- Proof of full-disk encryption on the fleet (makes cryptographic erasure valid) — reuses the CC6.4 MDM report.
- System description language delegating server-side media sanitization to GCP/AWS as subservice organizations, plus the provider SOC 2 review.
- Offboarding tickets showing what happened to leavers' devices (reissued after wipe, or disposed).

## How a tiny AI-first startup satisfies it

- **Encrypt everything on day one** (CC6.4 control) so every later disposal reduces to a key wipe.
- **A five-line disposal procedure** in `policies/physical-security.md`: before any device is sold, recycled, returned, or reassigned — (1) confirm FileVault was enabled, (2) issue MDM remote wipe or "Erase All Content and Settings", (3) remove device from MDM and inventory, (4) log it in `inventory/disposals.yaml` with date, serial, method, operator, (5) attach proof (screenshot / MDM log / recycler certificate).
- **Reassignment counts.** Wipe-and-reinstall between users; a handed-down laptop with the old user's data is a CC6.5 failure even though the device never left the company.
- **Cloud media: inherit.** Policy states media sanitization for cloud infrastructure is inherited from GCP/AWS; deleting resources through the API is the entity's disposal action. No self-hosted servers, no backup tapes, no external drives with production data (prohibited by policy).
- **Unreturned devices.** If a leaver never returns a laptop, the MDM remote wipe is the disposal event — log it the same way. This is a strong argument for MDM even at 4 people.

## Automated shadow checks

| Check | Source | Method |
|---|---|---|
| Disposal log exists (or explicit "none this period" entry) | Repo | File-existence + schema check `inventory/disposals.yaml` |
| Every inventory device marked `disposed` has a disposal-log entry with method + proof reference | Repo | Cross-reference `inventory/devices.yaml` status vs. `disposals.yaml` |
| No disposed/retired device still enrolled in MDM | MDM API | Vendor API device list vs. inventory `disposed` entries; MANUAL if no MDM |
| Fleet encryption at 100% (prerequisite for crypto-erase) | MDM API | Reuse CC6.4 encryption compliance check |
| Disposal procedure documented | Repo | Grep `policies/physical-security.md` for disposal section |
| Wipe actually executed for sampled disposals | MDM / receipts | MANUAL — review MDM wipe command history or recycler certificate |
| Cloud media sanitization | Cloud provider | MANUAL/INHERITED — covered by provider SOC 2 report review (CC6.4 memo) |

## Evidence artifacts

- `inventory/disposals.yaml` — append-only disposal log (date, serial, owner, method, operator, proof link); git history proves entries weren't backfilled.
- `evidence/endpoints/wipes/<serial>-<date>.png` — MDM wipe confirmation screenshots or command logs.
- Recycler/trade-in certificates of destruction in `evidence/endpoints/wipes/`.
- `policies/physical-security.md` — disposal procedure section, version-controlled.
- `evidence/subservice/*-soc2-review-<year>.md` — provider report review memos (shared with CC6.4) covering media sanitization.
