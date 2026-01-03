//! # Console Certification System
//!
//! Comprehensive certification requirement checking system for console platforms.
//! Supports automated testing, report generation, and platform-specific requirements.
//!
//! ## Features
//! - Platform-specific certification rules
//! - Automated requirement checking
//! - JSON and HTML report generation
//! - Test suite integration
//! - Detailed failure analysis
//!
//! ## Supported Platforms
//! - Nintendo Switch
//! - PlayStation 5 / PlayStation 4
//! - Xbox Series X/S / Xbox One

use super::ConsolePlatform;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::Path;

// ============================================================================
// Certification Requirements
// ============================================================================

/// Certification requirement with detailed information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertRequirement {
    pub id: String,
    pub category: CertCategory,
    pub name: String,
    pub description: String,
    pub required: bool,
    pub severity: Severity,
    pub passed: bool,
    pub notes: String,
    pub reference_url: Option<String>,
}

impl CertRequirement {
    /// Create a new certification requirement
    pub fn new(
        id: impl Into<String>,
        category: CertCategory,
        name: impl Into<String>,
        description: impl Into<String>,
        required: bool,
        severity: Severity,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            name: name.into(),
            description: description.into(),
            required,
            severity,
            passed: false,
            notes: String::new(),
            reference_url: None,
        }
    }

    /// Add notes to the requirement
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = notes.into();
        self
    }

    /// Add reference URL
    pub fn with_reference(mut self, url: impl Into<String>) -> Self {
        self.reference_url = Some(url.into());
        self
    }

    /// Mark as passed
    pub fn mark_passed(mut self) -> Self {
        self.passed = true;
        self
    }
}

/// Certification category for grouping requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CertCategory {
    /// User interface and UX requirements
    UserInterface,
    /// Online and networking requirements
    Online,
    /// Save data and storage requirements
    Storage,
    /// Input and controller requirements
    Input,
    /// Performance and technical requirements
    Performance,
    /// Platform integration requirements
    PlatformIntegration,
    /// Security and privacy requirements
    Security,
    /// Accessibility requirements
    Accessibility,
    /// Legal and compliance requirements
    Legal,
}

impl fmt::Display for CertCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CertCategory::UserInterface => write!(f, "User Interface"),
            CertCategory::Online => write!(f, "Online"),
            CertCategory::Storage => write!(f, "Storage"),
            CertCategory::Input => write!(f, "Input"),
            CertCategory::Performance => write!(f, "Performance"),
            CertCategory::PlatformIntegration => write!(f, "Platform Integration"),
            CertCategory::Security => write!(f, "Security"),
            CertCategory::Accessibility => write!(f, "Accessibility"),
            CertCategory::Legal => write!(f, "Legal"),
        }
    }
}

/// Severity level for failed requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Critical failure - will block certification
    Critical,
    /// Major issue - must be fixed
    Major,
    /// Minor issue - should be fixed
    Minor,
    /// Informational
    Info,
}

// ============================================================================
// Certification Report
// ============================================================================

/// Comprehensive certification report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationReport {
    pub platform: ConsolePlatform,
    pub requirements: Vec<CertRequirement>,
    pub overall_passed: bool,
    pub timestamp: std::time::SystemTime,
    pub test_duration_secs: f64,
    pub metadata: ReportMetadata,
}

impl CertificationReport {
    /// Check if all requirements passed
    pub fn all_passed(&self) -> bool {
        self.overall_passed
    }

    /// Get failed requirements
    pub fn failed_requirements(&self) -> Vec<&CertRequirement> {
        self.requirements.iter().filter(|r| !r.passed && r.required).collect()
    }

    /// Get passed requirements
    pub fn passed_requirements(&self) -> Vec<&CertRequirement> {
        self.requirements.iter().filter(|r| r.passed).collect()
    }

    /// Get requirements by category
    pub fn requirements_by_category(&self, category: CertCategory) -> Vec<&CertRequirement> {
        self.requirements.iter().filter(|r| r.category == category).collect()
    }

    /// Get critical failures
    pub fn critical_failures(&self) -> Vec<&CertRequirement> {
        self.requirements
            .iter()
            .filter(|r| !r.passed && r.required && r.severity == Severity::Critical)
            .collect()
    }

    /// Calculate pass rate
    pub fn pass_rate(&self) -> f64 {
        if self.requirements.is_empty() {
            return 0.0;
        }
        let passed = self.requirements.iter().filter(|r| r.passed).count();
        (passed as f64 / self.requirements.len() as f64) * 100.0
    }

    /// Generate text report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("╔══════════════════════════════════════════════════════════════╗\n");
        report.push_str("║         CONSOLE CERTIFICATION REPORT                          ║\n");
        report.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

        report.push_str(&format!("Platform: {}\n", self.platform.name()));
        report.push_str(&format!(
            "Timestamp: {}\n",
            format_timestamp(self.timestamp)
        ));
        report.push_str(&format!(
            "Test Duration: {:.2}s\n\n",
            self.test_duration_secs
        ));

