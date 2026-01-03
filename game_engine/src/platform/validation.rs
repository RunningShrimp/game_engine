//! # Platform Compatibility Validation System
//!
//! Comprehensive validation tools for platform compatibility checking.

use crate::platform::console::{ConsolePlatform, ControllerState};
use crate::platform::detection_extended::{Feature, Platform, PlatformCapabilities};
use crate::platform::mock::base_mock::MockPlatform;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Compatibility validator
pub struct CompatibilityValidator {
    platform: Platform,
    strict_mode: bool,
    capabilities: PlatformCapabilities,
}

impl CompatibilityValidator {
    /// Create a new validator for the specified platform
    pub fn new(platform: Platform) -> Self {
        let capabilities = crate::platform::detection_extended::platform_capabilities(platform);

        Self {
            platform,
            strict_mode: false,
            capabilities,
        }
    }

    /// Create a new validator with strict mode enabled
    pub fn new_strict(platform: Platform) -> Self {
        let mut validator = Self::new(platform);
        validator.strict_mode = true;
        validator
    }

    /// Set strict mode
    pub fn set_strict_mode(&mut self, strict: bool) {
        self.strict_mode = strict;
    }

    /// Run all validations
    pub fn validate_all(&self) -> CompatibilityReport {
        let mut report = CompatibilityReport::new(self.platform);

        // Validate controller API
        match self.validate_controller() {
            Ok(_) => report.passed += 1,
            Err(errors) => {
                report.failed += errors.len();
                report.errors.extend(errors);
            }
        }

        // Validate certification system
        match self.validate_certification() {
            Ok(_) => report.passed += 1,
            Err(errors) => {
                report.failed += errors.len();
                report.errors.extend(errors);
            }
        }

        // Validate GPU features
        match self.validate_gpu() {
            Ok(_) => report.passed += 1,
            Err(errors) => {
                report.failed += errors.len();
                report.errors.extend(errors);
            }
        }

        // Validate memory constraints
        match self.validate_memory() {
            Ok(_) => report.passed += 1,
            Err(errors) => {
                report.failed += errors.len();
                report.errors.extend(errors);
            }
        }

        // Validate performance constraints
        match self.validate_performance() {
            Ok(_) => report.passed += 1,
            Err(errors) => {
                report.failed += errors.len();
                report.errors.extend(errors);
            }
        }

        // Check for warnings
        self.check_warnings(&mut report);

        report
    }

