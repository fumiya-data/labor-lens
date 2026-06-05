pub fn mask_log_value(_value: &str, visible_mask_width: usize) -> String {
    "*".repeat(visible_mask_width)
}

pub fn verify_input_fingerprint(
    before_processing: &str,
    after_processing: &str,
) -> Result<(), String> {
    if before_processing == after_processing {
        Ok(())
    } else {
        Err(format!(
            "input fingerprint changed before_processing={before_processing} after_processing={after_processing}"
        ))
    }
}

pub fn source_archive_path(tenant_id: &str, run_id: &str) -> String {
    format!(".laborlens/{tenant_id}/{run_id}/source-archive")
}

pub fn artifact_store_path(tenant_id: &str, run_id: &str) -> String {
    format!(".laborlens/{tenant_id}/{run_id}/artifact-store")
}

#[cfg(test)]
mod tests {
    use crate::shared::ops::{
        artifact_store_path, mask_log_value, source_archive_path, verify_input_fingerprint,
    };

    #[test]
    fn masks_raw_values_before_logging() {
        let masked = mask_log_value("佐藤 花子", 4);

        assert_eq!(masked, "****");
        assert!(!masked.contains("佐藤"));
    }

    #[test]
    fn verifies_input_fingerprint_before_and_after_processing() {
        assert!(verify_input_fingerprint("fnv1a64:abc", "fnv1a64:abc").is_ok());
        assert!(verify_input_fingerprint("fnv1a64:abc", "fnv1a64:def").is_err());
    }

    #[test]
    fn defines_source_archive_and_artifact_store_paths() {
        assert_eq!(
            source_archive_path("tenant-a", "run-001"),
            ".laborlens/tenant-a/run-001/source-archive"
        );
        assert_eq!(
            artifact_store_path("tenant-a", "run-001"),
            ".laborlens/tenant-a/run-001/artifact-store"
        );
    }
}
