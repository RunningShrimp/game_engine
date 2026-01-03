use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::sync::Mutex;
use tauri::State;
use chrono::{DateTime, Utc};

// 教程系统状态
pub struct TutorialState {
    tutorials: Mutex<HashMap<String, Tutorial>>,
    user_progress: Mutex<HashMap<String, UserProgress>>,
    user_stats: Mutex<HashMap<String, UserStats>>,
}

// 核心数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tutorial {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: TutorialCategory,
    pub difficulty: TutorialDifficulty,
    pub estimated_time: u32, // 分钟
    pub prerequisites: Vec<String>,
    pub skills: Vec<String>,
    pub xp_reward: u32,
    pub badges: Vec<String>,
    pub steps: Vec<TutorialStep>,
    pub challenges: Option<Vec<TutorialChallenge>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TutorialCategory {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TutorialDifficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialStep {
    pub id: String,
    pub title: String,
    pub content: String, // Markdown/HTML
    #[serde(rename = "type")]
    pub step_type: StepType,
    pub code_template: Option<String>,
    pub expected_output: Option<String>,
    pub hints: Vec<String>,
    pub verify_fn: Option<String>,
    pub resources: Option<Vec<StepResource>>,
    pub order: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepType {
    Concept,
    Demo,
    Exercise,
    Challenge,
    Quiz,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResource {
    #[serde(rename = "type")]
    pub resource_type: ResourceType,
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Image,
    Video,
    Code,
    Document,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialChallenge {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub challenge_type: ChallengeType,
    pub difficulty: TutorialDifficulty,
    pub time_limit: Option<u32>, // 秒
    pub starter_code: Option<String>,
    pub solution: Option<String>,
    pub test_cases: Option<Vec<TestCase>>,
    pub xp_reward: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChallengeType {
    FillBlank,
    Debug,
    Implement,
    Optimization,
    Creative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub input: serde_json::Value,
    pub expected_output: serde_json::Value,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProgress {
    pub user_id: String,
    pub tutorial_id: String,
    pub current_step: String,
    pub completed_steps: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub time_spent: u64, // 秒
    pub attempts: u32,
    pub hints_used: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub user_id: String,
    pub total_xp: u32,
    pub level: u32,
    pub current_level_xp: u32,
    pub next_level_xp: u32,
    pub completed_tutorials: Vec<String>,
    pub in_progress_tutorials: Vec<String>,
    pub badges: Vec<Badge>,
    pub skills: Vec<SkillProgress>,
    pub streak_days: u32,
    pub last_active_date: DateTime<Utc>,
    pub achievements: Vec<Achievement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub rarity: BadgeRarity,
    pub earned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BadgeRarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProgress {
    pub name: String,
    pub level: u32,
    pub progress: u32, // 0-100
    pub tutorials_completed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub xp_reward: u32,
    pub unlocked_at: DateTime<Utc>,
    pub progress: Option<u32>,
    pub total: Option<u32>,
}

// Tauri 命令实现

#[tauri::command]
pub async fn get_tutorials(
    state: State<'_, TutorialState>,
) -> Result<Vec<Tutorial>, String> {
    let tutorials = state.tutorials.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    Ok(tutorials.values().cloned().collect())
}

#[tauri::command]
pub async fn get_tutorial(
    tutorial_id: String,
    state: State<'_, TutorialState>,
) -> Result<Tutorial, String> {
    let tutorials = state.tutorials.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    tutorials.get(&tutorial_id)
        .cloned()
        .ok_or_else(|| format!("Tutorial not found: {}", tutorial_id))
}

#[tauri::command]
pub async fn create_tutorial(
    tutorial: Tutorial,
    state: State<'_, TutorialState>,
) -> Result<Tutorial, String> {
    let mut tutorials = state.tutorials.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    tutorials.insert(tutorial.id.clone(), tutorial.clone());
    Ok(tutorial)
}

#[tauri::command]
pub async fn update_tutorial(
    tutorial_id: String,
    updates: Tutorial,
    state: State<'_, TutorialState>,
) -> Result<Tutorial, String> {
    let mut tutorials = state.tutorials.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if !tutorials.contains_key(&tutorial_id) {
        return Err(format!("Tutorial not found: {}", tutorial_id));
    }

    let mut updated = updates;
    updated.updated_at = Utc::now();
    tutorials.insert(tutorial_id, updated.clone());

    Ok(updated)
}

#[tauri::command]
pub async fn delete_tutorial(
    tutorial_id: String,
    state: State<'_, TutorialState>,
) -> Result<(), String> {
    let mut tutorials = state.tutorials.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    tutorials.remove(&tutorial_id)
        .ok_or_else(|| format!("Tutorial not found: {}", tutorial_id))?;

    Ok(())
}

#[tauri::command]
pub async fn get_tutorial_progress(
    tutorial_id: String,
    user_id: String,
    state: State<'_, TutorialState>,
) -> Result<UserProgress, String> {
    let progress = state.user_progress.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    progress.get(&format!("{}_{}", user_id, tutorial_id))
        .cloned()
        .ok_or_else(|| format!("Progress not found for tutorial: {}", tutorial_id))
}

#[tauri::command]
pub async fn start_tutorial(
    tutorial_id: String,
    user_id: String,
    state: State<'_, TutorialState>,
) -> Result<(), String> {
    let tutorials = state.tutorials.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let tutorial = tutorials.get(&tutorial_id)
        .ok_or_else(|| format!("Tutorial not found: {}", tutorial_id))?;

    let first_step = tutorial.steps.first()
        .ok_or_else(|| format!("Tutorial has no steps: {}", tutorial_id))?;

    let mut progress = state.user_progress.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    progress.insert(format!("{}_{}", user_id, tutorial_id), UserProgress {
        user_id: user_id.clone(),
        tutorial_id,
        current_step: first_step.id.clone(),
        completed_steps: Vec::new(),
        started_at: Utc::now(),
        completed_at: None,
        time_spent: 0,
        attempts: 0,
        hints_used: 0,
    });

    Ok(())
}

#[tauri::command]
pub async fn complete_tutorial_step(
    tutorial_id: String,
    step_id: String,
    user_id: String,
    state: State<'_, TutorialState>,
) -> Result<(), String> {
    let key = format!("{}_{}", user_id, tutorial_id);
    let mut progress = state.user_progress.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let entry = progress.get_mut(&key)
        .ok_or_else(|| format!("Progress not found"))?;

    if !entry.completed_steps.contains(&step_id) {
        entry.completed_steps.push(step_id);
    }

    Ok(())
}

#[tauri::command]
pub async fn save_tutorial_progress(
    tutorial_id: String,
    user_id: String,
    updates: serde_json::Value,
    state: State<'_, TutorialState>,
) -> Result<(), String> {
    let key = format!("{}_{}", user_id, tutorial_id);
    let mut progress = state.user_progress.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let entry = progress.get_mut(&key)
        .ok_or_else(|| format!("Progress not found"))?;

    // 更新字段
    if let Some(current_step) = updates.get("currentStep") {
        if let Some(s) = current_step.as_str() {
            entry.current_step = s.to_string();
        }
    }

    if let Some(time_spent) = updates.get("timeSpent") {
        if let Some(t) = time_spent.as_u64() {
            entry.time_spent = t;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn complete_tutorial(
    tutorial_id: String,
    user_id: String,
    state: State<'_, TutorialState>,
) -> Result<u32, String> {
    // 获取教程信息
    let tutorials = state.tutorials.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let tutorial = tutorials.get(&tutorial_id)
        .ok_or_else(|| format!("Tutorial not found: {}", tutorial_id))?;

    let xp_reward = tutorial.xp_reward;

    // 更新进度
    let key = format!("{}_{}", user_id, tutorial_id);
    let mut progress = state.user_progress.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if let Some(entry) = progress.get_mut(&key) {
        entry.completed_at = Some(Utc::now());
    }

    // 更新用户统计
    let mut stats = state.user_stats.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let user_stat = stats.entry(user_id.clone()).or_insert_with(|| UserStats {
        user_id: user_id.clone(),
        total_xp: 0,
        level: 1,
        current_level_xp: 0,
        next_level_xp: 100,
        completed_tutorials: Vec::new(),
        in_progress_tutorials: Vec::new(),
        badges: Vec::new(),
        skills: Vec::new(),
        streak_days: 0,
        last_active_date: Utc::now(),
        achievements: Vec::new(),
    });

    // 添加经验值
    user_stat.total_xp += xp_reward;
    user_stat.current_level_xp += xp_reward;

    // 检查升级
    while user_stat.current_level_xp >= user_stat.next_level_xp {
        user_stat.current_level_xp -= user_stat.next_level_xp;
        user_stat.level += 1;
        user_stat.next_level_xp = (user_stat.next_level_xp as f64 * 1.5) as u32;
    }

    // 添加到完成列表
    if !user_stat.completed_tutorials.contains(&tutorial_id) {
        user_stat.completed_tutorials.push(tutorial_id);
    }

    // 从进行中移除
    user_stat.in_progress_tutorials.retain(|t| t != &tutorial_id);

    // 更新技能
    for skill_name in &tutorial.skills {
        let skill = user_stat.skills.iter_mut()
            .find(|s| &s.name == skill_name);

        if let Some(s) = skill {
            if !s.tutorials_completed.contains(&tutorial_id) {
                s.tutorials_completed.push(tutorial_id.clone());
                s.progress = std::cmp::min(100, s.progress + 20);

                if s.progress >= 100 {
                    s.level += 1;
                    s.progress = 0;
                }
            }
        } else {
            user_stat.skills.push(SkillProgress {
                name: skill_name.clone(),
                level: 1,
                progress: 20,
                tutorials_completed: vec![tutorial_id.clone()],
            });
        }
    }

    // 检查成就
    check_achievements(user_stat);

    Ok(xp_reward)
}

#[tauri::command]
pub async fn get_user_stats(
    user_id: String,
    state: State<'_, TutorialState>,
) -> Result<UserStats, String> {
    let stats = state.user_stats.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    stats.get(&user_id)
        .cloned()
        .ok_or_else(|| format!("User stats not found"))
}

#[tauri::command]
pub async fn get_leaderboard(
    limit: usize,
    state: State<'_, TutorialState>,
) -> Result<Vec<UserStats>, String> {
    let stats = state.user_stats.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let mut leaderboard: Vec<_> = stats.values().cloned().collect();
    leaderboard.sort_by(|a, b| b.total_xp.cmp(&a.total_xp));
    leaderboard.truncate(limit);

    Ok(leaderboard)
}

#[tauri::command]
pub async fn execute_tutorial_code(
    code: String,
    _language: String,
) -> Result<String, String> {
    // 这是一个简化的实现，实际应该集成一个代码执行沙箱
    // 可以使用 wasmtime 或其他安全的执行环境

    // 暂时返回模拟输出
    if code.contains("println!") {
        // 提取 println! 的内容
        if let Some(start) = code.find("\"") {
            if let Some(end) = code.rfind("\"") {
                return Ok(code[start + 1..end].to_string());
            }
        }
    }

    Ok("代码执行成功".to_string())
}

#[tauri::command]
pub async fn verify_tutorial_answer(
    _tutorial_id: String,
    _step_id: String,
    answer: serde_json::Value,
) -> Result<bool, String> {
    // 这是一个简化的实现
    // 实际应该根据步骤类型和验证函数来检查答案

    // 基本验证：检查答案是否非空
    if let Some(s) = answer.as_str() {
        Ok(!s.trim().is_empty())
    } else if let Some(n) = answer.as_number() {
        Ok(true)
    } else if let Some(b) = answer.as_bool() {
        Ok(b)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn add_user_xp(
    user_id: String,
    amount: u32,
    _source: String,
    state: State<'_, TutorialState>,
) -> Result<UserStats, String> {
    let mut stats = state.user_stats.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let user_stat = stats.entry(user_id.clone()).or_insert_with(|| UserStats {
        user_id: user_id.clone(),
        total_xp: 0,
        level: 1,
        current_level_xp: 0,
        next_level_xp: 100,
        completed_tutorials: Vec::new(),
        in_progress_tutorials: Vec::new(),
        badges: Vec::new(),
        skills: Vec::new(),
        streak_days: 0,
        last_active_date: Utc::now(),
        achievements: Vec::new(),
    });

    user_stat.total_xp += amount;
    user_stat.current_level_xp += amount;

    // 检查升级
    let level_up = if user_stat.current_level_xp >= user_stat.next_level_xp {
        user_stat.current_level_xp -= user_stat.next_level_xp;
        user_stat.level += 1;
        user_stat.next_level_xp = (user_stat.next_level_xp as f64 * 1.5) as u32;
        true
    } else {
        false
    };

    Ok(user_stat.clone())
}

#[tauri::command]
pub async fn award_badge(
    user_id: String,
    badge_id: String,
    state: State<'_, TutorialState>,
) -> Result<Badge, String> {
    let mut stats = state.user_stats.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let user_stat = stats.entry(user_id.clone()).or_insert_with(|| UserStats {
        user_id: user_id.clone(),
        total_xp: 0,
        level: 1,
        current_level_xp: 0,
        next_level_xp: 100,
        completed_tutorials: Vec::new(),
        in_progress_tutorials: Vec::new(),
        badges: Vec::new(),
        skills: Vec::new(),
        streak_days: 0,
        last_active_date: Utc::now(),
        achievements: Vec::new(),
    });

    // 检查是否已有此徽章
    if user_stat.badges.iter().any(|b| b.id == badge_id) {
        return Err("Badge already awarded".to_string());
    }

    // 创建徽章（这里应该从徽章数据库获取）
    let badge = Badge {
        id: badge_id.clone(),
        name: badge_id.clone(),
        description: format!("Badge {}", badge_id),
        icon: "🏆".to_string(),
        rarity: BadgeRarity::Common,
        earned_at: Utc::now(),
    };

    user_stat.badges.push(badge.clone());

    Ok(badge)
}

#[tauri::command]
pub async fn check_user_achievements(
    user_id: String,
    state: State<'_, TutorialState>,
) -> Result<Vec<String>, String> {
    let mut stats = state.user_stats.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let user_stat = stats.entry(user_id.clone()).or_insert_with(|| UserStats {
        user_id: user_id.clone(),
        total_xp: 0,
        level: 1,
        current_level_xp: 0,
        next_level_xp: 100,
        completed_tutorials: Vec::new(),
        in_progress_tutorials: Vec::new(),
        badges: Vec::new(),
        skills: Vec::new(),
        streak_days: 0,
        last_active_date: Utc::now(),
        achievements: Vec::new(),
    });

    check_achievements(user_stat);

    // 返回新解锁的成就ID
    Ok(user_stat.achievements.iter().map(|a| a.id.clone()).collect())
}

fn check_achievements(stats: &mut UserStats) {
    // 检查首次完成教程
    if stats.completed_tutorials.len() == 1 {
        stats.achievements.push(Achievement {
            id: "first_tutorial".to_string(),
            title: "初出茅庐".to_string(),
            description: "完成第一个教程".to_string(),
            icon: "🎓".to_string(),
            xp_reward: 50,
            unlocked_at: Utc::now(),
            progress: Some(1),
            total: Some(1),
        });
    }

    // 检查完成10个教程
    if stats.completed_tutorials.len() >= 10 {
        if !stats.achievements.iter().any(|a| a.id == "ten_tutorials") {
            stats.achievements.push(Achievement {
                id: "ten_tutorials".to_string(),
                title: "勤奋学习".to_string(),
                description: "完成10个教程".to_string(),
                icon: "📚".to_string(),
                xp_reward: 200,
                unlocked_at: Utc::now(),
                progress: Some(stats.completed_tutorials.len() as u32),
                total: Some(10),
            });
        }
    }

    // 检查达到等级10
    if stats.level >= 10 {
        if !stats.achievements.iter().any(|a| a.id == "level_10") {
            stats.achievements.push(Achievement {
                id: "level_10".to_string(),
                title: "渐入佳境".to_string(),
                description: "达到等级10".to_string(),
                icon: "⭐".to_string(),
                xp_reward: 300,
                unlocked_at: Utc::now(),
                progress: Some(stats.level),
                total: Some(10),
            });
        }
    }
}

#[tauri::command]
pub async fn log_tutorial_hint(
    _tutorial_id: String,
    _step_id: String,
    _hint_index: usize,
    _user_id: String,
) -> Result<(), String> {
    // 记录提示使用情况（可用于分析）
    Ok(())
}

#[tauri::command]
pub async fn load_tutorials_from_disk(
    tutorials_dir: String,
    state: State<'_, TutorialState>,
) -> Result<usize, String> {
    let dir = PathBuf::from(tutorials_dir);

    if !dir.exists() {
        return Err("Directory does not exist".to_string());
    }

    let mut count = 0;
    let mut tutorials = state.tutorials.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    for entry in fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read file: {}", e))?;

            let tutorial: Tutorial = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse tutorial: {}", e))?;

            tutorials.insert(tutorial.id.clone(), tutorial);
            count += 1;
        }
    }

    Ok(count)
}

pub fn init_tutorial_system() -> TutorialState {
    TutorialState {
        tutorials: Mutex::new(HashMap::new()),
        user_progress: Mutex::new(HashMap::new()),
        user_stats: Mutex::new(HashMap::new()),
    }
}
