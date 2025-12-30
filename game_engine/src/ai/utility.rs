// 效用AI系统 (Utility AI)
//
// 基于数值决策的AI系统，相比GOAP更灵活

use std::collections::HashMap;

/// 效用值 (0.0-1.0)
pub type UtilityValue = f32;

/// 效用曲线类型
#[derive(Debug, Clone, Copy)]
pub enum CurveType {
    Linear,
    Quadratic,
    Logistic,
    Logit,
    Sinusoidal,
}

/// 效用曲线
pub struct UtilityCurve {
    curve_type: CurveType,
    slope: f32,
    exponent: f32,
    midpoint: f32,
    y_shift: f32,
}

impl UtilityCurve {
    /// 创建新的效用曲线
    pub fn new(curve_type: CurveType) -> Self {
        Self {
            curve_type,
            slope: 1.0,
            exponent: 2.0,
            midpoint: 0.5,
            y_shift: 0.0,
        }
    }

    /// 计算效用值
    pub fn evaluate(&self, input: f32) -> UtilityValue {
        let x = (input - self.midpoint) * self.slope;

        let y = match self.curve_type {
            CurveType::Linear => x,
            CurveType::Quadratic => {
                if x >= 0.0 {
                    x.powf(self.exponent)
                } else {
                    -(-x).powf(self.exponent)
                }
            }
            CurveType::Logistic => 1.0 / (1.0 + (-x).exp()),
            CurveType::Logit => {
                let p = x.max(0.001).min(0.999);
                (p / (1.0 - p)).ln()
            }
            CurveType::Sinusoidal => (x * std::f32::consts::PI / 2.0).sin(),
        };

        (y + self.y_shift).max(0.0).min(1.0)
    }

    /// 设置斜率
    pub fn set_slope(&mut self, slope: f32) -> &mut Self {
        self.slope = slope;
        self
    }

    /// 设置指数
    pub fn set_exponent(&mut self, exponent: f32) -> &mut Self {
        self.exponent = exponent;
        self
    }

    /// 设置中点
    pub fn set_midpoint(&mut self, midpoint: f32) -> &mut Self {
        self.midpoint = midpoint;
        self
    }

    /// 设置Y轴偏移
    pub fn set_y_shift(&mut self, y_shift: f32) -> &mut Self {
        self.y_shift = y_shift;
        self
    }
}

impl Default for UtilityCurve {
    fn default() -> Self {
        Self::new(CurveType::Linear)
    }
}

/// 效用考虑因素
pub struct UtilityConsideration {
    name: String,
    curve: UtilityCurve,
    weight: f32,
    input_fn: Box<dyn Fn(&UtilityContext) -> f32 + Send + Sync>,
}

impl UtilityConsideration {
    /// 创建新的效用考虑因素
    pub fn new(
        name: &str,
        curve: UtilityCurve,
        weight: f32,
        input_fn: Box<dyn Fn(&UtilityContext) -> f32 + Send + Sync>,
    ) -> Self {
        Self {
            name: name.to_string(),
            curve,
            weight,
            input_fn,
        }
    }

    /// 计算考虑因素的效用值
    pub fn evaluate(&self, context: &UtilityContext) -> UtilityValue {
        let input = (self.input_fn)(context);
        self.curve.evaluate(input) * self.weight
    }

    /// 获取名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// 效用上下文
#[derive(Default)]
pub struct UtilityContext {
    values: HashMap<String, f32>,
}

impl UtilityContext {
    /// 创建新的上下文
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置值
    pub fn set(&mut self, key: &str, value: f32) {
        self.values.insert(key.to_string(), value);
    }

    /// 获取值
    pub fn get(&self, key: &str) -> Option<f32> {
        self.values.get(key).copied()
    }
}

/// 效用动作
pub struct UtilityAction {
    name: String,
    considerations: Vec<UtilityConsideration>,
    cooldown: f32,
    last_execution: f32,
}

impl UtilityAction {
    /// 创建新的效用动作
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            considerations: Vec::new(),
            cooldown: 0.0,
            last_execution: -9999.0,
        }
    }

    /// 添加考虑因素
    pub fn add_consideration(&mut self, consideration: UtilityConsideration) -> &mut Self {
        self.considerations.push(consideration);
        self
    }

    /// 计算总效用值
    pub fn evaluate(&self, context: &UtilityContext) -> UtilityValue {
        let mut sum = 0.0;
        let mut weight_sum = 0.0;

        for consideration in &self.considerations {
            let value = consideration.evaluate(context);
            sum += value;
            weight_sum += consideration.weight;
        }

        if weight_sum > 0.0 {
            sum / weight_sum
        } else {
            0.0
        }
    }

    /// 检查是否可以执行
    pub fn can_execute(&self, current_time: f32) -> bool {
        current_time - self.last_execution >= self.cooldown
    }

    /// 执行动作
    pub fn execute(&mut self, current_time: f32) {
        self.last_execution = current_time;
    }

    /// 设置冷却时间
    pub fn set_cooldown(&mut self, cooldown: f32) {
        self.cooldown = cooldown;
    }

    /// 获取名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// 效用AI系统
