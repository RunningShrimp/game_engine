// 版本管理和迁移系统
//
// 提供版本控制和数据迁移功能。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 版本号
pub type Version = u32;

/// 版本迁移函数类型
pub type MigrationFn = fn(serde_json::Value) -> Result<serde_json::Value, String>;

/// 版本迁移规则
#[derive(Clone)]
pub struct MigrationRule {
    /// 从版本
    pub from_version: Version,
    /// 到版本
    pub to_version: Version,
    /// 迁移函数
    pub migrate_fn: MigrationFn,
    /// 描述
    pub description: String,
}

/// 版本管理器
///
/// 管理数据版本和迁移规则。
#[derive(Clone)]
pub struct VersionManager {
    /// 当前版本
    current_version: Version,
    /// 迁移规则映射: from_version -> rule
    migration_rules: HashMap<Version, MigrationRule>,
}

impl VersionManager {
    /// 创建新的版本管理器
    pub fn new(current_version: Version) -> Self {
        Self {
            current_version,
            migration_rules: HashMap::new(),
        }
    }

    /// 添加迁移规则
    pub fn add_migration_rule(&mut self, rule: MigrationRule) {
        self.migration_rules.insert(rule.from_version, rule);
    }

    /// 执行版本迁移
    ///
    /// 将旧版本数据迁移到当前版本
    pub fn migrate(
        &self,
        mut data: serde_json::Value,
        from_version: Version,
    ) -> Result<serde_json::Value, String> {
        let mut current = from_version;

        while current < self.current_version {
            let rule = self
                .migration_rules
                .get(&current)
                .ok_or_else(|| format!("No migration rule found for version {}", current))?;

            tracing::info!(
                "Migrating from version {} to {}",
                current,
                rule.to_version
            );

            data = (rule.migrate_fn)(data)?;
            current = rule.to_version;
        }

        Ok(data)
    }

    /// 获取当前版本
    pub fn current_version(&self) -> Version {
        self.current_version
    }

    /// 检查是否需要迁移
    pub fn needs_migration(&self, version: Version) -> bool {
        version < self.current_version
    }
}

/// 版本化数据
///
/// 包装数据并附加版本信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedData<T> {
    /// 数据版本
    pub version: Version,
    /// 实际数据
    pub data: T,
}

impl<T> VersionedData<T> {
    /// 创建新的版本化数据
    pub fn new(version: Version, data: T) -> Self {
        Self { version, data }
    }

    /// 获取数据
    pub fn get(&self) -> &T {
        &self.data
    }

    /// 获取可变数据
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// 解构数据
    pub fn into_inner(self) -> T {
        self.data
    }
}

/// 兼容性检查器
///
/// 检查数据版本兼容性。
pub struct CompatibilityChecker {
    /// 最低兼容版本
    min_compatible_version: Version,
    /// 当前版本
    current_version: Version,
}

impl CompatibilityChecker {
    /// 创建新的兼容性检查器
    pub fn new(min_compatible_version: Version, current_version: Version) -> Self {
        Self {
            min_compatible_version,
            current_version,
        }
    }

    /// 检查版本是否兼容
    pub fn is_compatible(&self, version: Version) -> bool {
        version >= self.min_compatible_version && version <= self.current_version
    }

    /// 检查是否需要升级
    pub fn needs_upgrade(&self, version: Version) -> bool {
        version < self.current_version
    }

    /// 检查是否来自未来版本（不兼容）
    pub fn is_from_future(&self, version: Version) -> bool {
        version > self.current_version
    }

    /// 获取兼容性信息
    pub fn compatibility_info(&self, version: Version) -> CompatibilityInfo {
        CompatibilityInfo {
            version,
            is_compatible: self.is_compatible(version),
            needs_upgrade: self.needs_upgrade(version),
            is_from_future: self.is_from_future(version),
            min_compatible_version: self.min_compatible_version,
            current_version: self.current_version,
        }
    }
}

/// 兼容性信息
#[derive(Debug, Clone)]
pub struct CompatibilityInfo {
    /// 检查的版本
    pub version: Version,
    /// 是否兼容
    pub is_compatible: bool,
    /// 是否需要升级
    pub needs_upgrade: bool,
    /// 是否来自未来版本
    pub is_from_future: bool,
    /// 最低兼容版本
    pub min_compatible_version: Version,
    /// 当前版本
    pub current_version: Version,
}

/// 语义化版本
///
/// 使用major.minor.patch格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SemanticVersion {
    /// 主版本号（不兼容的API更改）
    pub major: u32,
    /// 次版本号（向后兼容的功能添加）
    pub minor: u32,
    /// 补丁版本号（向后兼容的问题修复）
    pub patch: u32,
}