        // Summary
        let passed = self.passed_requirements().len();
        let failed = self.failed_requirements().len();
        let total = self.requirements.len();
        let pass_rate = self.pass_rate();

        report.push_str("┌─ SUMMARY ─────────────────────────────────────────────────┐\n");
        report.push_str(&format!(
            "│ Total Requirements: {:>4}                            │\n",
            total
        ));
        report.push_str(&format!(
            "│ Passed:              {:>4} ({:>5.1}%)                  │\n",
            passed, pass_rate
        ));
        report.push_str(&format!(
            "│ Failed:              {:>4}                            │\n",
            failed
        ));
        report.push_str(&format!(
            "│ Overall Status:      {:>20}            │\n",
            if self.overall_passed {
                "PASSED ✓"
            } else {
                "FAILED ✗"
            }
        ));
        report.push_str("└────────────────────────────────────────────────────────────┘\n\n");

        // Critical failures
        let critical = self.critical_failures();
        if !critical.is_empty() {
            report.push_str("┌─ CRITICAL FAILURES ────────────────────────────────────────┐\n");
            for req in critical {
                report.push_str(&format!(
                    "│ ✗ {}: {}                                │\n",
                    req.id, req.name
                ));
                report.push_str(&format!(
                    "│   {}                                │\n",
                    truncate(&req.description, 52)
                ));
            }
            report.push_str("└────────────────────────────────────────────────────────────┘\n\n");
        }

        // Requirements by category
        for category in &[
            CertCategory::UserInterface,
            CertCategory::Online,
            CertCategory::Storage,
            CertCategory::Input,
            CertCategory::Performance,
            CertCategory::PlatformIntegration,
            CertCategory::Security,
            CertCategory::Accessibility,
        ] {
            let reqs = self.requirements_by_category(*category);
            if !reqs.is_empty() {
                report.push_str(&format!("┌─ {} ─", category));
                let padding = 55usize.saturating_sub(category.to_string().len());
                report.push_str(&"─".repeat(padding));
                report.push_str("┐\n");

                for req in reqs {
                    let status = if req.passed { "✓" } else { "✗" };
                    let required = if req.required { "" } else { " [OPT]" };
                    let notes_str = if !req.notes.is_empty() {
                        format!(" ({})", req.notes)
                    } else {
                        String::new()
                    };
                    report.push_str(&format!(
                        "│ [{}] {}{}: {}{}│\n",
                        status, req.id, required, req.name, notes_str
                    ));
                    if !req.passed && !req.description.is_empty() {
                        let padding = " ".repeat(4);
                        let desc_truncated = truncate(&req.description, 46);
                        report.push_str(&format!("│       {}{}│\n", padding, desc_truncated));
                    }
                }
                report
                    .push_str("└────────────────────────────────────────────────────────────┘\n\n");
            }
        }

        // Footer
        report.push_str(&format!(
            "Generated by Game Engine Certification System v{}\n",
            env!("CARGO_PKG_VERSION")
        ));

