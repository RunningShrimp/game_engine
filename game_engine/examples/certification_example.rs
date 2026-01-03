//! Platform Certification System Example
//!
//! This example demonstrates how to use the console certification system
//! to check platform-specific requirements.

use game_engine::platform::console::certification::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Console Certification System Example\n");

    // Test Nintendo Switch certification
    println!("=== Nintendo Switch Certification ===\n");
    let switch_checker =
        CertificationChecker::new(game_engine::platform::console::ConsolePlatform::NintendoSwitch);
    let switch_report = switch_checker.check_all();

    println!("{}\n", switch_report.generate_report());

    // Test PlayStation 5 certification
    println!("=== PlayStation 5 Certification ===\n");
    let ps5_checker =
        CertificationChecker::new(game_engine::platform::console::ConsolePlatform::PlayStation5);
    let ps5_report = ps5_checker.check_all();

    println!("Total Requirements: {}", ps5_report.requirements.len());
    println!("Passed: {}", ps5_report.passed_requirements().len());
    println!("Failed: {}", ps5_report.failed_requirements().len());
    println!("Pass Rate: {:.1}%\n", ps5_report.pass_rate());

    // Show critical failures
    let critical = ps5_report.critical_failures();
    if !critical.is_empty() {
        println!("Critical Failures:");
        for req in critical {
            println!("  - {}: {}", req.id, req.name);
        }
        println!();
    }

    // Test Xbox Series certification
    println!("=== Xbox Series X/S Certification ===\n");
    let xbox_checker =
        CertificationChecker::new(game_engine::platform::console::ConsolePlatform::XboxSeries);
    let xbox_report = xbox_checker.check_all();

    println!("Total Requirements: {}", xbox_report.requirements.len());
    println!("Passed: {}", xbox_report.passed_requirements().len());
    println!("Failed: {}", xbox_report.failed_requirements().len());
    println!("Pass Rate: {:.1}%\n", xbox_report.pass_rate());

    // Save reports to files
    let temp_dir = std::env::temp_dir();

    let json_path = temp_dir.join("switch_cert_report.json");
    switch_report.save_json(&json_path)?;
    println!("✓ JSON report saved to: {:?}", json_path);

    let html_path = temp_dir.join("ps5_cert_report.html");
    ps5_report.save_html(&html_path)?;
    println!("✓ HTML report saved to: {:?}", html_path);

    // Test custom rules
    println!("\n=== Custom Rules Example ===\n");
    let mut checker =
        CertificationChecker::new(game_engine::platform::console::ConsolePlatform::PlayStation5);

    let custom_req = CertRequirement::new(
        "custom_framerate",
        CertCategory::Performance,
        "Custom 60 FPS Requirement",
        "Game must maintain 60 FPS during gameplay",
        true,
        Severity::Critical,
    )
    .with_notes("Custom requirement for this game")
    .with_reference("https://example.com/framerate")
    .mark_passed();

    checker.add_custom_rule(custom_req);
    let custom_report = checker.check_all();

    println!(
        "Custom report includes {} requirements",
        custom_report.requirements.len()
    );
    println!(
        "Custom rule present: {}",
        custom_report.requirements.iter().any(|r| r.id == "custom_framerate")
    );

    println!("\n✓ Certification system test completed successfully!");
    Ok(())
}
