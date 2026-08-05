# Expiring control attestations

Use an attestation only when a control requires human judgment and no stronger
system evidence exists. It is a signed, time-bounded observation—not a permanent
green checkbox.

Example:

```sh
CRITERION_ID=CC6.4 \
PROCEDURE_ID=vendor-register \
ATTESTED_BY=security-owner@example.com \
ATTESTATION_NOTE='Reviewed Google Cloud SOC 2 report and bridge letter' \
EVIDENCE_LINK='internal://vendors/google-cloud/2026-review' \
EXPIRES_AT=2027-08-01T00:00:00Z \
.shadow/ci/target/release/shadow-ci control-attest
```

Commit the generated JSON through the normal pull-request gates. The verifier
uses only the latest attestation per criterion and fails it after its expiry.
