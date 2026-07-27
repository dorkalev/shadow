---
id: CC6.4
family: CC6 — Logical and Physical Access Controls
category: Security (Common Criteria)
coso: null
title: Physical Access Restriction
weight: 2
automatable: manual
nature: document
---

# CC6.4 — Physical Access Restriction

## Criterion (AICPA TSP Section 100, verbatim)
> The entity restricts physical access to facilities and protected information assets (for example, data center facilities, backup media storage, and other sensitive locations) to authorized personnel to meet the entity's objectives.

## What it means

CC6.4 asks who can physically touch the hardware that holds protected information. For a traditional company that means badge readers, visitor logs, and locked server rooms. For a tiny cloud-native startup, the honest answer is: you do not operate any facility that stores customer data. The data centers belong to Google/Amazon, which are **subservice organizations** — you inherit their physical controls and rely on their SOC 2 reports (the "carve-out" method). Your system description must say this explicitly, list the inherited controls, and show that you actually obtain and review the providers' SOC 2 reports annually (this is a CSOC — complementary subservice organization control — dependency).

What remains in your scope is small but real: employee laptops and any office/home-office environment. Laptops are the one physical asset class that holds credentials and possibly cached data, so the controls are full-disk encryption, screen lock, remote wipe capability, and a device inventory. If you have a WeWork-style office, physical entry is the landlord's badge system; if fully remote, say so — "no facilities in scope beyond endpoints" is an acceptable and common posture, provided the endpoint controls are demonstrable.

Auditors mark this criterion largely "inherited + endpoint controls" for companies like yours. The failure mode is not missing badge readers; it is failing to document the inheritance, not reviewing the cloud provider's SOC 2 report, or having no laptop inventory at all.

## Points of focus (2022 revision, summarized)

Summaries of AICPA points of focus — guidance, not requirements:

- **Creates or modifies physical access** — physical access to facilities/assets is granted based on authorization.
- **Removes physical access** — access is revoked promptly on termination or role change.
- **Reviews physical access** — physical access rights are reviewed periodically.
- **Recovers physical devices** (added in 2022 guidance) — the entity recovers devices such as laptops from departing personnel and maintains a device inventory.

## What the auditor will ask for

- The system description section identifying AWS/GCP as subservice organizations and the physical controls inherited from them (carve-out disclosure).
- Evidence you obtained and reviewed the cloud providers' most recent SOC 2 reports (a dated review memo suffices).
- Device inventory: every laptop, its assigned owner, serial, encryption status.
- MDM report (or signed attestations) showing full-disk encryption, screen lock, and remote-wipe enrollment on all devices.
- Offboarding tickets showing laptop recovery or remote wipe for each departure in the period.
- If an office exists: description of entry controls (building badge, keys) and who holds access; if fully remote, a statement to that effect in the system description.

## How a tiny AI-first startup satisfies it

- **Inherit and document.** System description states: all production infrastructure runs in GCP/AWS regions; physical security of data centers is the responsibility of the subservice organization; the entity reviews the provider's SOC 2 Type II report annually. Download the reports (GCP: Compliance Reports Manager; AWS: Artifact), write a half-page review memo each year noting the report period, opinion, and any relevant exceptions/CUECs.
- **Own the laptops.** Maintain `inventory/devices.yaml`: owner, model, serial, purchase date, encryption, MDM status. Enroll Macs in a lightweight MDM (Kandji, Mosyle, Jamf Now — Mosyle is cheap at this scale) enforcing FileVault, screen lock ≤5 min, and remote wipe. If MDM is deferred, collect signed per-device attestations with FileVault screenshots — honest, but plan to graduate to MDM.
- **Recover devices at offboarding.** Laptop return (or remote wipe for unreturned devices) is a mandatory line on the CC6.2 offboarding checklist, with date recorded.
- **Office (if any).** Coworking badge access rides on the CC6.2 lifecycle tickets; no servers or backup media are ever kept on premises — state this in policy. Fully-remote teams simply scope facilities out and lean on endpoint controls.
- **No physical media.** Policy: no production data on removable media, no local backups on external drives. This also feeds CC6.5 and CC6.7.

## Automated shadow checks

| Check | Source | Method |
|---|---|---|
| Device inventory file exists and every roster member has a device entry | Repo + roster | File check `inventory/devices.yaml`; cross-reference owners vs. `roster.yaml` |
| MDM shows 100% FileVault/encryption compliance | MDM API | Vendor API (Kandji/Mosyle) device list → encryption + screen-lock policy status; MANUAL if no MDM |
| Cloud provider SOC 2 review memo current (≤12 months) | Repo | File-existence + date check `evidence/subservice/gcp-soc2-review-<year>.md` (and AWS equivalent) |
| Subservice org disclosure present in system description | Repo | Grep `system-description.md` for subservice-organization section |
| Departed users' devices recovered/wiped | Linear | Linear API: offboarding issues contain completed "device recovered/wiped" checklist item — MANUAL verification of actual wipe |
| Data center physical controls | Cloud provider | MANUAL/INHERITED — rely on provider SOC 2 report; no CLI check possible |
| Office access list review | Facilities | MANUAL — annual review note if an office exists |

## Evidence artifacts

- `inventory/devices.yaml` — version-controlled device inventory.
- `evidence/endpoints/mdm-report-<date>.csv` — MDM compliance export (or `evidence/endpoints/attestations/` signed statements + screenshots).
- `evidence/subservice/gcp-soc2-<year>.pdf`, `aws-soc2-<year>.pdf` — provider reports (access-restricted), plus `*-review-<year>.md` memos.
- `policies/physical-security.md` — short policy covering remote posture, no-local-media rule, device recovery.
- Offboarding Linear tickets with device-recovery line items, exported under `evidence/lifecycle/`.