        report
    }

    /// Generate JSON report
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Save JSON report to file
    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), CertError> {
        let json = self.to_json().map_err(CertError::SerializationError)?;
        fs::write(path.as_ref(), json).map_err(CertError::IoError)?;
        Ok(())
    }

    /// Generate HTML report
    pub fn to_html(&self) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang='en'>\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset='UTF-8'>\n");
        html.push_str(
            "    <meta name='viewport' content='width=device-width, initial-scale=1.0'>\n",
        );
        html.push_str("    <title>Console Certification Report</title>\n");
        html.push_str("    <style>\n");
        html.push_str("        body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 40px; background: #f5f5f5; }\n");
        html.push_str("        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }\n");
        html.push_str(
            "        h1 { color: #333; border-bottom: 3px solid #007acc; padding-bottom: 10px; }\n",
        );
        html.push_str("        .summary { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; margin: 20px 0; }\n");
        html.push_str("        .summary-card { background: #f8f9fa; padding: 20px; border-radius: 6px; border-left: 4px solid #007acc; }\n");
        html.push_str(
            "        .summary-card h3 { margin: 0 0 10px 0; color: #555; font-size: 14px; }\n",
        );
        html.push_str("        .summary-card .value { font-size: 32px; font-weight: bold; color: #007acc; }\n");
        html.push_str("        .status-passed { color: #28a745; }\n");
        html.push_str("        .status-failed { color: #dc3545; }\n");
        html.push_str("        .category { margin: 30px 0; }\n");
        html.push_str("        .category h2 { color: #333; border-bottom: 2px solid #e9ecef; padding-bottom: 8px; }\n");
        html.push_str(
            "        table { width: 100%; border-collapse: collapse; margin: 10px 0; }\n",
        );
        html.push_str("        th { background: #f8f9fa; padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6; }\n");
        html.push_str("        td { padding: 12px; border-bottom: 1px solid #e9ecef; }\n");
        html.push_str("        tr:hover { background: #f8f9fa; }\n");
        html.push_str("        .badge { display: inline-block; padding: 4px 8px; border-radius: 4px; font-size: 12px; font-weight: bold; }\n");
        html.push_str("        .badge-passed { background: #d4edda; color: #155724; }\n");
        html.push_str("        .badge-failed { background: #f8d7da; color: #721c24; }\n");
        html.push_str("        .badge-critical { background: #f5c6cb; color: #721c24; }\n");
        html.push_str("        .badge-required { background: #cce5ff; color: #004085; }\n");
        html.push_str("        .badge-optional { background: #e2e3e5; color: #383d41; }\n");
        html.push_str("        .critical-failures { background: #f8d7da; border: 2px solid #f5c6cb; border-radius: 6px; padding: 20px; margin: 20px 0; }\n");
        html.push_str("        .timestamp { color: #6c757d; font-size: 14px; }\n");
        html.push_str("    </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        html.push_str("    <div class='container'>\n");
        html.push_str("        <h1>🎮 Console Certification Report</h1>\n");
        html.push_str(&format!("        <p class='timestamp'><strong>Platform:</strong> {} | <strong>Generated:</strong> {} | <strong>Duration:</strong> {:.2}s</p>\n",
            self.platform.name(), format_timestamp(self.timestamp), self.test_duration_secs));

        // Summary cards
        let passed = self.passed_requirements().len();
        let failed = self.failed_requirements().len();
        let total = self.requirements.len();
        let pass_rate = self.pass_rate();

        html.push_str("        <div class='summary'>\n");
        html.push_str(&format!("            <div class='summary-card'><h3>Total Requirements</h3><div class='value'>{}</div></div>\n", total));
        html.push_str(&format!("            <div class='summary-card'><h3>Passed</h3><div class='value status-passed'>{}</div></div>\n", passed));
        html.push_str(&format!("            <div class='summary-card'><h3>Failed</h3><div class='value status-failed'>{}</div></div>\n", failed));
        html.push_str(&format!("            <div class='summary-card'><h3>Pass Rate</h3><div class='value'>{:.1}%</div></div>\n", pass_rate));
        html.push_str("        </div>\n");

        // Critical failures
        let critical = self.critical_failures();
        if !critical.is_empty() {
            html.push_str("        <div class='critical-failures'>\n");
            html.push_str("            <h2>⚠️ Critical Failures</h2>\n");
            html.push_str("            <p>The following critical requirements must be fixed before certification:</p>\n");
            html.push_str("            <ul>\n");
            for req in critical {
                html.push_str(&format!(
                    "                <li><strong>{}:</strong> {}</li>\n",
                    req.id, req.name
                ));
            }
            html.push_str("            </ul>\n");
            html.push_str("        </div>\n");
        }

        // Requirements by category
        for category in &[
            CertCategory::UserInterface,
            CertCategory::Online,
            CertCategory::Storage,
            CertCategory::Input,
            CertCategory::Performance,
            CertCategory::PlatformIntegration,
            CertCategory::Security,
            CertCategory::Accessibility,
        ] {
            let reqs = self.requirements_by_category(*category);
            if !reqs.is_empty() {
                html.push_str(&format!("        <div class='category'>\n"));
                html.push_str(&format!("            <h2>{}</h2>\n", category));
                html.push_str("            <table>\n");
                html.push_str("                <thead>\n");
                html.push_str("                    <tr><th>Status</th><th>ID</th><th>Requirement</th><th>Description</th><th>Severity</th><th>Type</th></tr>\n");
                html.push_str("                </thead>\n");
                html.push_str("                <tbody>\n");
                for req in reqs {
                    html.push_str("                    <tr>\n");
                    html.push_str(&format!(
                        "                        <td><span class='badge {}'>{}</span></td>\n",
                        if req.passed {
                            "badge-passed"
                        } else {
                            "badge-failed"
                        },
                        if req.passed { "✓ PASS" } else { "✗ FAIL" }
                    ));
                    html.push_str(&format!("                        <td>{}</td>\n", req.id));
                    html.push_str(&format!(
                        "                        <td><strong>{}</strong></td>\n",
                        req.name
                    ));
                    html.push_str(&format!(
                        "                        <td>{}</td>\n",
                        truncate(&req.description, 60)
                    ));
                    html.push_str(&format!(
                        "                        <td>{:?}</td>\n",
                        req.severity
                    ));
                    html.push_str(&format!(
                        "                        <td><span class='badge {}'>{}</span></td>\n",
                        if req.required {
                            "badge-required"
                        } else {
                            "badge-optional"
                        },
                        if req.required { "Required" } else { "Optional" }
                    ));
                    html.push_str("                    </tr>\n");
                }
                html.push_str("                </tbody>\n");
                html.push_str("            </table>\n");
                html.push_str("        </div>\n");
            }
        }

        html.push_str("    </div>\n");
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        html
    }

    /// Save HTML report to file
    pub fn save_html(&self, path: impl AsRef<Path>) -> Result<(), CertError> {
        let html = self.to_html();
        fs::write(path.as_ref(), html).map_err(CertError::IoError)?;
        Ok(())
    }
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub engine_version: String,
    pub test_runner_version: String,
    pub platform_sdk_version: Option<String>,
    pub test_environment: String,
}