    /// Validate controller API compatibility
    pub fn validate_controller(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check if platform supports controllers
        if self.platform.is_console() {
            // All consoles should support at least one controller
            if self.capabilities.hardware.cpu_cores == 0 {
                errors.push("No CPU cores detected".into());
            }
        }

        // Check controller-specific features
        if self.platform.is_console() && !self.capabilities.supports_feature(Feature::Vibration) {
            if self.strict_mode {
                errors.push("Controller vibration not supported".into());
            } else {
                // Only warn in non-strict mode
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate certification system compatibility
    pub fn validate_certification(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check platform-specific certification requirements
        match self.platform {
            Platform::PlayStation5 | Platform::PlayStation4 => {
                // PlayStation requires specific features
                if !self.capabilities.supports_feature(Feature::Achievements) {
                    errors.push("PlayStation platforms must support achievements".into());
                }
                if !self.capabilities.supports_feature(Feature::Leaderboards) {
                    errors.push("PlayStation platforms must support leaderboards".into());
                }
            }
            Platform::XboxSeries | Platform::XboxOne => {
                // Xbox requires specific features
                if !self.capabilities.supports_feature(Feature::Achievements) {
                    errors.push("Xbox platforms must support achievements".into());
                }
                if !self.capabilities.supports_feature(Feature::CloudSave) {
                    errors.push("Xbox platforms must support cloud save".into());
                }
            }
            Platform::NintendoSwitch => {
                // Switch has more flexible requirements
                if !self.capabilities.supports_feature(Feature::OnlineMultiplayer) {
                    if self.strict_mode {
                        errors.push("Switch should support online multiplayer".into());
                    }
                }
            }
            _ => {
                // Non-console platforms don't have strict certification requirements
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate GPU feature compatibility
    pub fn validate_gpu(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check minimum GPU memory requirements
        let min_gpu_memory = match self.platform {
            Platform::NintendoSwitch => 1024,                   // 1GB minimum
            Platform::PlayStation4 | Platform::XboxOne => 2048, // 2GB minimum
            Platform::PlayStation5 | Platform::XboxSeries => 8192, // 8GB minimum
            _ => 512,                                           // 512MB for other platforms
        };

        if self.capabilities.hardware.gpu_memory_mb < min_gpu_memory {
            errors.push(format!(
                "Insufficient GPU memory: {}MB (required: {}MB)",
                self.capabilities.hardware.gpu_memory_mb, min_gpu_memory
            ));
        }

        // Validate texture size support
        if self.capabilities.max_texture_size < 4096 {
            errors.push(format!(
                "Texture size too small: {} (minimum: 4096)",
                self.capabilities.max_texture_size
            ));
        }

        // Check render target support
        if self.capabilities.max_render_targets < 4 {
            errors.push(format!(
                "Insufficient render targets: {} (minimum: 4)",
                self.capabilities.max_render_targets
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate memory constraints
    pub fn validate_memory(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check minimum memory requirements
        let min_memory = match self.platform {
            Platform::NintendoSwitch => 2048,                   // 2GB minimum
            Platform::PlayStation4 | Platform::XboxOne => 4096, // 4GB minimum
            Platform::PlayStation5 | Platform::XboxSeries => 8192, // 8GB minimum
            _ => 1024,                                          // 1GB for other platforms
        };

        if self.capabilities.hardware.memory_mb < min_memory {
            errors.push(format!(
                "Insufficient memory: {}MB (required: {}MB)",
                self.capabilities.hardware.memory_mb, min_memory
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate performance constraints
    pub fn validate_performance(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check CPU requirements
        let min_cores = match self.platform {
            Platform::NintendoSwitch => 3, // Switch has 4 cores but typically uses 3
            Platform::PlayStation4 | Platform::PlayStation5 => 8,
            Platform::XboxOne | Platform::XboxSeries => 8,
            _ => 2,
        };

        if self.capabilities.hardware.cpu_cores < min_cores {
            errors.push(format!(
                "Insufficient CPU cores: {} (required: {})",
                self.capabilities.hardware.cpu_cores, min_cores
            ));
        }

        // Check SIMD support
        if !self.capabilities.hardware.supports_simd && self.platform.is_console() {
            if self.strict_mode {
                errors.push("Console platforms should support SIMD".into());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Check for potential warnings
    fn check_warnings(&self, report: &mut CompatibilityReport) {
        // Check for missing optional features
        let recommended_features = match self.platform {
            Platform::PlayStation5 | Platform::XboxSeries => {
                vec![Feature::RayTracing, Feature::HDR, Feature::SpatialAudio]
            }
            Platform::PlayStation4 | Platform::XboxOne => vec![Feature::HDR, Feature::SpatialAudio],
            Platform::NintendoSwitch => vec![Feature::MotionControls],
            _ => vec![],
        };

        for feature in recommended_features {
            if !self.capabilities.supports_feature(feature) {
                report
                    .warnings
                    .push(format!("Recommended feature not available: {:?}", feature));
            }
        }

        // Check for low memory configurations
        if self.capabilities.hardware.memory_mb < 4096 {
            report.warnings.push(format!(
                "Low memory configuration: {}MB may affect performance",
                self.capabilities.hardware.memory_mb
            ));
        }
    }
}

/// Compatibility validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    /// Platform being validated
    pub platform: Platform,
    /// Number of passed tests
    pub passed: usize,
    /// Number of failed tests
    pub failed: usize,
    /// Warnings (non-critical issues)
    pub warnings: Vec<String>,
    /// Errors (critical issues)
    pub errors: Vec<String>,
    /// Validation timestamp
    pub timestamp: String,
}

impl CompatibilityReport {
    /// Create a new report
    pub fn new(platform: Platform) -> Self {
        Self {
            platform,
            passed: 0,
            failed: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if validation passed with no warnings
    pub fn is_perfect(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    /// Get success rate
    pub fn success_rate(&self) -> f32 {
        let total = self.passed + self.failed;
        if total == 0 {
            1.0
        } else {
            self.passed as f32 / total as f32
        }
    }

    /// Get summary
    pub fn summary(&self) -> String {
        format!(
            "Platform: {} | Passed: {} | Failed: {} | Warnings: {} | Success Rate: {:.1}%",
            self.platform,
            self.passed,
            self.failed,
            self.warnings.len(),
            self.success_rate() * 100.0
        )
    }
}

impl fmt::Display for CompatibilityReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Compatibility Report for {}", self.platform)?;
        writeln!(f, "Timestamp: {}", self.timestamp)?;
        writeln!(f, "{}", "-".repeat(60))?;
        writeln!(f, "Passed:  {}", self.passed)?;
        writeln!(f, "Failed:  {}", self.failed)?;
        writeln!(f, "Warnings: {}", self.warnings.len())?;
        writeln!(f, "{}", "-".repeat(60))?;

        if !self.errors.is_empty() {
            writeln!(f, "Errors:")?;
            for error in &self.errors {
                writeln!(f, "  - {}", error)?;
            }
            writeln!(f)?;
        }

        if !self.warnings.is_empty() {
            writeln!(f, "Warnings:")?;
            for warning in &self.warnings {
                writeln!(f, "  - {}", warning)?;
            }
            writeln!(f)?;
        }

        writeln!(
            f,
            "Status: {}",
            if self.is_valid() { "PASS" } else { "FAIL" }
        )?;

        Ok(())
    }
}

/// Hardware capability matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilityMatrix {
    /// Platform capabilities mapping
    pub platforms: HashMap<String, PlatformCapabilities>,
    /// Feature availability matrix
    pub feature_matrix: HashMap<String, HashMap<String, bool>>,
}

impl HardwareCapabilityMatrix {
    /// Create a new capability matrix
    pub fn new() -> Self {
        let mut platforms = HashMap::new();
        let mut feature_matrix = HashMap::new();

        // Initialize with all console platforms
        let console_platforms = vec![
            Platform::NintendoSwitch,
            Platform::PlayStation5,
            Platform::PlayStation4,
            Platform::XboxSeries,
            Platform::XboxOne,
        ];

        for platform in &console_platforms {
            let caps = crate::platform::detection_extended::platform_capabilities(*platform);
            let platform_name = platform.to_string();

            // Add feature availability
            let mut features = HashMap::new();
            for feature in &caps.supported_features {
                features.insert(format!("{:?}", feature), true);
            }

            platforms.insert(platform_name.clone(), caps);
            feature_matrix.insert(platform_name, features);
        }

        Self {
            platforms,
            feature_matrix,
        }
    }

    /// Get capabilities for a platform
    pub fn get_capabilities(&self, platform: &str) -> Option<&PlatformCapabilities> {
        self.platforms.get(platform)
    }

    /// Check if a platform supports a feature
    pub fn supports_feature(&self, platform: &str, feature: &str) -> bool {
        self.feature_matrix
            .get(platform)
            .and_then(|features| features.get(feature))
            .copied()
            .unwrap_or(false)
    }

    /// Generate a markdown comparison table
    pub fn generate_comparison_table(&self) -> String {
        let mut table = String::new();
        table.push_str("# Platform Capability Comparison\n\n");
        table.push_str("| Feature | Switch | PS5 | PS4 | Xbox Series | Xbox One |\n");
        table.push_str("|---------|--------|-----|-----|-------------|----------|\n");

        // Get all unique features
        let mut all_features = Vec::new();
        for features in self.feature_matrix.values() {
            for feature in features.keys() {
                if !all_features.contains(feature) {
                    all_features.push(feature.clone());
                }
            }
        }

        all_features.sort();

        // Generate table rows
        for feature in all_features {
            table.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                feature,
                if self.supports_feature("Nintendo Switch", &feature) {
                    "✓"
                } else {
                    "✗"
                },
                if self.supports_feature("PlayStation 5", &feature) {
                    "✓"
                } else {
                    "✗"
                },
                if self.supports_feature("PlayStation 4", &feature) {
                    "✓"
                } else {
                    "✗"
                },
                if self.supports_feature("Xbox Series X/S", &feature) {
                    "✓"
                } else {
                    "✗"
                },
                if self.supports_feature("Xbox One", &feature) {
                    "✓"
                } else {
                    "✗"
                },
            ));
        }

        table
    }
}

impl Default for HardwareCapabilityMatrix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = CompatibilityValidator::new(Platform::PlayStation5);
        assert_eq!(validator.platform, Platform::PlayStation5);
        assert!(!validator.strict_mode);
    }

    #[test]
    fn test_validator_strict() {
        let validator = CompatibilityValidator::new_strict(Platform::XboxSeries);
        assert!(validator.strict_mode);
    }

    #[test]
    fn test_ps5_validation() {
        let validator = CompatibilityValidator::new(Platform::PlayStation5);
        let report = validator.validate_all();

        // PS5 should pass most validations
        assert!(report.is_valid());
        assert!(report.passed >= 5);
    }

    #[test]
    fn test_switch_validation() {
        let validator = CompatibilityValidator::new(Platform::NintendoSwitch);
        let report = validator.validate_all();

        // Switch has different requirements
        assert!(report.passed >= 4);
    }

    #[test]
    fn test_report_summary() {
        let mut report = CompatibilityReport::new(Platform::PlayStation5);
        report.passed = 5;
        report.failed = 0;

        assert!(report.is_valid());
        assert!(report.is_perfect());
        assert_eq!(report.success_rate(), 1.0);

        report.warnings.push("Test warning".into());
        assert!(report.is_valid());
        assert!(!report.is_perfect());
    }

    #[test]
    fn test_capability_matrix() {
        let matrix = HardwareCapabilityMatrix::new();

        // Check PS5 capabilities
        let ps5_caps = matrix.get_capabilities("PlayStation 5");
        assert!(ps5_caps.is_some());

        // Check feature support
        assert!(matrix.supports_feature("PlayStation 5", "RayTracing"));
        assert!(!matrix.supports_feature("Nintendo Switch", "RayTracing"));
    }

    #[test]
    fn test_comparison_table() {
        let matrix = HardwareCapabilityMatrix::new();
        let table = matrix.generate_comparison_table();

        assert!(table.contains("| Feature |"));
        assert!(table.contains("RayTracing"));
        assert!(table.contains("HDR"));
    }
}
