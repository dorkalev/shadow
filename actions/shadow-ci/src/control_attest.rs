//! Record evidence for a human-operated control without an LLM.
use serde_json::json;

fn required(key: &str) -> Result<String, String> {
    std::env::var(key).map_err(|_| format!("{key} is required"))
}

pub fn run_control_attest() -> Result<i32, String> {
    let criterion = required("CRITERION_ID")?;
    let attested_by = required("ATTESTED_BY")?;
    let note = required("ATTESTATION_NOTE")?;
    let expires_at = required("EXPIRES_AT")?;
    let procedure_id = std::env::var("PROCEDURE_ID").ok();
    let evidence_link = std::env::var("EVIDENCE_LINK").ok();
    let attested_at = crate::util::utc_date("%Y-%m-%dT%H:%M:%SZ");
    if expires_at <= attested_at {
        return Err("EXPIRES_AT must be a future ISO-8601 UTC timestamp".into());
    }
    if !criterion
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.')
    {
        return Err("CRITERION_ID contains invalid characters".into());
    }

    let body = json!({
        "schema_version": 1,
        "criterion": criterion,
        "procedure_id": procedure_id,
        "note": note,
        "evidence_link": evidence_link,
        "attested_by": attested_by,
        "attested_at": attested_at,
        "expires_at": expires_at
    });
    let dir = std::path::Path::new("evidence/attestations");
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let date = crate::util::utc_date("%F");
    let path = dir.join(format!(
        "{}-{date}.json",
        body["criterion"].as_str().unwrap()
    ));
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&body).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!("recorded {}", path.display());
    Ok(0)
}

#[cfg(test)]
mod tests {
    #[test]
    fn criterion_filename_alphabet_is_path_safe() {
        assert!("CC6.4"
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.'));
        assert!(!"../../secret"
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.'));
    }
}