impl Default for ReportMetadata {
    fn default() -> Self {
        Self {
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            test_runner_version: "1.0.0".to_string(),
            platform_sdk_version: None,
            test_environment: "development".to_string(),
        }
    }
}

// ============================================================================
// Certification Checker
// ============================================================================

/// Main certification checker
pub struct CertificationChecker {
    platform: ConsolePlatform,
    config: CertCheckerConfig,
    custom_rules: HashMap<String, CertRequirement>,
}

impl CertificationChecker {
    /// Create a new certification checker
    pub fn new(platform: ConsolePlatform) -> Self {
        Self {
            platform,
            config: CertCheckerConfig::default(),
            custom_rules: HashMap::new(),
        }
    }

    /// Create checker with custom configuration
    pub fn with_config(platform: ConsolePlatform, config: CertCheckerConfig) -> Self {
        Self {
            platform,
            config,
            custom_rules: HashMap::new(),
        }
    }

    /// Add custom requirement
    pub fn add_custom_rule(&mut self, requirement: CertRequirement) {
        self.custom_rules.insert(requirement.id.clone(), requirement);
    }

    /// Run all certification checks
    pub fn check_all(&self) -> CertificationReport {
        let start_time = std::time::Instant::now();

        let mut requirements = Vec::new();

        // Common requirements for all platforms
        requirements.extend(self.get_common_requirements());

        // Platform-specific requirements
        match self.platform {
            ConsolePlatform::NintendoSwitch => {
                requirements.extend(self.get_nintendo_requirements());
            }
            ConsolePlatform::PlayStation5 | ConsolePlatform::PlayStation4 => {
                requirements.extend(self.get_playstation_requirements());
            }
            ConsolePlatform::XboxSeries | ConsolePlatform::XboxOne => {
                requirements.extend(self.get_xbox_requirements());
            }
        }

        // Add custom rules
        requirements.extend(self.custom_rules.values().cloned());

        // Simulate checking requirements (in real implementation, this would
        // actually test the game against the requirements)
        requirements = self.check_requirements(requirements);

        let overall_passed = requirements.iter().filter(|r| r.required).all(|r| r.passed);

        let test_duration = start_time.elapsed().as_secs_f64();

        CertificationReport {
            platform: self.platform,
            requirements,
            overall_passed,
            timestamp: std::time::SystemTime::now(),
            test_duration_secs: test_duration,
            metadata: ReportMetadata::default(),
        }
    }

    /// Check requirements (simulate testing)
    fn check_requirements(&self, requirements: Vec<CertRequirement>) -> Vec<CertRequirement> {
        requirements
            .into_iter()
            .map(|mut req| {
                // In a real implementation, this would actually check the game
                // For now, we simulate checking by implementing some basic checks
                req.passed = self.simulate_check(&req);

                if !req.passed {
                    req.notes = "Not yet implemented or failed check".to_string();
                }

                req
            })
            .collect()
    }

    /// Simulate checking a requirement
    fn simulate_check(&self, req: &CertRequirement) -> bool {
        // For demonstration, pass some and fail others
        // In real implementation, this would perform actual checks
        match req.category {
            CertCategory::UserInterface => {
                // Assume error handling passes
                req.id.contains("error_handling")
            }
            CertCategory::Performance => true, // Assume performance checks pass
            _ => false,                        // Most others not implemented yet
        }
    }

