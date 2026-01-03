//! Integration tests for Platform Certification System
//!
//! Test suite for console platform certification requirements checking

use game_engine::platform::console::ConsolePlatform;
use game_engine::platform::console::certification::*;

#[test]
fn test_certification_system_basic() {
    // Test that we can create a certification checker
    let checker = CertificationChecker::new(ConsolePlatform::PlayStation5);
    assert_eq!(checker.platform, ConsolePlatform::PlayStation5);

    // Test that we can run certification checks
    let report = checker.check_all();
    assert!(!report.requirements.is_empty());
}

#[test]
fn test_nintendo_switch_requirements() {
    let checker = CertificationChecker::new(ConsolePlatform::NintendoSwitch);
    let report = checker.check_all();

    // Verify platform-specific requirements
    assert!(report.requirements.iter().any(|r| r.id == "switch_save_limits"));
    assert!(report.requirements.iter().any(|r| r.id == "joycon_pairing"));
    assert!(report.requirements.iter().any(|r| r.id == "dock_mode"));
    assert!(report.requirements.iter().any(|r| r.id == "screenshot"));

    // Verify common requirements are also included
    assert!(report.requirements.iter().any(|r| r.id == "achievements"));
    assert!(report.requirements.iter().any(|r| r.id == "cloud_save"));
}

#[test]
fn test_playstation_requirements() {
    let checker = CertificationChecker::new(ConsolePlatform::PlayStation5);
    let report = checker.check_all();

    // Check for PlayStation-specific requirements
    assert!(report.requirements.iter().any(|r| r.id == "trophy_icons"));
    assert!(report.requirements.iter().any(|r| r.id == "psn_integration"));
    assert!(report.requirements.iter().any(|r| r.id == "dualsense_feedback"));
    assert!(report.requirements.iter().any(|r| r.id == "party_chat"));

    // Verify PS5-specific requirement
    assert!(report.requirements.iter().any(|r| r.id == "activity_cards"));
}

#[test]
fn test_xbox_requirements() {
    let checker = CertificationChecker::new(ConsolePlatform::XboxSeries);
    let report = checker.check_all();

    // Check for Xbox-specific requirements
    assert!(report.requirements.iter().any(|r| r.id == "achievement_icons"));
    assert!(report.requirements.iter().any(|r| r.id == "xbox_live_integration"));
    assert!(report.requirements.iter().any(|r| r.id == "gamerscore"));
    assert!(report.requirements.iter().any(|r| r.id == "quick_resume"));
    assert!(report.requirements.iter().any(|r| r.id == "smart_delivery"));
}

#[test]
fn test_requirement_categories() {
    let checker = CertificationChecker::new(ConsolePlatform::NintendoSwitch);
    let report = checker.check_all();

    // Check requirements are properly categorized
    let ui_reqs = report.requirements_by_category(CertCategory::UserInterface);
    let online_reqs = report.requirements_by_category(CertCategory::Online);
    let storage_reqs = report.requirements_by_category(CertCategory::Storage);
    let input_reqs = report.requirements_by_category(CertCategory::Input);
    let performance_reqs = report.requirements_by_category(CertCategory::Performance);
    let platform_reqs = report.requirements_by_category(CertCategory::PlatformIntegration);
    let accessibility_reqs = report.requirements_by_category(CertCategory::Accessibility);

    assert!(!ui_reqs.is_empty(), "UI requirements should not be empty");
    assert!(
        !online_reqs.is_empty(),
        "Online requirements should not be empty"
    );
    assert!(
        !storage_reqs.is_empty(),
        "Storage requirements should not be empty"
    );
    assert!(
        !input_reqs.is_empty(),
        "Input requirements should not be empty"
    );
    assert!(
        !performance_reqs.is_empty(),
        "Performance requirements should not be empty"
    );
    assert!(
        !platform_reqs.is_empty(),
        "Platform integration requirements should not be empty"
    );
    assert!(
        !accessibility_reqs.is_empty(),
        "Accessibility requirements should not be empty"
    );
}