impl SemanticVersion {
    /// 创建新的语义化版本
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// 从字符串解析
    ///
    /// 格式: "major.minor.patch"
    pub fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid semantic version: {}", s));
        }

        let major = parts[0]
            .parse()
            .map_err(|_| format!("Invalid major version: {}", parts[0]))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| format!("Invalid minor version: {}", parts[1]))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| format!("Invalid patch version: {}", parts[2]))?;

        Ok(Self::new(major, minor, patch))
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// 检查兼容性（major版本必须相同）
    pub fn is_compatible(&self, other: &Self) -> bool {
        self.major == other.major
    }

    /// 增加主版本号
    pub fn bump_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }

    /// 增加次版本号
    pub fn bump_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    /// 增加补丁版本号
    pub fn bump_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_version_manager() {
        let mut manager = VersionManager::new(3);

        // 添加迁移规则: 0 -> 1
        manager.add_migration_rule(MigrationRule {
            from_version: 0,
            to_version: 1,
            migrate_fn: |data| {
                let mut obj = data.as_object().unwrap().clone();
                obj.insert("v1_field".to_string(), json!("added_in_v1"));
                Ok(serde_json::Value::Object(obj))
            },
            description: "Migrate v0 to v1".to_string(),
        });

        // 添加迁移规则: 1 -> 2
        manager.add_migration_rule(MigrationRule {
            from_version: 1,
            to_version: 2,
            migrate_fn: |data| {
                let mut obj = data.as_object().unwrap().clone();
                obj.insert("v2_field".to_string(), json!("added_in_v2"));
                Ok(serde_json::Value::Object(obj))
            },
            description: "Migrate v1 to v2".to_string(),
        });

        // 添加迁移规则: 2 -> 3
        manager.add_migration_rule(MigrationRule {
            from_version: 2,
            to_version: 3,
            migrate_fn: |data| {
                let mut obj = data.as_object().unwrap().clone();
                obj.insert("v3_field".to_string(), json!("added_in_v3"));
                Ok(serde_json::Value::Object(obj))
            },
            description: "Migrate v2 to v3".to_string(),
        });

        // 测试迁移
        let v0_data = json!({"old_field": "value"});
        let migrated = manager.migrate(v0_data, 0).unwrap();

        assert_eq!(migrated["old_field"], "value");
        assert_eq!(migrated["v1_field"], "added_in_v1");
        assert_eq!(migrated["v2_field"], "added_in_v2");
        assert_eq!(migrated["v3_field"], "added_in_v3");
    }

    #[test]
    fn test_versioned_data() {
        let data = VersionedData::new(1, "test data".to_string());
        assert_eq!(data.version, 1);
        assert_eq!(data.get(), "test data");

        let mut data = data;
        data.get_mut().push_str(" (modified)");
        assert_eq!(data.get(), "test data (modified)");
    }

    #[test]
    fn test_compatibility_checker() {
        let checker = CompatibilityChecker::new(2, 5);

        // 测试兼容版本
        assert!(checker.is_compatible(2));
        assert!(checker.is_compatible(3));
        assert!(checker.is_compatible(4));
        assert!(checker.is_compatible(5));

        // 测试不兼容版本
        assert!(!checker.is_compatible(1)); // 太旧
        assert!(!checker.is_compatible(6)); // 太新

        // 测试升级需求
        assert!(!checker.needs_upgrade(5));
        assert!(checker.needs_upgrade(2));

        // 测试未来版本
        assert!(checker.is_from_future(6));
        assert!(!checker.is_from_future(5));
    }

    #[test]
    fn test_semantic_version() {
        let v1 = SemanticVersion::new(1, 2, 3);
        let v2 = SemanticVersion::new(1, 2, 4);
        let v3 = SemanticVersion::new(2, 0, 0);

        assert_eq!(v1.to_string(), "1.2.3");

        // 测试比较
        assert!(v2 > v1);
        assert!(v3 > v1);

        // 测试兼容性
        assert!(v1.is_compatible(&v2)); // major相同
        assert!(!v1.is_compatible(&v3)); // major不同

        // 测试版本号增加
        assert_eq!(v1.bump_patch(), SemanticVersion::new(1, 2, 4));
        assert_eq!(v1.bump_minor(), SemanticVersion::new(1, 3, 0));
        assert_eq!(v1.bump_major(), SemanticVersion::new(2, 0, 0));

        // 测试解析
        let parsed = SemanticVersion::from_str("2.5.10").unwrap();
        assert_eq!(parsed.major, 2);
        assert_eq!(parsed.minor, 5);
        assert_eq!(parsed.patch, 10);
    }
}