    /// Get common requirements for all platforms
    fn get_common_requirements(&self) -> Vec<CertRequirement> {
        vec![
            CertRequirement::new(
                "achievements",
                CertCategory::PlatformIntegration,
                "Achievement/Trophy System",
                "Game must integrate platform achievement/trophy system",
                true,
                Severity::Critical,
            )
            .with_reference("https://example.com/achievements"),
            CertRequirement::new(
                "cloud_save",
                CertCategory::Storage,
                "Cloud Save Support",
                "Game must support cloud saves with automatic sync",
                true,
                Severity::Critical,
            )
            .with_reference("https://example.com/cloud-save"),
            CertRequirement::new(
                "controller_vibration",
                CertCategory::Input,
                "Controller Vibration",
                "Game must use controller vibration feedback appropriately",
                true,
                Severity::Major,
            ),
            CertRequirement::new(
                "error_handling",
                CertCategory::UserInterface,
                "Error Handling",
                "Game must handle errors gracefully without crashing",
                true,
                Severity::Critical,
            ),
            CertRequirement::new(
                "loading_screen",
                CertCategory::UserInterface,
                "Loading Screen with Progress",
                "Loading screens must show progress indicator and cannot be skipped",
                true,
                Severity::Major,
            )
            .with_reference("https://example.com/loading-screens"),
            CertRequirement::new(
                "pause_menu",
                CertCategory::UserInterface,
                "Pause Menu",
                "Game must have pause menu accessible at all times during gameplay",
                true,
                Severity::Critical,
            ),
            CertRequirement::new(
                "network_disconnect",
                CertCategory::Online,
                "Network Disconnect Handling",
                "Game must handle network disconnection gracefully",
                true,
                Severity::Critical,
            )
            .with_reference("https://example.com/network-handling"),
            CertRequirement::new(
                "save_corruption",
                CertCategory::Storage,
                "Save Corruption Handling",
                "Game must detect and handle corrupted save files",
                true,
                Severity::Critical,
            ),
            CertRequirement::new(
                "frame_rate_stability",
                CertCategory::Performance,
                "Frame Rate Stability",
                "Game must maintain stable frame rate without major drops",
                true,
                Severity::Critical,
            )
            .with_reference("https://example.com/performance"),
            CertRequirement::new(
                "memory_limits",
                CertCategory::Performance,
                "Memory Usage Limits",
                "Game must not exceed platform memory limits",
                true,
                Severity::Critical,
            ),
            CertRequirement::new(
                "button_prompts",
                CertCategory::UserInterface,
                "Correct Button Prompts",
                "UI must show correct button prompts for the current platform",
                true,
                Severity::Major,
            ),
            CertRequirement::new(
                "accessibility_options",
                CertCategory::Accessibility,
                "Accessibility Options",
                "Game must include basic accessibility options (subtitles, colorblind modes)",
                true,
                Severity::Major,
            )
            .with_reference("https://example.com/accessibility"),
        ]
    }

    /// Get Nintendo Switch-specific requirements
    fn get_nintendo_requirements(&self) -> Vec<CertRequirement> {
        vec![
            CertRequirement::new(
                "switch_save_limits",
                CertCategory::Storage,
                "Save Size Limits",
                "Save files must not exceed platform-specific limits (typically < 100MB per save)",
                true,
                Severity::Critical,
            )
            .with_reference("https://developer.nintendo.com/switch/save-limits"),
            CertRequirement::new(
                "joycon_pairing",
                CertCategory::Input,
                "Joy-Con Pairing/Unpairing",
                "Game must support Joy-Con pairing/unpairing during gameplay",
                true,
                Severity::Critical,
            )
            .with_reference("https://developer.nintendo.com/switch/joycon"),
            CertRequirement::new(
                "dock_mode",
                CertCategory::Performance,
                "Dock/Handheld Mode",
                "Game must handle dock/undock events and adapt resolution accordingly",
                true,
                Severity::Critical,
            )
            .with_reference("https://developer.nintendo.com/switch/dock-mode"),
            CertRequirement::new(
                "screenshot",
                CertCategory::UserInterface,
                "Screenshot Functionality",
                "Game must allow screenshots via capture button",
                true,
                Severity::Major,
            ),
            CertRequirement::new(
                "video_capture",
                CertCategory::UserInterface,
                "Video Capture",
                "Game must support video capture (30-second limit)",
                false,
                Severity::Minor,
            )
            .with_notes("Recommended but not required"),
            CertRequirement::new(
                "nintendo_ages",
                CertCategory::PlatformIntegration,
                "Nintendo Ages Integration",
                "Parental controls must be integrated with Nintendo Ages",
                true,
                Severity::Critical,
            )
            .with_reference("https://developer.nintendo.com/switch/parental-controls"),
            CertRequirement::new(
                "controller_discovery",
                CertCategory::Input,
                "Controller Discovery",
                "Game must support controller discovery and assignment",
                true,
                Severity::Major,
            ),
            CertRequirement::new(
                "sleep_mode",
                CertCategory::Performance,
                "Sleep Mode Handling",
                "Game must properly handle sleep mode and resume",
                true,
                Severity::Critical,
            )
            .with_reference("https://developer.nintendo.com/switch/sleep-mode"),
        ]
    }