#[test]
fn test_report_generation() {
    let checker = CertificationChecker::new(ConsolePlatform::PlayStation4);
    let report = checker.check_all();

    // Test text report generation
    let text_report = report.generate_report();
    assert!(text_report.contains("Console Certification Report"));
    assert!(text_report.contains("PlayStation 4"));
    assert!(text_report.contains("SUMMARY"));
    assert!(text_report.contains("Total Requirements"));

    // Test JSON report generation
    let json_report = report.to_json();
    assert!(json_report.is_ok());
    let json = json_report.unwrap();
    assert!(json.contains("\"platform\""));
    assert!(json.contains("\"requirements\""));
    assert!(json.contains("\"overall_passed\""));

    // Test HTML report generation
    let html_report = report.to_html();
    assert!(html_report.contains("<!DOCTYPE html>"));
    assert!(html_report.contains("<title>Console Certification Report</title>"));
    assert!(html_report.contains("summary-card"));
    assert!(html_report.contains("class='category'"));
}

#[test]
fn test_requirement_severity_levels() {
    let req_critical = CertRequirement::new(
        "test_critical",
        CertCategory::Performance,
        "Critical Test",
        "A critical requirement",
        true,
        Severity::Critical,
    );

    let req_major = CertRequirement::new(
        "test_major",
        CertCategory::Performance,
        "Major Test",
        "A major requirement",
        true,
        Severity::Major,
    );

    let req_minor = CertRequirement::new(
        "test_minor",
        CertCategory::Performance,
        "Minor Test",
        "A minor requirement",
        true,
        Severity::Minor,
    );

    assert_eq!(req_critical.severity, Severity::Critical);
    assert_eq!(req_major.severity, Severity::Major);
    assert_eq!(req_minor.severity, Severity::Minor);
}

#[test]
fn test_requirement_builder_pattern() {
    let req = CertRequirement::new(
        "builder_test",
        CertCategory::Input,
        "Builder Pattern Test",
        "Testing the builder pattern",
        true,
        Severity::Major,
    )
    .with_notes("These are test notes")
    .with_reference("https://example.com/test")
    .mark_passed();

    assert_eq!(req.notes, "These are test notes");
    assert_eq!(
        req.reference_url,
        Some("https://example.com/test".to_string())
    );
    assert!(req.passed);
}

#[test]
fn test_pass_rate_calculation() {
    let checker = CertificationChecker::new(ConsolePlatform::XboxOne);
    let report = checker.check_all();

    let pass_rate = report.pass_rate();
    assert!(pass_rate >= 0.0);
    assert!(pass_rate <= 100.0);

    // Calculate expected pass rate manually
    let passed_count = report.passed_requirements().len();
    let total_count = report.requirements.len();
    let expected_rate = (passed_count as f64 / total_count as f64) * 100.0;

    assert!((pass_rate - expected_rate).abs() < 0.01);
}

#[test]
fn test_failed_and_passed_requirements() {
    let checker = CertificationChecker::new(ConsolePlatform::PlayStation5);
    let report = checker.check_all();

    let failed = report.failed_requirements();
    let passed = report.passed_requirements();

    // Both should be non-empty in simulation
    assert!(!report.requirements.is_empty());

    // Failed should only contain required items that failed
    for req in failed {
        assert!(!req.passed);
        assert!(req.required);
    }

    // Passed should contain all items that passed
    for req in passed {
        assert!(req.passed);
    }
}

#[test]
fn test_critical_failures() {
    let checker = CertificationChecker::new(ConsolePlatform::NintendoSwitch);
    let report = checker.check_all();

    let critical = report.critical_failures();

    for req in critical {
        assert!(!req.passed, "Critical failures must be failed");
        assert!(req.required, "Critical failures must be required");
        assert_eq!(
            req.severity,
            Severity::Critical,
            "Must have critical severity"
        );
    }
}

#[test]
fn test_custom_rules() {
    let mut checker = CertificationChecker::new(ConsolePlatform::PlayStation5);

    // Add a custom rule
    let custom_req = CertRequirement::new(
        "custom_rule_test",
        CertCategory::Performance,
        "Custom Rule Test",
        "This is a custom certification rule",
        true,
        Severity::Major,
    )
    .with_notes("Custom test rule")
    .mark_passed();

    checker.add_custom_rule(custom_req);
    let report = checker.check_all();

    // Verify custom rule is included
    assert!(report.requirements.iter().any(|r| r.id == "custom_rule_test"));

    let custom_rule = report
        .requirements
        .iter()
        .find(|r| r.id == "custom_rule_test")
        .expect("Custom rule should be in report");

    assert!(custom_rule.passed);
    assert_eq!(custom_rule.notes, "Custom test rule");
}

