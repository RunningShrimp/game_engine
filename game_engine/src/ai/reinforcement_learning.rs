// 强化学习集成
//
// 提供Q-Learning和Deep Q-Network (DQN)框架

use std::collections::HashMap;

/// Q-Learning智能体
pub struct QLearningAgent {
    /// Q表: 状态 -> 动作 -> Q值
    q_table: HashMap<usize, HashMap<usize, f32>>,
    /// 学习率
    learning_rate: f32,
    /// 折扣因子
    discount_factor: f32,
    /// 探索率 (epsilon)
    exploration_rate: f32,
    /// 探索率衰减
    exploration_decay: f32,
    /// 最小探索率
    min_exploration: f32,
}

impl QLearningAgent {
    /// 创建新的Q-Learning智能体
    pub fn new(learning_rate: f32, discount_factor: f32, exploration_rate: f32) -> Self {
        Self {
            q_table: HashMap::new(),
            learning_rate,
            discount_factor,
            exploration_rate,
            exploration_decay: 0.995,
            min_exploration: 0.01,
        }
    }

    /// 选择动作 (epsilon-greedy策略)
    pub fn select_action(&mut self, state: usize, num_actions: usize) -> usize {
        // epsilon概率随机探索
        if rand_random() < self.exploration_rate {
            return (rand_random() * num_actions as f32) as usize;
        }

        // 否则选择最佳动作
        self.get_best_action(state, num_actions)
    }

    /// 获取最佳动作
    fn get_best_action(&self, state: usize, num_actions: usize) -> usize {
        if let Some(actions) = self.q_table.get(&state) {
            let mut best_action = 0;
            let mut best_value = f32::NEG_INFINITY;

            for action in 0..num_actions {
                let q_value = *actions.get(&action).unwrap_or(&0.0);
                if q_value > best_value {
                    best_value = q_value;
                    best_action = action;
                }
            }

            best_action
        } else {
            // 状态未见过，随机选择
            (rand_random() * num_actions as f32) as usize
        }
    }

    /// 更新Q值
    pub fn learn(
        &mut self,
        state: usize,
        action: usize,
        reward: f32,
        next_state: usize,
        num_actions: usize,
    ) {
        // 获取当前Q值
        let current_q = *self
            .q_table
            .entry(state)
            .or_insert_with(HashMap::new)
            .entry(action)
            .or_insert(0.0);

        // 计算最大未来Q值
        let max_next_q = self.get_max_q_value(next_state, num_actions);

        // Q-Learning更新公式
        // Q(s,a) = Q(s,a) + α * (r + γ * max(Q(s',a')) - Q(s,a))
        let new_q = current_q
            + self.learning_rate * (reward + self.discount_factor * max_next_q - current_q);

        // 更新Q表
        self.q_table.get_mut(&state).unwrap().insert(action, new_q);

        // 衰减探索率
        self.exploration_rate =
            (self.exploration_rate * self.exploration_decay).max(self.min_exploration);
    }

    /// 获取状态的最大Q值
    fn get_max_q_value(&self, state: usize, num_actions: usize) -> f32 {
        if let Some(actions) = self.q_table.get(&state) {
            let mut max_q = 0.0;
            for action in 0..num_actions {
                let q_value = *actions.get(&action).unwrap_or(&0.0);
                if q_value > max_q {
                    max_q = q_value;
                }
            }
            max_q
        } else {
            0.0
        }
    }

    /// 获取Q值
    pub fn get_q_value(&self, state: usize, action: usize) -> f32 {
        self.q_table
            .get(&state)
            .and_then(|actions| actions.get(&action))
            .copied()
            .unwrap_or(0.0)
    }

    /// 设置学习率
    pub fn set_learning_rate(&mut self, rate: f32) {
        self.learning_rate = rate;
    }

    /// 设置折扣因子
    pub fn set_discount_factor(&mut self, factor: f32) {
        self.discount_factor = factor;
    }

    /// 获取探索率
    pub fn exploration_rate(&self) -> f32 {
        self.exploration_rate
    }
}

impl Default for QLearningAgent {
    fn default() -> Self {
        Self::new(0.1, 0.99, 0.3)
    }
}

/// 经验回放缓冲区 (用于DQN)
pub struct ExperienceBuffer {
    capacity: usize,
    buffer: Vec<Experience>,
    position: usize,
}

/// 经验样本
#[derive(Clone, Debug)]
pub struct Experience {
    pub state: usize,
    pub action: usize,
    pub reward: f32,
    pub next_state: usize,
    pub done: bool,
}

impl ExperienceBuffer {
    /// 创建新的经验缓冲区
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            buffer: Vec::with_capacity(capacity),
            position: 0,
        }
    }

    /// 添加经验
    pub fn add(&mut self, experience: Experience) {
        if self.buffer.len() < self.capacity {
            self.buffer.push(experience);
        } else {
            self.buffer[self.position] = experience;
            self.position = (self.position + 1) % self.capacity;
        }
    }

    /// 随机采样一批经验
    pub fn sample(&self, batch_size: usize) -> Vec<&Experience> {
        if self.buffer.is_empty() {
            return Vec::new();
        }

        let mut batch = Vec::with_capacity(batch_size);
        for _ in 0..batch_size {
            let idx = (rand_random() * self.buffer.len() as f32) as usize;
            batch.push(&self.buffer[idx]);
        }
        batch
    }

    /// 获取缓冲区大小
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