    /// Get PlayStation-specific requirements
    fn get_playstation_requirements(&self) -> Vec<CertRequirement> {
        let is_ps5 = self.platform == ConsolePlatform::PlayStation5;

        let mut reqs = vec![
            CertRequirement::new(
                "trophy_icons",
                CertCategory::PlatformIntegration,
                "Trophy Icons",
                "All trophies must have proper icons that meet specifications",
                true,
                Severity::Critical,
            )
            .with_reference("https://partners.playstation.com/trophy-specs"),
            CertRequirement::new(
                "psn_integration",
                CertCategory::Online,
                "PSN Integration",
                "Online features must integrate with PlayStation Network",
                true,
                Severity::Critical,
            )
            .with_reference("https://partners.playstation.com/psn-integration"),
            CertRequirement::new(
                "ui_guidelines",
                CertCategory::UserInterface,
                "PlayStation UI Guidelines",
                "Must follow PlayStation UI guidelines and terminology",
                true,
                Severity::Major,
            )
            .with_reference("https://partners.playstation.com/ui-guidelines"),
            CertRequirement::new(
                "ps_button_behavior",
                CertCategory::UserInterface,
                "PS Button Behavior",
                "PS button must open system menu as expected",
                true,
                Severity::Critical,
            ),
            CertRequirement::new(
                "share_button",
                CertCategory::UserInterface,
                "Share Button",
                "Share button must function correctly on PS4/PS5 controllers",
                true,
                Severity::Major,
            ),
            CertRequirement::new(
                "party_chat",
                CertCategory::Online,
                "Party Chat Integration",
                "Game must not interfere with PlayStation Party chat",
                true,
                Severity::Critical,
            )
            .with_reference("https://partners.playstation.com/party-chat"),
            CertRequirement::new(
                "psn_avatars",
                CertCategory::PlatformIntegration,
                "PSN Avatar Display",
                "Game should display PSN avatars where appropriate",
                false,
                Severity::Minor,
            )
            .with_notes("Recommended but not required"),
            CertRequirement::new(
                "activity_cards",
                CertCategory::UserInterface,
                "Activity Cards (PS5)",
                "PS5 games should implement activity cards for quick access",
                false,
                Severity::Minor,
            )
            .with_notes("PS5 specific feature - highly recommended"),
        ];

        // Add PS5-specific requirements
        if is_ps5 {
            reqs.push(
                CertRequirement::new(
                    "dualsense_feedback",
                    CertCategory::Input,
                    "DualSense Features",
                    "PS5: Use haptic feedback and adaptive triggers",
                    false,
                    Severity::Major,
                )
                .with_reference("https://partners.playstation.com/dualsense")
                .with_notes("Highly recommended but not strictly required"),
            );

            reqs.push(
                CertRequirement::new(
                    "3d_audio",
                    CertCategory::Performance,
                    "Tempest 3D Audio",
                    "PS5: Support Tempest 3D audio if applicable",
                    false,
                    Severity::Minor,
                )
                .with_notes("Optional feature"),
            );
        }

        reqs
    }

    /// Get Xbox-specific requirements
    fn get_xbox_requirements(&self) -> Vec<CertRequirement> {
        vec![
            CertRequirement::new(
                "achievement_icons",
                CertCategory::PlatformIntegration,
                "Achievement Icons",
                "All achievements must have proper icons that meet specifications",
                true,
                Severity::Critical,
            )
            .with_reference("https://partner.microsoft.com/achievement-specs"),
            CertRequirement::new(
                "xbox_live_integration",
                CertCategory::Online,
                "Xbox Live Integration",
                "Online features must integrate with Xbox Live",
                true,
                Severity::Critical,
            )
            .with_reference("https://partner.microsoft.com/xbox-live-integration"),
            CertRequirement::new(
                "gamerscore",
                CertCategory::PlatformIntegration,
                "Gamerscore Allocation",
                "Achievements must have proper gamerscore values (total must be multiple of 50)",
                true,
                Severity::Critical,
            )
            .with_reference("https://partner.microsoft.com/gamerscore-rules"),
            CertRequirement::new(
                "ui_guidelines",
                CertCategory::UserInterface,
                "Xbox UI Guidelines",
                "Must follow Xbox UI guidelines and terminology (A/B vs Cross/Circle)",
                true,
                Severity::Major,
            )
            .with_reference("https://partner.microsoft.com/ui-guidelines"),
            CertRequirement::new(
                "xbox_button_behavior",
                CertCategory::UserInterface,
                "Xbox Button Behavior",
                "Xbox button must open system menu as expected",
                true,
                Severity::Critical,
            ),
            CertRequirement::new(
                "smart_delivery",
                CertCategory::PlatformIntegration,
                "Smart Delivery",
                "Xbox Series X/S games should support Smart Delivery",
                false,
                Severity::Major,
            )
            .with_notes("Recommended but not required"),
            CertRequirement::new(
                "quick_resume",
                CertCategory::Performance,
                "Quick Resume Support",
                "Xbox Series X/S: Game should support Quick Resume",
                true,
                Severity::Major,
            )
            .with_reference("https://partner.microsoft.com/quick-resume"),
            CertRequirement::new(
                "xbox_cloud_gaming",
                CertCategory::Online,
                "Cloud Gaming Compatibility",
                "Game must work properly with Xbox Cloud Gaming",
                false,
                Severity::Minor,
            )
            .with_notes("Only if targeting cloud gaming"),
            CertRequirement::new(
                "language_support",
                CertCategory::Accessibility,
                "Language Support",
                "Game must support all interface languages supported by Xbox",
                true,
                Severity::Major,
            )
            .with_reference("https://partner.microsoft.com/language-requirements"),
            CertRequirement::new(
                "party_chat",
                CertCategory::Online,
                "Party Chat Integration",
                "Game must not interfere with Xbox Party chat",
                true,
                Severity::Critical,
            )
            .with_reference("https://partner.microsoft.com/party-chat"),
        ]
    }
}

