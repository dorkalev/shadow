---
title: SOC 2 Auditor Checklist — 2017 Trust Services Criteria (2022 Revised Points of Focus)
source: AICPA TSP Section 100
criteria_count: 61
---

# SOC 2 Auditor Checklist

This is the complete, unmodified list of criteria a CPA firm tests in a SOC 2 examination — **AICPA TSP Section 100, "2017 Trust Services Criteria for Security, Availability, Processing Integrity, Confidentiality, and Privacy" (with Revised Points of Focus — 2022)**. The 2022 revision changed only the points of focus; the criterion IDs and text are unchanged from 2017.

Nothing here is invented. Each criterion's text is verbatim. Each links to an in-depth file in [`criteria/`](criteria/).

**Scope note:** the 33 Common Criteria (CC-series, the Security category) are mandatory in every SOC 2. Availability (A), Confidentiality (C), Processing Integrity (PI), and Privacy (P) apply only if included in the engagement's scope. Total: **61 criteria**.

---

## Security (Common Criteria) — mandatory in every SOC 2

### CC1 — Control Environment

- [ ] **[CC1.1](criteria/CC1.1.md)** — COSO Principle 1: The entity demonstrates a commitment to integrity and ethical values.
- [ ] **[CC1.2](criteria/CC1.2.md)** — COSO Principle 2: The board of directors demonstrates independence from management and exercises oversight of the development and performance of internal control.
- [ ] **[CC1.3](criteria/CC1.3.md)** — COSO Principle 3: Management establishes, with board oversight, structures, reporting lines, and appropriate authorities and responsibilities in the pursuit of objectives.
- [ ] **[CC1.4](criteria/CC1.4.md)** — COSO Principle 4: The entity demonstrates a commitment to attract, develop, and retain competent individuals in alignment with objectives.
- [ ] **[CC1.5](criteria/CC1.5.md)** — COSO Principle 5: The entity holds individuals accountable for their internal control responsibilities in the pursuit of objectives.

### CC2 — Communication and Information

- [ ] **[CC2.1](criteria/CC2.1.md)** — COSO Principle 13: The entity obtains or generates and uses relevant, quality information to support the functioning of internal control.
- [ ] **[CC2.2](criteria/CC2.2.md)** — COSO Principle 14: The entity internally communicates information, including objectives and responsibilities for internal control, necessary to support the functioning of internal control.
- [ ] **[CC2.3](criteria/CC2.3.md)** — COSO Principle 15: The entity communicates with external parties regarding matters affecting the functioning of internal control.

### CC3 — Risk Assessment

- [ ] **[CC3.1](criteria/CC3.1.md)** — COSO Principle 6: The entity specifies objectives with sufficient clarity to enable the identification and assessment of risks relating to objectives.
- [ ] **[CC3.2](criteria/CC3.2.md)** — COSO Principle 7: The entity identifies risks to the achievement of its objectives across the entity and analyzes risks as a basis for determining how the risks should be managed.
- [ ] **[CC3.3](criteria/CC3.3.md)** — COSO Principle 8: The entity considers the potential for fraud in assessing risks to the achievement of objectives.
- [ ] **[CC3.4](criteria/CC3.4.md)** — COSO Principle 9: The entity identifies and assesses changes that could significantly impact the system of internal control.

### CC4 — Monitoring Activities

- [ ] **[CC4.1](criteria/CC4.1.md)** — COSO Principle 16: The entity selects, develops, and performs ongoing and/or separate evaluations to ascertain whether the components of internal control are present and functioning.
- [ ] **[CC4.2](criteria/CC4.2.md)** — COSO Principle 17: The entity evaluates and communicates internal control deficiencies in a timely manner to those parties responsible for taking corrective action, including senior management and the board of directors, as appropriate.

### CC5 — Control Activities

- [ ] **[CC5.1](criteria/CC5.1.md)** — COSO Principle 10: The entity selects and develops control activities that contribute to the mitigation of risks to the achievement of objectives to acceptable levels.
- [ ] **[CC5.2](criteria/CC5.2.md)** — COSO Principle 11: The entity also selects and develops general control activities over technology to support the achievement of objectives.
- [ ] **[CC5.3](criteria/CC5.3.md)** — COSO Principle 12: The entity deploys control activities through policies that establish what is expected and in procedures that put policies into action.

### CC6 — Logical and Physical Access Controls

