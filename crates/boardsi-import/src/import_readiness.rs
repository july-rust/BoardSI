//! E_008: bounded PADS PCB format/version classification and evidence-limited readiness.
//! This is NOT a full CAD import qualification or authorization to run SI.
use crate::{probe_pads_header, route_preview, Signature};
use boardsi_model::CapabilityState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadinessReport {
    pub source_format: &'static str,
    pub version: &'static str,
    pub format_status: &'static str,
    pub parser_status: &'static str,
    pub failure_code: &'static str,
    pub geometry: &'static str,
    pub net: &'static str,
    pub pin_pad: &'static str,
    pub stackup: &'static str,
    pub reference: &'static str,
    pub model: &'static str,
    pub skipped_records: u32,
    pub preview_subset_verified: bool,
}

impl ReadinessReport {
    fn unknown() -> Self {
        Self {
            source_format: "UNKNOWN",
            version: "UNKNOWN",
            format_status: "UNKNOWN_OR_UNSUPPORTED",
            parser_status: "NOT_RUN",
            failure_code: "UNRECOGNIZED_SOURCE",
            geometry: "UNKNOWN",
            net: "UNKNOWN",
            pin_pad: "UNKNOWN",
            stackup: "UNKNOWN",
            reference: "UNKNOWN",
            model: "UNKNOWN",
            skipped_records: 0,
            preview_subset_verified: false,
        }
    }

    /// Stable, untrusted-input-free diagnostic fields (no paths, net names or source lines).
    pub fn diagnostic(&self) -> String {
        format!(
            "format={};version={};format_status={};parser_status={};reason={};G={};N={};P={};S={};R={};M={};skipped={};preview_subset_verified={};solver_ready=false;physical_si=NOT_RUN",
            self.source_format,
            self.version,
            self.format_status,
            self.parser_status,
            self.failure_code,
            self.geometry,
            self.net,
            self.pin_pad,
            self.stackup,
            self.reference,
            self.model,
            self.skipped_records,
            self.preview_subset_verified,
        )
    }
}

fn capability(state: CapabilityState) -> &'static str {
    match state {
        CapabilityState::Complete => "COMPLETE",
        CapabilityState::Partial => "PARTIAL",
        CapabilityState::Missing => "MISSING",
        CapabilityState::Conflicting => "CONFLICTING",
    }
}

