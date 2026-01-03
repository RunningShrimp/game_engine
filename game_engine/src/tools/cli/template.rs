//! # Template Management
//!
//! Defines and manages project templates for scaffolding.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Available project templates
///
/// Each template provides a complete starter project with predefined structure,
/// assets, scripts, and configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProjectTemplate {
    /// Basic game template with minimal setup
    ///
    /// Includes:
    /// - Basic window and rendering setup
    /// - Simple game loop
    /// - Minimal ECS setup
    /// - Example scene
    Basic,

    /// 2D Platformer game template
    ///
    /// Includes:
    /// - 2D rendering system
    /// - Platform physics
    /// - Player controller
    /// - Tile map support
    /// - Sprite animation system
    /// - Example level
    Platformer2D,

    /// 3D First-Person Shooter template
    ///
    /// Includes:
    /// - 3D rendering with lighting
    /// - First-person camera controller
    /// - Weapon system
    /// - Enemy AI
    /// - 3D physics
    /// - Example arena map
    Fps3D,

    /// 2D Top-Down RPG template
    ///
    /// Includes:
    /// - Top-down camera system
    /// - Tile-based world
    /// - Dialog system
    /// - Inventory system
    /// - Character stats
    /// - Quest system
    TopDownRPG,

    /// 3D Third-Person Action template
    ///
    /// Includes:
    /// - Third-person camera
    /// - Character controller
    /// - Combat system
    /// - Animation blending
    /// - Enemy AI
    /// - 3D environment
    ThirdPersonAction,

    /// Multiplayer Game template
    ///
    /// Includes:
    /// - Networked gameplay
    /// - Player synchronization
    /// - Server-authoritative physics
    /// - Matchmaking system
    /// - Lobby system
    Multiplayer,

    /// VR Experience template
    ///
    /// Includes:
    /// - VR headset support
    /// - Motion controller tracking
    /// - Teleportation system
    /// - VR UI system
    /// - Performance optimization
    VirtualReality,
}

impl ProjectTemplate {
    /// Returns all available templates
    pub fn all() -> Vec<Self> {
        vec![
            Self::Basic,
            Self::Platformer2D,
            Self::Fps3D,
            Self::TopDownRPG,
            Self::ThirdPersonAction,
            Self::Multiplayer,
            Self::VirtualReality,
        ]
    }

    /// Returns the template name as a string
    pub fn name(&self) -> &str {
        match self {
            Self::Basic => "basic",
            Self::Platformer2D => "2d-platformer",
            Self::Fps3D => "3d-fps",
            Self::TopDownRPG => "2d-rpg",
            Self::ThirdPersonAction => "3d-action",
            Self::Multiplayer => "multiplayer",
            Self::VirtualReality => "vr",
        }
    }

    /// Returns a human-readable description of the template
    pub fn description(&self) -> &str {
        match self {
            Self::Basic => "Basic game template with minimal setup",
            Self::Platformer2D => "2D platformer game with physics, tile maps, and sprites",
            Self::Fps3D => "3D first-person shooter with lighting, weapons, and AI",
            Self::TopDownRPG => "2D top-down RPG with dialogs, inventory, and quests",
            Self::ThirdPersonAction => "3D third-person action game with combat and animations",
            Self::Multiplayer => "Multiplayer game with networking and matchmaking",
            Self::VirtualReality => "VR experience with headset and controller support",
        }
    }

    /// Returns template directory name
    pub fn dir_name(&self) -> &str {
        self.name()
    }

    /// Parses a template name string into a ProjectTemplate
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "basic" => Some(Self::Basic),
            "2d-platformer" => Some(Self::Platformer2D),
            "3d-fps" => Some(Self::Fps3D),
            "2d-rpg" => Some(Self::TopDownRPG),
            "3d-action" => Some(Self::ThirdPersonAction),
            "multiplayer" => Some(Self::Multiplayer),
            "vr" => Some(Self::VirtualReality),
            _ => None,
        }
    }

    /// Returns templates by category
    pub fn by_category(category: TemplateCategory) -> Vec<Self> {
        match category {
            TemplateCategory::Starter => vec![Self::Basic],
            TemplateCategory::TwoD => vec![Self::Platformer2D, Self::TopDownRPG],
            TemplateCategory::ThreeD => vec![Self::Fps3D, Self::ThirdPersonAction],
            TemplateCategory::Advanced => vec![Self::Multiplayer, Self::VirtualReality],
        }
    }
}

impl fmt::Display for ProjectTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Template categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TemplateCategory {
    /// Starter templates
    Starter,
    /// 2D games
    TwoD,
    /// 3D games
    ThreeD,
    /// Advanced features
    Advanced,
}

/// Template metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template version
    pub version: String,
    /// Required engine features
    pub required_features: Vec<String>,
    /// Template categories
    pub categories: Vec<String>,
    /// Tags for search
    pub tags: Vec<String>,
    /// Estimated difficulty (1-5)
    pub difficulty: u8,
    /// Estimated setup time in minutes
    pub setup_time_minutes: u32,
}