#[test]
fn test_report_metadata() {
    let metadata = ReportMetadata::default();

    assert!(!metadata.engine_version.is_empty());
    assert!(!metadata.test_runner_version.is_empty());
    assert_eq!(metadata.test_environment, "development");
}

#[test]
fn test_config_defaults() {
    let config = CertCheckerConfig::default();

    assert!(config.include_optional);
    assert_eq!(config.severity_threshold, Severity::Info);
    assert!(!config.verbose);
    assert!(!config.save_reports);
    assert!(config.output_dir.is_none());
}

#[test]
fn test_category_display() {
    assert_eq!(CertCategory::UserInterface.to_string(), "User Interface");
    assert_eq!(CertCategory::Online.to_string(), "Online");
    assert_eq!(CertCategory::Storage.to_string(), "Storage");
    assert_eq!(CertCategory::Input.to_string(), "Input");
    assert_eq!(CertCategory::Performance.to_string(), "Performance");
    assert_eq!(
        CertCategory::PlatformIntegration.to_string(),
        "Platform Integration"
    );
    assert_eq!(CertCategory::Security.to_string(), "Security");
    assert_eq!(CertCategory::Accessibility.to_string(), "Accessibility");
    assert_eq!(CertCategory::Legal.to_string(), "Legal");
}

#[test]
fn test_all_platforms() {
    let platforms = [
        ConsolePlatform::NintendoSwitch,
        ConsolePlatform::PlayStation5,
        ConsolePlatform::PlayStation4,
        ConsolePlatform::XboxSeries,
        ConsolePlatform::XboxOne,
    ];

    for platform in platforms {
        let checker = CertificationChecker::new(platform);
        let report = checker.check_all();

        assert_eq!(report.platform, platform);
        assert!(!report.requirements.is_empty());

        // Verify each platform has both common and platform-specific requirements
        assert!(report.requirements.iter().any(|r| r.id == "achievements"));
        assert!(report.requirements.iter().any(|r| r.id == "error_handling"));
    }
}

#[test]
fn test_save_and_load_reports() {
    let checker = CertificationChecker::new(ConsolePlatform::XboxSeries);
    let report = checker.check_all();

    let temp_dir = std::env::temp_dir();

    // Test JSON save
    let json_path = temp_dir.join("test_cert_report.json");
    let result = report.save_json(&json_path);
    assert!(result.is_ok(), "JSON save should succeed");
    assert!(json_path.exists(), "JSON file should exist");

    // Verify JSON content
    let json_content = std::fs::read_to_string(&json_path).unwrap();
    assert!(json_content.contains("\"platform\""));
    assert!(json_content.contains("\"requirements\""));

    // Clean up
    std::fs::remove_file(&json_path).ok();

    // Test HTML save
    let html_path = temp_dir.join("test_cert_report.html");
    let result = report.save_html(&html_path);
    assert!(result.is_ok(), "HTML save should succeed");
    assert!(html_path.exists(), "HTML file should exist");

    // Verify HTML content
    let html_content = std::fs::read_to_string(&html_path).unwrap();
    assert!(html_content.contains("<!DOCTYPE html>"));
    assert!(html_content.contains("Console Certification Report"));

    // Clean up
    std::fs::remove_file(&html_path).ok();
}

#[test]
fn test_required_vs_optional() {
    let checker = CertificationChecker::new(ConsolePlatform::PlayStation5);
    let report = checker.check_all();

    let required_count = report.requirements.iter().filter(|r| r.required).count();
    let optional_count = report.requirements.iter().filter(|r| !r.required).count();

    assert!(required_count > 0, "Should have required requirements");
    assert!(optional_count > 0, "Should have optional requirements");

    // Overall pass status should only consider required requirements
    let required_passed = report.requirements.iter().filter(|r| r.required && r.passed).count();

    let expected_pass = required_passed == required_count;
    assert_eq!(report.overall_passed, expected_pass);
}

#[test]
fn test_report_structure() {
    let checker = CertificationChecker::new(ConsolePlatform::NintendoSwitch);
    let report = checker.check_all();

    // Verify report structure
    assert!(!report.requirements.is_empty());
    assert!(report.test_duration_secs >= 0.0);

    // Verify metadata
    assert!(!report.metadata.engine_version.is_empty());
    assert!(!report.metadata.test_runner_version.is_empty());
}
