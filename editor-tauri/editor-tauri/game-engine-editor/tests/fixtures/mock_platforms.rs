// Mock Platforms
// 提供mock平台实现用于测试

use std::collections::HashMap;
use crate::fixtures::test_entities::{TestControllerState, TestPlatformInfo, TestGPUInfo};

/// Mock平台认证系统
#[derive(Debug, Clone)]
pub struct MockCertificationSystem {
    pub platform: String,
    pub certified: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl MockCertificationSystem {
    pub fn new(platform: &str) -> Self {
        Self {
            platform: platform.to_string(),
            certified: false,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn check_certification(&mut self) -> Result<bool, String> {
        self.errors.clear();
        self.warnings.clear();

        // 模拟平台特定的认证规则
        match self.platform.as_str() {
            "PS5" => {
                self.check_ps5_certification();
            }
            "Xbox" => {
                self.check_xbox_certification();
            }
            "Switch" => {
                self.check_switch_certification();
            }
            "Steam" => {
                self.check_steam_certification();
            }
            "Epic" => {
                self.check_epic_certification();
            }
            _ => {
                return Err(format!("Unknown platform: {}", self.platform));
            }
        }

        self.certified = self.errors.is_empty();
        Ok(self.certified)
    }

    fn check_ps5_certification(&mut self) {
        // PS5特定检查
        if !self.has_trophy_support() {
            self.errors.push("Missing trophy support".to_string());
        }
        if !self.has_ps5_features() {
            self.warnings.push("PS5 features not fully utilized".to_string());
        }
    }

    fn check_xbox_certification(&mut self) {
        if !self.has_achievement_support() {
            self.errors.push("Missing achievement support".to_string());
        }
    }

    fn check_switch_certification(&mut self) {
        if !self.meets_memory_requirements() {
            self.errors.push("Exceeds memory limits".to_string());
        }
    }

    fn check_steam_certification(&mut self) {
        if !self.has_cloud_save() {
            self.warnings.push("Cloud save not implemented".to_string());
        }
    }

    fn check_epic_certification(&mut self) {
        if !self.has_crossplay() {
            self.warnings.push("Crossplay not supported".to_string());
        }
    }

    // Mock辅助方法
    fn has_trophy_support(&self) -> bool {
        true
    }

    fn has_ps5_features(&self) -> bool {
        false
    }

    fn has_achievement_support(&self) -> bool {
        true
    }

    fn meets_memory_requirements(&self) -> bool {
        true
    }

    fn has_cloud_save(&self) -> bool {
        false
    }

    fn has_crossplay(&self) -> bool {
        false
    }

    pub fn add_custom_rule(&mut self, rule: String) {
        // 自定义规则检查
        if !self.check_rule(&rule) {
            self.errors.push(format!("Rule check failed: {}", rule));
        }
    }

    fn check_rule(&self, _rule: &str) -> bool {
        true
    }
}

/// Mock控制器
#[derive(Debug, Clone)]
pub struct MockController {
    pub id: u32,
    pub platform: String,
    pub state: TestControllerState,
    pub vibration_supported: bool,
    pub led_supported: bool,
    pub touchpad_supported: bool,
    pub motion_supported: bool,
    pub haptic_supported: bool,
    pub adaptive_triggers_supported: bool,
}

impl MockController {
    pub fn new(id: u32, platform: &str) -> Self {
        let (vibration, led, touchpad, motion, haptic, adaptive) = match platform {
            "PS5" => (true, true, true, true, true, true),
            "PS4" => (true, true, true, true, false, false),
            "Xbox" => (true, false, false, false, false, false),
            "Switch" => (true, false, false, true, false, false),
            _ => (false, false, false, false, false, false),
        };

        Self {
            id,
            platform: platform.to_string(),
            state: TestControllerState::new(),
            vibration_supported: vibration,
            led_supported: led,
            touchpad_supported: touchpad,
            motion_supported: motion,
            haptic_supported: haptic,
            adaptive_triggers_supported: adaptive,
        }
    }

    pub fn set_button(&mut self, button: &str, pressed: bool) {
        self.state.buttons.insert(button.to_string(), pressed);
    }

    pub fn set_axis(&mut self, axis: &str, value: f32) {
        self.state.axes.insert(axis.to_string(), value);
    }

    pub fn vibrate(&self, _low_frequency: f32, _high_frequency: f32) -> Result<(), String> {
        if !self.vibration_supported {
            return Err("Vibration not supported".to_string());
        }
        Ok(())
    }

    pub fn set_led_color(&self, _r: u8, _g: u8, _b: u8) -> Result<(), String> {
        if !self.led_supported {
            return Err("LED not supported".to_string());
        }
        Ok(())
    }

    pub fn set_haptic(&self, _position: [f32; 2], _strength: f32) -> Result<(), String> {
        if !self.haptic_supported {
            return Err("Haptic feedback not supported".to_string());
        }
        Ok(())
    }

    pub fn set_adaptive_trigger(
        &self,
        _trigger: &str,
        _position: f32,
        _resistance: f32,
    ) -> Result<(), String> {
        if !self.adaptive_triggers_supported {
            return Err("Adaptive triggers not supported".to_string());
        }
        Ok(())
    }
}

/// Mock GPU管理器
#[derive(Debug, Clone)]
pub struct MockGPUManager {
    pub gpu_info: TestGPUInfo,
    pub vram_used_mb: u32,
    pub vram_total_mb: u32,
    pub frustum_culling_enabled: bool,
    pub occlusion_culling_enabled: bool,
    pub distance_culling_enabled: bool,
    pub indirect_draw_enabled: bool,
}

impl MockGPUManager {
    pub fn new(gpu_info: TestGPUInfo) -> Self {
        let vram_total = gpu_info.memory_mb;
        Self {
            gpu_info,
            vram_used_mb: 0,
            vram_total_mb: vram_total,
            frustum_culling_enabled: true,
            occlusion_culling_enabled: false,
            distance_culling_enabled: true,
            indirect_draw_enabled: false,
        }
    }

    pub fn allocate_vram(&mut self, size_mb: u32) -> Result<(), String> {
        if self.vram_used_mb + size_mb > self.vram_total_mb {
            return Err("Insufficient VRAM".to_string());
        }
        self.vram_used_mb += size_mb;
        Ok(())
    }

    pub fn free_vram(&mut self, size_mb: u32) {
        self.vram_used_mb = self.vram_used_mb.saturating_sub(size_mb);
    }

    pub fn get_vram_usage(&self) -> f32 {
        self.vram_used_mb as f32 / self.vram_total_mb as f32
    }

    pub fn enable_feature(&mut self, feature: &str) -> Result<(), String> {
        match feature {
            "frustum_culling" => self.frustum_culling_enabled = true,
            "occlusion_culling" => {
                if !self.gpu_info.supports_raytracing {
                    return Err("Occlusion culling requires raytracing support".to_string());
                }
                self.occlusion_culling_enabled = true;
            }
            "distance_culling" => self.distance_culling_enabled = true,
            "indirect_draw" => {
                if !self.gpu_info.supports_mesh_shaders {
                    return Err("Indirect draw requires mesh shader support".to_string());
                }
                self.indirect_draw_enabled = true;
            }
            _ => return Err(format!("Unknown feature: {}", feature)),
        }
        Ok(())
    }

    pub fn disable_feature(&mut self, feature: &str) -> Result<(), String> {
        match feature {
            "frustum_culling" => self.frustum_culling_enabled = false,
            "occlusion_culling" => self.occlusion_culling_enabled = false,
            "distance_culling" => self.distance_culling_enabled = false,
            "indirect_draw" => self.indirect_draw_enabled = false,
            _ => return Err(format!("Unknown feature: {}", feature)),
        }
        Ok(())
    }

    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "frustum_culling" => self.frustum_culling_enabled,
            "occlusion_culling" => self.occlusion_culling_enabled,
            "distance_culling" => self.distance_culling_enabled,
            "indirect_draw" => self.indirect_draw_enabled,
            _ => false,
        }
    }
}

/// Mock平台管理器
#[derive(Debug, Clone)]
pub struct MockPlatformManager {
    pub platforms: HashMap<String, TestPlatformInfo>,
    pub active_platform: Option<String>,
}

impl MockPlatformManager {
    pub fn new() -> Self {
        let mut platforms = HashMap::new();

        platforms.insert(
            "PS5".to_string(),
            TestPlatformInfo::new("PS5")
                .with_capability("haptic_feedback")
                .with_capability("adaptive_triggers")
                .with_capability("raytracing"),
        );

        platforms.insert(
            "Xbox".to_string(),
            TestPlatformInfo::new("Xbox")
                .with_capability("achievements")
                .with_capability("cloud_save"),
        );

        platforms.insert(
            "Switch".to_string(),
            TestPlatformInfo::new("Switch").with_capability("motion_controls"),
        );

        platforms.insert(
            "Steam".to_string(),
            TestPlatformInfo::new("Steam")
                .with_capability("achievements")
                .with_capability("workshop"),
        );

        platforms.insert(
            "Epic".to_string(),
            TestPlatformInfo::new("Epic").with_capability("crossplay"),
        );

        Self {
            platforms,
            active_platform: None,
        }
    }