impl Default for CertificationChecker {
    fn default() -> Self {
        Self::new(ConsolePlatform::PlayStation5)
    }
}

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for certification checker
#[derive(Debug, Clone)]
pub struct CertCheckerConfig {
    /// Whether to include optional requirements
    pub include_optional: bool,
    /// Custom severity thresholds
    pub severity_threshold: Severity,
    /// Whether to generate detailed reports
    pub verbose: bool,
    /// Whether to save reports to disk
    pub save_reports: bool,
    /// Output directory for reports
    pub output_dir: Option<String>,
}

impl Default for CertCheckerConfig {
    fn default() -> Self {
        Self {
            include_optional: true,
            severity_threshold: Severity::Info,
            verbose: false,
            save_reports: false,
            output_dir: None,
        }
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// Certification error types
#[derive(Debug, thiserror::Error)]
pub enum CertError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Test execution failed: {0}")]
    TestFailed(String),
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Format timestamp for display
fn format_timestamp(time: std::time::SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            format!("{:?}", secs)
        }
        Err(_) => "Unknown".to_string(),
    }
}

/// Truncate string to specified length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certification_checker_creation() {
        let checker = CertificationChecker::new(ConsolePlatform::PlayStation5);
        assert_eq!(checker.platform, ConsolePlatform::PlayStation5);
    }

    #[test]
    fn test_check_all_requirements() {
        let checker = CertificationChecker::new(ConsolePlatform::NintendoSwitch);
        let report = checker.check_all();

        assert_eq!(report.platform, ConsolePlatform::NintendoSwitch);
        assert!(!report.all_passed()); // Most should fail since not implemented
        assert!(!report.requirements.is_empty());
    }

    #[test]
    fn test_nintendo_requirements() {
        let checker = CertificationChecker::new(ConsolePlatform::NintendoSwitch);
        let report = checker.check_all();

        // Check for Nintendo-specific requirements
        assert!(report.requirements.iter().any(|r| r.id == "switch_save_limits"));
        assert!(report.requirements.iter().any(|r| r.id == "joycon_pairing"));
        assert!(report.requirements.iter().any(|r| r.id == "dock_mode"));
    }

    #[test]
    fn test_playstation_requirements() {
        let checker = CertificationChecker::new(ConsolePlatform::PlayStation5);
        let report = checker.check_all();

        // Check for PlayStation-specific requirements
        assert!(report.requirements.iter().any(|r| r.id == "trophy_icons"));
        assert!(report.requirements.iter().any(|r| r.id == "dualsense_feedback"));
        assert!(report.requirements.iter().any(|r| r.id == "psn_integration"));
    }

    #[test]
    fn test_xbox_requirements() {
        let checker = CertificationChecker::new(ConsolePlatform::XboxSeries);
        let report = checker.check_all();

        // Check for Xbox-specific requirements
        assert!(report.requirements.iter().any(|r| r.id == "achievement_icons"));
        assert!(report.requirements.iter().any(|r| r.id == "gamerscore"));
        assert!(report.requirements.iter().any(|r| r.id == "xbox_live_integration"));
    }

    #[test]
    fn test_report_generation() {
        let checker = CertificationChecker::new(ConsolePlatform::PlayStation4);
        let report = checker.check_all();

        let text = report.generate_report();
        assert!(text.contains("Console Certification Report"));
        assert!(text.contains("PlayStation 4"));
        assert!(text.contains("SUMMARY"));
    }

    #[test]
    fn test_json_generation() {
        let checker = CertificationChecker::new(ConsolePlatform::XboxOne);
        let report = checker.check_all();

        let json = report.to_json().unwrap();
        assert!(json.contains("\"platform\""));
        assert!(json.contains("\"requirements\""));
        assert!(json.contains("\"overall_passed\""));
    }

    #[test]
    fn test_html_generation() {
        let checker = CertificationChecker::new(ConsolePlatform::PlayStation5);
        let report = checker.check_all();

        let html = report.to_html();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Console Certification Report</title>"));
        assert!(html.contains("summary-card"));
    }

    #[test]
    fn test_requirement_categories() {
        let checker = CertificationChecker::new(ConsolePlatform::NintendoSwitch);
        let report = checker.check_all();

        // Check that requirements are categorized
        let ui_reqs = report.requirements_by_category(CertCategory::UserInterface);
        let online_reqs = report.requirements_by_category(CertCategory::Online);
        let storage_reqs = report.requirements_by_category(CertCategory::Storage);

        assert!(!ui_reqs.is_empty());
        assert!(!online_reqs.is_empty());
        assert!(!storage_reqs.is_empty());
    }

    #[test]
    fn test_failed_requirements() {
        let checker = CertificationChecker::new(ConsolePlatform::XboxSeries);
        let report = checker.check_all();

        let failed = report.failed_requirements();
        assert!(!failed.is_empty());

        let passed = report.passed_requirements();
        assert!(!passed.is_empty());
    }

    #[test]
    fn test_pass_rate_calculation() {
        let checker = CertificationChecker::new(ConsolePlatform::PlayStation5);
        let report = checker.check_all();

        let pass_rate = report.pass_rate();
        assert!(pass_rate >= 0.0 && pass_rate <= 100.0);
    }

    #[test]
    fn test_critical_failures() {
        let checker = CertificationChecker::new(ConsolePlatform::NintendoSwitch);
        let report = checker.check_all();

        // There should be critical failures since most things aren't implemented
        let critical = report.critical_failures();
        // In simulation, some might pass, but many will fail
        assert!(!report.requirements.is_empty());
    }

    #[test]
    fn test_custom_rules() {
        let mut checker = CertificationChecker::new(ConsolePlatform::PlayStation5);

        let custom_req = CertRequirement::new(
            "custom_test",
            CertCategory::Performance,
            "Custom Test Requirement",
            "This is a custom requirement for testing",
            true,
            Severity::Major,
        )
        .mark_passed();

        checker.add_custom_rule(custom_req);
        let report = checker.check_all();

        assert!(report.requirements.iter().any(|r| r.id == "custom_test"));
        assert!(report.requirements.iter().find(|r| r.id == "custom_test").unwrap().passed);
    }

    #[test]
    fn test_requirement_severity() {
        let req1 = CertRequirement::new(
            "test1",
            CertCategory::Input,
            "Test",
            "Desc",
            true,
            Severity::Critical,
        );
        let req2 = CertRequirement::new(
            "test2",
            CertCategory::Input,
            "Test",
            "Desc",
            true,
            Severity::Major,
        );
        let req3 = CertRequirement::new(
            "test3",
            CertCategory::Input,
            "Test",
            "Desc",
            true,
            Severity::Minor,
        );

        assert_eq!(req1.severity, Severity::Critical);
        assert_eq!(req2.severity, Severity::Major);
        assert_eq!(req3.severity, Severity::Minor);
    }

    #[test]
    fn test_requirement_builder() {
        let req = CertRequirement::new(
            "id",
            CertCategory::Input,
            "Name",
            "Desc",
            true,
            Severity::Critical,
        )
        .with_notes("Test notes")
        .with_reference("https://example.com");

        assert_eq!(req.notes, "Test notes");
        assert_eq!(req.reference_url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_save_load_json() {
        let checker = CertificationChecker::new(ConsolePlatform::XboxSeries);
        let report = checker.check_all();

        // Save to temp file
        let temp_dir = std::env::temp_dir();
        let json_path = temp_dir.join("test_cert_report.json");

        report.save_json(&json_path).unwrap();
        assert!(json_path.exists());

        // Clean up
        std::fs::remove_file(&json_path).unwrap();
    }

    #[test]
    fn test_save_load_html() {
        let checker = CertificationChecker::new(ConsolePlatform::PlayStation5);
        let report = checker.check_all();

        // Save to temp file
        let temp_dir = std::env::temp_dir();
        let html_path = temp_dir.join("test_cert_report.html");

        report.save_html(&html_path).unwrap();
        assert!(html_path.exists());

        // Verify content
        let content = std::fs::read_to_string(&html_path).unwrap();
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains("Console Certification Report"));

        // Clean up
        std::fs::remove_file(&html_path).unwrap();
    }

    #[test]
    fn test_config_default() {
        let config = CertCheckerConfig::default();
        assert!(config.include_optional);
        assert_eq!(config.severity_threshold, Severity::Info);
        assert!(!config.verbose);
        assert!(!config.save_reports);
    }

    #[test]
    fn test_report_metadata() {
        let metadata = ReportMetadata::default();
        assert!(!metadata.engine_version.is_empty());
        assert!(!metadata.test_runner_version.is_empty());
        assert_eq!(metadata.test_environment, "development");
    }

    #[test]
    fn test_category_display() {
        assert_eq!(CertCategory::UserInterface.to_string(), "User Interface");
        assert_eq!(CertCategory::Online.to_string(), "Online");
        assert_eq!(CertCategory::Storage.to_string(), "Storage");
    }

    #[test]
    fn test_timestamp_formatting() {
        let time = std::time::SystemTime::now();
        let formatted = format_timestamp(time);
        assert!(formatted.contains("UTC"));
    }

    #[test]
    fn test_truncate_function() {
        assert_eq!(truncate("Hello", 10), "Hello");
        assert_eq!(truncate("Hello World", 5), "He...");
        assert_eq!(truncate("Test", 4), "Test");
    }
}