impl TemplateMetadata {
    /// Creates metadata for a template
    pub fn new(template: &ProjectTemplate) -> Self {
        let (required_features, categories, tags, difficulty, setup_time) = match template {
            ProjectTemplate::Basic => (
                vec!["rendering".to_string(), "ecs".to_string()],
                vec!["starter".to_string(), "minimal".to_string()],
                vec![
                    "basic".to_string(),
                    "simple".to_string(),
                    "starter".to_string(),
                ],
                1,
                5,
            ),
            ProjectTemplate::Platformer2D => (
                vec![
                    "rendering".to_string(),
                    "ecs".to_string(),
                    "physics".to_string(),
                    "2d".to_string(),
                ],
                vec!["2d".to_string(), "platformer".to_string()],
                vec![
                    "platformer".to_string(),
                    "2d".to_string(),
                    "physics".to_string(),
                    "tiles".to_string(),
                ],
                2,
                15,
            ),
            ProjectTemplate::Fps3D => (
                vec![
                    "rendering".to_string(),
                    "ecs".to_string(),
                    "physics".to_string(),
                    "3d".to_string(),
                    "ai".to_string(),
                ],
                vec!["3d".to_string(), "fps".to_string(), "shooter".to_string()],
                vec![
                    "fps".to_string(),
                    "3d".to_string(),
                    "shooter".to_string(),
                    "ai".to_string(),
                ],
                4,
                30,
            ),
            ProjectTemplate::TopDownRPG => (
                vec![
                    "rendering".to_string(),
                    "ecs".to_string(),
                    "2d".to_string(),
                    "ui".to_string(),
                ],
                vec!["2d".to_string(), "rpg".to_string()],
                vec![
                    "rpg".to_string(),
                    "2d".to_string(),
                    "top-down".to_string(),
                    "inventory".to_string(),
                    "quests".to_string(),
                ],
                3,
                25,
            ),
            ProjectTemplate::ThirdPersonAction => (
                vec![
                    "rendering".to_string(),
                    "ecs".to_string(),
                    "physics".to_string(),
                    "3d".to_string(),
                    "animation".to_string(),
                ],
                vec!["3d".to_string(), "action".to_string()],
                vec![
                    "action".to_string(),
                    "3d".to_string(),
                    "third-person".to_string(),
                    "combat".to_string(),
                ],
                4,
                35,
            ),
            ProjectTemplate::Multiplayer => (
                vec![
                    "rendering".to_string(),
                    "ecs".to_string(),
                    "networking".to_string(),
                ],
                vec!["multiplayer".to_string(), "networking".to_string()],
                vec![
                    "multiplayer".to_string(),
                    "networking".to_string(),
                    "online".to_string(),
                ],
                5,
                45,
            ),
            ProjectTemplate::VirtualReality => (
                vec![
                    "rendering".to_string(),
                    "ecs".to_string(),
                    "vr".to_string(),
                    "physics".to_string(),
                ],
                vec!["vr".to_string(), "advanced".to_string()],
                vec![
                    "vr".to_string(),
                    "virtual-reality".to_string(),
                    "headset".to_string(),
                ],
                5,
                40,
            ),
        };

        Self {
            name: template.name().to_string(),
            description: template.description().to_string(),
            version: "0.1.0".to_string(),
            required_features,
            categories,
            tags,
            difficulty,
            setup_time_minutes: setup_time,
        }
    }
}

/// Template registry managing all available templates
pub struct TemplateRegistry {
    templates: HashMap<String, TemplateMetadata>,
}

impl TemplateRegistry {
    /// Creates a new template registry with all built-in templates
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        for template in ProjectTemplate::all() {
            let metadata = TemplateMetadata::new(&template);
            templates.insert(template.name().to_string(), metadata);
        }

        Self { templates }
    }

    /// Returns metadata for a specific template
    pub fn get(&self, name: &str) -> Option<&TemplateMetadata> {
        self.templates.get(name)
    }

    /// Returns all available templates
    pub fn list_all(&self) -> Vec<&TemplateMetadata> {
        self.templates.values().collect()
    }

    /// Searches templates by category or tag
    pub fn search(&self, query: &str) -> Vec<&TemplateMetadata> {
        let query_lower = query.to_lowercase();
        self.templates
            .values()
            .filter(|metadata| {
                metadata.name.to_lowercase().contains(&query_lower)
                    || metadata.description.to_lowercase().contains(&query_lower)
                    || metadata.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
                    || metadata
                        .categories
                        .iter()
                        .any(|cat| cat.to_lowercase().contains(&query_lower))
            })
            .collect()
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_names() {
        assert_eq!(ProjectTemplate::Basic.name(), "basic");
        assert_eq!(ProjectTemplate::Platformer2D.name(), "2d-platformer");
        assert_eq!(ProjectTemplate::Fps3D.name(), "3d-fps");
    }

    #[test]
    fn test_template_from_name() {
        assert_eq!(
            ProjectTemplate::from_name("basic"),
            Some(ProjectTemplate::Basic)
        );
        assert_eq!(
            ProjectTemplate::from_name("2d-platformer"),
            Some(ProjectTemplate::Platformer2D)
        );
        assert_eq!(
            ProjectTemplate::from_name("3d-fps"),
            Some(ProjectTemplate::Fps3D)
        );
        assert_eq!(ProjectTemplate::from_name("invalid"), None);
    }

    #[test]
    fn test_template_registry() {
        let registry = TemplateRegistry::new();

        // Check that all templates are registered
        assert_eq!(registry.templates.len(), 3);

        // Check getting specific templates
        assert!(registry.get("basic").is_some());
        assert!(registry.get("invalid").is_none());

        // Check listing
        let all = registry.list_all();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_template_search() {
        let registry = TemplateRegistry::new();

        // Search by name
        let results = registry.search("basic");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "basic");

        // Search by tag
        let results = registry.search("2d");
        assert!(results.len() >= 1);

        // Search by category
        let results = registry.search("platformer");
        assert!(results.len() >= 1);
    }
}