pub struct UtilityAI {
    actions: Vec<UtilityAction>,
    current_time: f32,
    threshold: UtilityValue,
}

impl UtilityAI {
    /// 创建新的效用AI系统
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            current_time: 0.0,
            threshold: 0.3,
        }
    }

    /// 添加动作
    pub fn add_action(&mut self, action: UtilityAction) -> &mut Self {
        self.actions.push(action);
        self
    }

    /// 选择最佳动作
    pub fn select_action(&mut self, context: &UtilityContext) -> Option<&UtilityAction> {
        let mut best_action = None;
        let mut best_score = self.threshold;

        for action in &self.actions {
            if !action.can_execute(self.current_time) {
                continue;
            }

            let score = action.evaluate(context);
            if score > best_score {
                best_score = score;
                best_action = Some(action);
            }
        }

        best_action
    }

    /// 执行最佳动作
    pub fn execute_best(&mut self, context: &UtilityContext) -> Option<String> {
        if let Some(action) = self.select_action(context) {
            let action_name = action.name().to_string();
            // 实际执行会通过回调或其他机制
            self.current_time += 0.016; // 假设60fps
            Some(action_name)
        } else {
            None
        }
    }

    /// 更新时间
    pub fn update(&mut self, delta_time: f32) {
        self.current_time += delta_time;
    }

    /// 设置决策阈值
    pub fn set_threshold(&mut self, threshold: UtilityValue) {
        self.threshold = threshold;
    }
}

impl Default for UtilityAI {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 预定义的效用考虑因素构建器
// ============================================================================

/// 效用考虑因素构建器
pub struct ConsiderationBuilder {
    name: String,
    curve: Option<UtilityCurve>,
    weight: f32,
}

impl ConsiderationBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            curve: None,
            weight: 1.0,
        }
    }

    pub fn with_curve(mut self, curve: UtilityCurve) -> Self {
        self.curve = Some(curve);
        self
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    pub fn build(
        self,
        input_fn: Box<dyn Fn(&UtilityContext) -> f32 + Send + Sync>,
    ) -> UtilityConsideration {
        UtilityConsideration::new(
            &self.name,
            self.curve.unwrap_or_default(),
            self.weight,
            input_fn,
        )
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utility_curve_linear() {
        let curve = UtilityCurve::new(CurveType::Linear);
        assert_eq!(curve.evaluate(0.5), 0.5);
    }

    #[test]
    fn test_utility_curve_quadratic() {
        let mut curve = UtilityCurve::new(CurveType::Quadratic);
        curve.set_exponent(2.0);

        let result = curve.evaluate(0.5);
        assert!(result > 0.0 && result < 1.0);
    }

    #[test]
    fn test_utility_curve_logistic() {
        let curve = UtilityCurve::new(CurveType::Logistic);

        // 在中点应该返回0.5
        let result = curve.evaluate(0.5);
        assert!((result - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_utility_context() {
        let mut context = UtilityContext::new();
        context.set("health", 0.7);
        context.set("ammo", 0.3);

        assert_eq!(context.get("health"), Some(0.7));
        assert_eq!(context.get("ammo"), Some(0.3));
    }

    #[test]
    fn test_utility_action() {
        let mut action = UtilityAction::new("attack");

        let curve = UtilityCurve::new(CurveType::Linear);
        let consideration = UtilityConsideration::new(
            "health",
            curve,
            1.0,
            Box::new(|ctx| ctx.get("health").unwrap_or(0.0)),
        );

        action.add_consideration(consideration);

        let mut context = UtilityContext::new();
        context.set("health", 0.8);

        let score = action.evaluate(&context);
        assert!(score > 0.0);
    }

    #[test]
    fn test_utility_ai_selection() {
        let mut ai = UtilityAI::new();

        // 创建攻击动作
        let mut attack = UtilityAction::new("attack");
        let attack_curve = UtilityCurve::new(CurveType::Linear);
        attack.add_consideration(UtilityConsideration::new(
            "enemy_distance",
            attack_curve,
            1.0,
            Box::new(|ctx| 1.0 - ctx.get("enemy_distance").unwrap_or(1.0)),
        ));

        // 创建逃跑动作
        let mut flee = UtilityAction::new("flee");
        let flee_curve = UtilityCurve::new(CurveType::Quadratic);
        flee.add_consideration(UtilityConsideration::new(
            "health",
            flee_curve,
            1.0,
            Box::new(|ctx| 1.0 - ctx.get("health").unwrap_or(0.0)),
        ));

        ai.add_action(attack);
        ai.add_action(flee);

        // 测试场景：近距离，低生命值
        let mut context = UtilityContext::new();
        context.set("enemy_distance", 0.2);
        context.set("health", 0.2);

        // 应该选择逃跑
        let best = ai.select_action(&context);
        assert!(best.is_some());
        assert_eq!(best.unwrap().name(), "flee");
    }
}