/// 简化的神经网络层 (用于演示DQN概念)
#[derive(Clone)]
pub struct DenseLayer {
    weights: Vec<Vec<f32>>,
    biases: Vec<f32>,
}

impl DenseLayer {
    /// 创建新的全连接层
    pub fn new(input_size: usize, output_size: usize) -> Self {
        let mut weights = Vec::with_capacity(output_size);
        for _ in 0..output_size {
            let mut neuron_weights = Vec::with_capacity(input_size);
            for _ in 0..input_size {
                // Xavier初始化
                let limit = (6.0 / (input_size + output_size) as f32).sqrt();
                let weight = (rand_random() * 2.0 - 1.0) * limit;
                neuron_weights.push(weight);
            }
            weights.push(neuron_weights);
        }

        let biases = vec![0.0; output_size];

        Self { weights, biases }
    }

    /// 前向传播
    pub fn forward(&self, input: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(self.weights.len());

        for (neuron_weights, &bias) in self.weights.iter().zip(&self.biases) {
            let mut sum = bias;
            for (&w, &i) in neuron_weights.iter().zip(input) {
                sum += w * i;
            }
            // ReLU激活函数
            output.push(sum.max(0.0));
        }

        output
    }

    /// 获取权重数量
    pub fn parameter_count(&self) -> usize {
        self.weights.iter().map(|w| w.len()).sum::<usize>() + self.biases.len()
    }
}

/// 简化的深度Q网络
pub struct DeepQNetwork {
    layers: Vec<DenseLayer>,
    learning_rate: f32,
}

impl DeepQNetwork {
    /// 创建新的DQN
    pub fn new(input_size: usize, hidden_sizes: &[usize], output_size: usize) -> Self {
        let mut layers = Vec::new();

        let mut prev_size = input_size;
        for &hidden_size in hidden_sizes {
            layers.push(DenseLayer::new(prev_size, hidden_size));
            prev_size = hidden_size;
        }
        layers.push(DenseLayer::new(prev_size, output_size));

        Self {
            layers,
            learning_rate: 0.001,
        }
    }

    /// 前向传播
    pub fn forward(&self, input: &[f32]) -> Vec<f32> {
        let mut current = input.to_vec();

        for layer in &self.layers {
            current = layer.forward(&current);
        }

        current
    }

    /// 选择动作
    pub fn select_action(&self, state: &[f32], exploration_rate: f32) -> usize {
        if rand_random() < exploration_rate {
            return (rand_random() * state.len() as f32) as usize;
        }

        let q_values = self.forward(state);
        let mut best_action = 0;
        let mut best_value = f32::NEG_INFINITY;

        for (i, &value) in q_values.iter().enumerate() {
            if value > best_value {
                best_value = value;
                best_action = i;
            }
        }

        best_action
    }

    /// 简化的训练步骤 (实际应该使用反向传播)
    pub fn train_step(&mut self, _batch: &[&Experience]) {
        // 实际DQN实现需要完整的反向传播算法
        // 这里只是框架代码
    }

    /// 获取总参数数量
    pub fn total_parameters(&self) -> usize {
        self.layers.iter().map(|l| l.parameter_count()).sum()
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 简单随机数生成器
fn rand_random() -> f32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static SEED: AtomicU32 = AtomicU32::new(12345);

    let seed = SEED.fetch_add(1, Ordering::Relaxed);
    let a = 1664525u32;
    let c = 1013904223u32;
    let m = 2u32.pow(32);

    ((seed.wrapping_mul(a).wrapping_add(c)) % m) as f32 / m as f32
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_q_learning_creation() {
        let agent = QLearningAgent::new(0.1, 0.99, 0.3);
        assert_eq!(agent.learning_rate, 0.1);
        assert_eq!(agent.discount_factor, 0.99);
    }

    #[test]
    fn test_q_learning_select_action() {
        let mut agent = QLearningAgent::new(0.1, 0.99, 0.3);

        // 测试动作选择
        let action = agent.select_action(0, 4);
        assert!(action < 4);
    }

    #[test]
    fn test_q_learning_update() {
        let mut agent = QLearningAgent::new(0.1, 0.99, 0.1); // 低探索率

        // 学习一个简单的转移
        agent.learn(0, 1, 1.0, 1, 2);

        let q_value = agent.get_q_value(0, 1);
        assert!(q_value > 0.0);
    }

    #[test]
    fn test_experience_buffer() {
        let mut buffer = ExperienceBuffer::new(100);

        // 添加经验
        buffer.add(Experience {
            state: 0,
            action: 1,
            reward: 1.0,
            next_state: 1,
            done: false,
        });

        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_dense_layer() {
        let layer = DenseLayer::new(3, 2);

        let input = vec![1.0, 2.0, 3.0];
        let output = layer.forward(&input);

        assert_eq!(output.len(), 2);
    }

    #[test]
    fn test_deep_q_network() {
        let dqn = DeepQNetwork::new(4, &[8, 8], 2);

        let input = vec![0.1, 0.2, 0.3, 0.4];
        let output = dqn.forward(&input);

        assert_eq!(output.len(), 2);

        // 应该有参数
        assert!(dqn.total_parameters() > 0);
    }
}