- [ ] **[CC6.1](criteria/CC6.1.md)** — The entity implements logical access security software, infrastructure, and architectures over protected information assets to protect them from security events to meet the entity's objectives.
- [ ] **[CC6.2](criteria/CC6.2.md)** — Prior to issuing system credentials and granting system access, the entity registers and authorizes new internal and external users whose access is administered by the entity. For those users whose access is administered by the entity, user system credentials are removed when user access is no longer authorized.
- [ ] **[CC6.3](criteria/CC6.3.md)** — The entity authorizes, modifies, or removes access to data, software, functions, and other protected information assets based on roles, responsibilities, or the system design and changes, giving consideration to the concepts of least privilege and segregation of duties, to meet the entity's objectives.
- [ ] **[CC6.4](criteria/CC6.4.md)** — The entity restricts physical access to facilities and protected information assets (for example, data center facilities, backup media storage, and other sensitive locations) to authorized personnel to meet the entity's objectives.
- [ ] **[CC6.5](criteria/CC6.5.md)** — The entity discontinues logical and physical protections over physical assets only after the ability to read or recover data and software from those assets has been diminished and is no longer required to meet the entity's objectives.
- [ ] **[CC6.6](criteria/CC6.6.md)** — The entity implements logical access security measures to protect against threats from sources outside its system boundaries.
- [ ] **[CC6.7](criteria/CC6.7.md)** — The entity restricts the transmission, movement, and removal of information to authorized internal and external users and processes, and protects it during transmission, movement, or removal to meet the entity's objectives.
- [ ] **[CC6.8](criteria/CC6.8.md)** — The entity implements controls to prevent or detect and act upon the introduction of unauthorized or malicious software to meet the entity's objectives.

### CC7 — System Operations

- [ ] **[CC7.1](criteria/CC7.1.md)** — To meet its objectives, the entity uses detection and monitoring procedures to identify (1) changes to configurations that result in the introduction of new vulnerabilities, and (2) susceptibilities to newly discovered vulnerabilities.
- [ ] **[CC7.2](criteria/CC7.2.md)** — The entity monitors system components and the operation of those components for anomalies that are indicative of malicious acts, natural disasters, and errors affecting the entity's ability to meet its objectives; anomalies are analyzed to determine whether they represent security events.
- [ ] **[CC7.3](criteria/CC7.3.md)** — The entity evaluates security events to determine whether they could or have resulted in a failure of the entity to meet its objectives (security incidents) and, if so, takes actions to prevent or address such failures.
- [ ] **[CC7.4](criteria/CC7.4.md)** — The entity responds to identified security incidents by executing a defined incident-response program to understand, contain, remediate, and communicate security incidents, as appropriate.
- [ ] **[CC7.5](criteria/CC7.5.md)** — The entity identifies, develops, and implements activities to recover from identified security incidents.

### CC8 — Change Management

- [ ] **[CC8.1](criteria/CC8.1.md)** — The entity authorizes, designs, develops or acquires, configures, documents, tests, approves, and implements changes to infrastructure, data, software, and procedures to meet its objectives.

### CC9 — Risk Mitigation

- [ ] **[CC9.1](criteria/CC9.1.md)** — The entity identifies, selects, and develops risk mitigation activities for risks arising from potential business disruptions.
- [ ] **[CC9.2](criteria/CC9.2.md)** — The entity assesses and manages risks associated with vendors and business partners.

---

## Availability — in scope only if selected

- [ ] **[A1.1](criteria/A1.1.md)** — The entity maintains, monitors, and evaluates current processing capacity and use of system components (infrastructure, data, and software) to manage capacity demand and to enable the implementation of additional capacity to help meet its objectives.
- [ ] **[A1.2](criteria/A1.2.md)** — The entity authorizes, designs, develops or acquires, implements, operates, approves, maintains, and monitors environmental protections, software, data backup processes, and recovery infrastructure to meet its objectives.
- [ ] **[A1.3](criteria/A1.3.md)** — The entity tests recovery plan procedures supporting system recovery to meet its objectives.

## Confidentiality — in scope only if selected

- [ ] **[C1.1](criteria/C1.1.md)** — The entity identifies and maintains confidential information to meet the entity's objectives related to confidentiality.
- [ ] **[C1.2](criteria/C1.2.md)** — The entity disposes of confidential information to meet the entity's objectives related to confidentiality.

## Processing Integrity — in scope only if selected

- [ ] **[PI1.1](criteria/PI1.1.md)** — The entity obtains or generates, uses, and communicates relevant, quality information regarding the objectives related to processing, including definitions of data processed and product and service specifications, to support the use of products and services.
- [ ] **[PI1.2](criteria/PI1.2.md)** — The entity implements policies and procedures over system inputs, including controls over completeness and accuracy, to result in products, services, and reporting to meet the entity's objectives.
- [ ] **[PI1.3](criteria/PI1.3.md)** — The entity implements policies and procedures over system processing to result in products, services, and reporting to meet the entity's objectives.
- [ ] **[PI1.4](criteria/PI1.4.md)** — The entity implements policies and procedures to make available or deliver output completely, accurately, and timely in accordance with specifications to meet the entity's objectives.
- [ ] **[PI1.5](criteria/PI1.5.md)** — The entity implements policies and procedures to store inputs, items in processing, and outputs completely, accurately, and timely in accordance with system specifications to meet the entity's objectives.

## Privacy — in scope only if selected

### P1 — Notice and Communication of Objectives

- [ ] **[P1.1](criteria/P1.1.md)** — The entity provides notice to data subjects about its privacy practices to meet the entity's objectives related to privacy. The notice is updated and communicated to data subjects in a timely manner for changes to the entity's privacy practices, including changes in the use of personal information, to meet the entity's objectives related to privacy.