/// Source bytes are intentionally limited to the existing restricted PADS parser budget.
/// Signature recognition alone cannot prove a supported revision or object coverage.
pub fn inspect_pads_readiness(source: &[u8]) -> ReadinessReport {
    let mut report = ReadinessReport::unknown();
    if source.len() > route_preview::MAX_FILE_BYTES {
        report.parser_status = "REJECTED";
        report.failure_code = "INPUT_TOO_LARGE";
        return report;
    }
    match probe_pads_header(source) {
        Signature::Unknown => return report,
        Signature::PadsSchematicAscii => {
            report.source_format = "PADS_SCHEMATIC_ASCII";
            report.format_status = "PCB_IMPORT_UNSUPPORTED";
            report.failure_code = "SCHEMATIC_NOT_PCB";
            return report;
        }
        Signature::PadsPcbAscii => {
            report.source_format = "PADS_PCB_ASCII";
        }
    }
    // The existing restricted parser accepts ONLY four literal ASCII V9.4-MILS
    // header tokens. A PADS-family signature with another tag is not an import
    // qualification. In particular, UTF-8 BOM, '*' wrappers and other versions
    // cannot silently bypass the parser's exact source grammar.
    let first = source
        .split(|&byte| byte == b'\n')
        .next()
        .unwrap_or_default();
    let tag = first
        .split(|b| b.is_ascii_whitespace())
        .next()
        .unwrap_or_default();
    const SUPPORTED_HEADERS: &[&[u8]] = &[
        b"!PADS-POWERPCB-V9.4-MILS!",
        b"!PADS-POWERPCB-V9.4-MILS-CP1252!",
        b"!PADS-POWERPCB-V9.4-MILS-250L!",
        b"!PADS-POWERPCB-V9.4-MILS-250L-CP1252!",
    ];
    let supported = SUPPORTED_HEADERS.contains(&tag);
    if !supported {
        report.version = "UNQUALIFIED";
        report.format_status = "FAMILY_ONLY";
        report.failure_code = "UNSUPPORTED_REVISION_OR_HEADER_VARIANT";
        return report;
    }
    report.version = "V9.4-MILS";
    report.format_status = "SUPPORTED_HEADER_SUBSET";
    // This is an opt-in bounded source inspection. No default probe behavior changes.
    let parsed = match route_preview::parse(source) {
        Ok(p) => p,
        Err(reason) => {
            report.parser_status = "REJECTED";
            report.failure_code = reason;
            return report;
        }
    };
    let board = match route_preview::into_canonical(source, &parsed) {
        Ok(b) => b,
        Err(reason) => {
            report.parser_status = "REJECTED";
            report.failure_code = reason;
            return report;
        }
    };
    let Some(src) = board.sources.first() else {
        report.parser_status = "REJECTED";
        report.failure_code = "NO_CANONICAL_SOURCE";
        return report;
    };
    // States come from the REAL restricted Canonical adapter, not the header.
    // Capabilities are never promoted to COMPLETE by this diagnostic.
    let c = &src.capability;
    report.geometry = capability(c.geometry);
    report.net = capability(c.net);
    report.pin_pad = capability(c.pin_pad);
    report.stackup = capability(c.stackup);
    report.reference = capability(c.reference);
    report.model = capability(c.model);
    report.skipped_records = parsed.skipped;
    report.parser_status = if parsed.skipped > 0 {
        "PARTIAL_WITH_SKIPPED_SOURCE_RECORDS"
    } else {
        "RESTRICTED_CANONICAL_PARTIAL"
    };
    report.failure_code = if parsed.skipped > 0 {
        "UNSUPPORTED_OBJECTS_SKIPPED"
    } else {
        "NONE"
    };
    report.preview_subset_verified = parsed.skipped == 0;
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n300 200 1 10 0\n*END*\n";

    #[test]
    fn accepted_restricted_source_is_partial_not_solver_ready() {
        let r = inspect_pads_readiness(VALID.as_bytes());
        assert_eq!(r.source_format, "PADS_PCB_ASCII");
        assert_eq!(r.version, "V9.4-MILS");
        assert_eq!(r.geometry, "PARTIAL");
        assert_eq!(r.parser_status, "RESTRICTED_CANONICAL_PARTIAL");
        assert!(r.preview_subset_verified);
        assert!(r
            .diagnostic()
            .contains("solver_ready=false;physical_si=NOT_RUN"));
        assert!(!r.diagnostic().contains("NET_A"));
        assert!(!r.diagnostic().contains("G=COMPLETE"));
    }

    #[test]
    fn recognized_future_revision_is_not_supported() {
        for replacement in ["V9.5-MILS", "V9.4-MM", "V9.4-MILS-CP932"] {
            let s = VALID.replace("V9.4-MILS", replacement);
            let r = inspect_pads_readiness(s.as_bytes());
            assert_eq!(r.source_format, "PADS_PCB_ASCII");
            assert_eq!(r.format_status, "FAMILY_ONLY");
            assert_eq!(r.parser_status, "NOT_RUN");
            assert_eq!(r.geometry, "UNKNOWN");
            assert!(!r.preview_subset_verified);
        }
    }

    #[test]
    fn schematic_not_a_pcb_even_if_named_asc() {
        let r = inspect_pads_readiness(b"!PADS-POWERLOGIC-V9.4!\n*PCB*\n");
        assert_eq!(r.failure_code, "SCHEMATIC_NOT_PCB");
        assert_eq!(r.geometry, "UNKNOWN");
    }

    #[test]
    fn fake_headers_and_non_pads_asc_never_promote_geometry() {
        for s in [
            b"*VERSION* 4.0\nSHEET 1\n".as_slice(),
            b"junk\n!PADS-POWERPCB-V9.4-MILS!\n",
            b"!PADS-POWERPCBEVIL-V9.4-MILS!\n",
            b"!PADS-POWERPCB-V9.4-MILS!\rX\n",
            b"!PADS-POWERPCB-V9.4-MILS".as_slice(),
        ] {
            let r = inspect_pads_readiness(s);
            assert_eq!(r.source_format, "UNKNOWN");
            assert_eq!(r.geometry, "UNKNOWN");
        }
    }

    #[test]
    fn incomplete_file_has_no_claimed_capabilities() {
        let s = VALID.replace("*END*\n", "");
        let r = inspect_pads_readiness(s.as_bytes());
        assert_eq!(r.parser_status, "REJECTED");
        assert_eq!(r.failure_code, "MISSING_END_MARKER");
        assert_eq!(r.geometry, "UNKNOWN");
        assert!(!r.preview_subset_verified);
    }

    #[test]
    fn source_size_budget_rejects_before_parsing() {
        let s = vec![b'A'; route_preview::MAX_FILE_BYTES + 1];
        let r = inspect_pads_readiness(&s);
        assert_eq!(r.failure_code, "INPUT_TOO_LARGE");
        assert_eq!(r.parser_status, "REJECTED");
        assert!(!r.preview_subset_verified);
    }

    #[test]
    fn exact_supported_header_variants_and_bom_boundary() {
        for variant in [
            "!PADS-POWERPCB-V9.4-MILS!",
            "!PADS-POWERPCB-V9.4-MILS-CP1252!",
            "!PADS-POWERPCB-V9.4-MILS-250L!",
            "!PADS-POWERPCB-V9.4-MILS-250L-CP1252!",
        ] {
            let r = inspect_pads_readiness(
                VALID
                    .replace("!PADS-POWERPCB-V9.4-MILS!", variant)
                    .as_bytes(),
            );
            assert_eq!(r.parser_status, "RESTRICTED_CANONICAL_PARTIAL");
        }
        let s = format!("\u{feff}{VALID}");
        let r = inspect_pads_readiness(s.as_bytes());
        assert_eq!(r.format_status, "FAMILY_ONLY");
        assert!(!r.preview_subset_verified);
    }

    #[test]
    fn unsupported_or_skipped_route_is_never_a_clean_preview() {
        // A defined unsupported route flag is a visible partial loss.
        let s = VALID.replace("300 200 1 10 0", "300 200 1 10 1");
        let r = inspect_pads_readiness(s.as_bytes());
        assert!(!r.preview_subset_verified);
        assert_ne!(r.parser_status, "RESTRICTED_CANONICAL_PARTIAL");
        assert!(r.diagnostic().contains("solver_ready=false"));
    }

    #[test]
    fn no_untrusted_source_tokens_in_diagnostic_even_on_failure() {
        let s = VALID.replace("NET_A", "SECRET_CUSTOMER_NET");
        let r = inspect_pads_readiness(s.as_bytes());
        assert!(!r.diagnostic().contains("SECRET_CUSTOMER_NET"));
    }

    #[test]
    fn repeated_results_are_deterministic() {
        let a = inspect_pads_readiness(VALID.as_bytes());
        let b = inspect_pads_readiness(VALID.as_bytes());
        assert_eq!(a, b);
    }
}