    pub fn get_platform(&self, name: &str) -> Option<&TestPlatformInfo> {
        self.platforms.get(name)
    }

    pub fn set_active_platform(&mut self, name: &str) -> Result<(), String> {
        if self.platforms.contains_key(name) {
            self.active_platform = Some(name.to_string());
            Ok(())
        } else {
            Err(format!("Unknown platform: {}", name))
        }
    }

    pub fn get_active_platform(&self) -> Option<&TestPlatformInfo> {
        self.active_platform
            .as_ref()
            .and_then(|name| self.platforms.get(name))
    }

    pub fn list_platforms(&self) -> Vec<String> {
        self.platforms.keys().cloned().collect()
    }
}

impl Default for MockPlatformManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_certification_system() {
        let mut cert_system = MockCertificationSystem::new("PS5");
        let result = cert_system.check_certification().unwrap();
        assert!(result);
        assert!(!cert_system.errors.is_empty() || cert_system.warnings.len() > 0);
    }

    #[test]
    fn test_mock_controller() {
        let controller = MockController::new(0, "PS5");
        assert!(controller.vibration_supported);
        assert!(controller.haptic_supported);
        assert!(controller.adaptive_triggers_supported);
    }

    #[test]
    fn test_mock_gpu_manager() {
        let gpu_info = TestGPUInfo::new("RTX 3080", "NVIDIA")
            .with_raytracing()
            .with_mesh_shaders();
        let mut gpu_manager = MockGPUManager::new(gpu_info);

        assert_eq!(gpu_manager.allocate_vram(1000), Ok(()));
        assert_eq!(gpu_manager.vram_used_mb, 1000);

        gpu_manager.free_vram(500);
        assert_eq!(gpu_manager.vram_used_mb, 500);

        assert_eq!(gpu_manager.get_vram_usage(), 500.0 / 4096.0);
    }

    #[test]
    fn test_mock_platform_manager() {
        let manager = MockPlatformManager::new();
        assert_eq!(manager.list_platforms().len(), 5);

        manager.set_active_platform("PS5").unwrap();
        let active = manager.get_active_platform().unwrap();
        assert_eq!(active.platform_type, "PS5");
        assert!(active.has_capability("haptic_feedback"));
    }

    #[test]
    fn test_controller_platform_differences() {
        let ps5_controller = MockController::new(0, "PS5");
        let xbox_controller = MockController::new(1, "Xbox");

        assert!(ps5_controller.led_supported);
        assert!(!xbox_controller.led_supported);

        assert!(ps5_controller.haptic_supported);
        assert!(!xbox_controller.haptic_supported);
    }
}