### P2 — Choice and Consent

- [ ] **[P2.1](criteria/P2.1.md)** — The entity communicates choices available regarding the collection, use, retention, disclosure, and disposal of personal information to the data subjects and the consequences, if any, of each choice. Explicit consent for the collection, use, retention, disclosure, and disposal of personal information is obtained from data subjects or other authorized persons, if required. Such consent is obtained only for the intended purpose of the information to meet the entity's objectives related to privacy. The entity's basis for determining implicit consent for the collection, use, retention, disclosure, and disposal of personal information is documented.

### P3 — Collection

- [ ] **[P3.1](criteria/P3.1.md)** — Personal information is collected consistent with the entity's objectives related to privacy.
- [ ] **[P3.2](criteria/P3.2.md)** — For information requiring explicit consent, the entity communicates the need for such consent as well as the consequences of a failure to provide consent for the request for personal information and obtains the consent prior to the collection of the information to meet the entity's objectives related to privacy.

### P4 — Use, Retention, and Disposal

- [ ] **[P4.1](criteria/P4.1.md)** — The entity limits the use of personal information to the purposes identified in the entity's objectives related to privacy.
- [ ] **[P4.2](criteria/P4.2.md)** — The entity retains personal information consistent with the entity's objectives related to privacy.
- [ ] **[P4.3](criteria/P4.3.md)** — The entity securely disposes of personal information to meet the entity's objectives related to privacy.

### P5 — Access

- [ ] **[P5.1](criteria/P5.1.md)** — The entity grants identified and authenticated data subjects the ability to access their stored personal information for review and, upon request, provides physical or electronic copies of that information to data subjects to meet the entity's objectives related to privacy. If access is denied, data subjects are informed of the denial and reason for such denial, as required, to meet the entity's objectives related to privacy.
- [ ] **[P5.2](criteria/P5.2.md)** — The entity corrects, amends, or appends personal information based on information provided by data subjects and communicates such information to third parties, as committed or required, to meet the entity's objectives related to privacy. If a request for correction is denied, data subjects are informed of the denial and reason for such denial to meet the entity's objectives related to privacy.

### P6 — Disclosure and Notification

- [ ] **[P6.1](criteria/P6.1.md)** — The entity discloses personal information to third parties with the explicit consent of data subjects and such consent is obtained prior to disclosure to meet the entity's objectives related to privacy.
- [ ] **[P6.2](criteria/P6.2.md)** — The entity creates and retains a complete, accurate, and timely record of authorized disclosures of personal information to meet the entity's objectives related to privacy.
- [ ] **[P6.3](criteria/P6.3.md)** — The entity creates and retains a complete, accurate, and timely record of detected or reported unauthorized disclosures (including breaches) of personal information to meet the entity's objectives related to privacy.
- [ ] **[P6.4](criteria/P6.4.md)** — The entity obtains privacy commitments from vendors and other third parties who have access to personal information to meet the entity's objectives related to privacy. The entity assesses those parties' compliance on a periodic and as-needed basis and takes corrective action, if necessary.
- [ ] **[P6.5](criteria/P6.5.md)** — The entity obtains commitments from vendors and other third parties with access to personal information to notify the entity in the event of actual or suspected unauthorized disclosures of personal information. Such notifications are reported to appropriate personnel and acted on in accordance with established incident-response procedures to meet the entity's objectives related to privacy.
- [ ] **[P6.6](criteria/P6.6.md)** — The entity provides notification of breaches and incidents to affected data subjects, regulators, and others to meet the entity's objectives related to privacy.
- [ ] **[P6.7](criteria/P6.7.md)** — The entity provides data subjects with an accounting of the personal information held and disclosure of the data subjects' personal information, upon the data subjects' request, to meet the entity's objectives related to privacy.

### P7 — Quality

- [ ] **[P7.1](criteria/P7.1.md)** — The entity collects and maintains accurate, up-to-date, complete, and relevant personal information to meet the entity's objectives related to privacy.

### P8 — Monitoring and Enforcement

- [ ] **[P8.1](criteria/P8.1.md)** — The entity implements a process for receiving, addressing, resolving, and communicating the resolution of inquiries, complaints, and disputes from data subjects and others and periodically monitors compliance to meet the entity's objectives related to privacy. Corrections and other necessary actions related to identified deficiencies are made or taken in a timely manner.

---

## Beyond the criteria: what else the auditor checks

A SOC 2 report is not only the TSC checklist. The auditor also evaluates the **system description** against the AICPA **Description Criteria (DC section 200)** — DC1 types of services, DC2 principal service commitments and system requirements, DC3 components of the system, DC4 system incidents, DC5 applicable trust services criteria and related controls, DC6 complementary user entity controls (CUECs), DC7 complementary subservice organization controls (CSOCs). See [docs/what-is-soc2.md](docs/what-is-soc2.md).
