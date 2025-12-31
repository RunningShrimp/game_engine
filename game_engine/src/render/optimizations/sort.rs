// 渲染排序优化
//
// 实现按材质、深度和透明度的渲染排序

use std::collections::HashMap;

// ============================================================================
// 排序键
// ============================================================================

/// 排序键
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SortKey {
    /// 材质ID（高优先级）
    material_id: u32,
    /// 深度（用于透明物体）
    depth: f32,
    /// 透明度标志
    transparent: bool,
}

impl SortKey {
    /// 创建不透明对象的排序键
    pub fn opaque(material_id: u32) -> Self {
        Self {
            material_id,
            depth: 0.0,
            transparent: false,
        }
    }

    /// 创建透明对象的排序键
    pub fn transparent(material_id: u32, depth: f32) -> Self {
        Self {
            material_id,
            depth,
            transparent: true,
        }
    }
}

// ============================================================================
// 渲染项
// ============================================================================

/// 渲染项
#[derive(Debug, Clone)]
pub struct RenderItem {
    /// 渲染项ID
    pub id: u32,
    /// 材质ID
    pub material_id: u32,
    /// 网格ID
    pub mesh_id: u32,
    /// 到相机的距离
    pub distance: f32,
    /// 是否透明
    pub transparent: bool,
    /// 变换矩阵
    pub transform: [[f32; 4]; 4],
    /// 用户数据
    pub user_data: Vec<u8>,
}

impl RenderItem {
    /// 创建新的渲染项
    pub fn new(
        id: u32,
        material_id: u32,
        mesh_id: u32,
        distance: f32,
        transparent: bool,
    ) -> Self {
        Self {
            id,
            material_id,
            mesh_id,
            distance,
            transparent,
            transform: [[1.0, 0.0, 0.0, 0.0]; 4],
            user_data: Vec::new(),
        }
    }

    /// 获取排序键
    pub fn sort_key(&self) -> SortKey {
        if self.transparent {
            SortKey::transparent(self.material_id, self.distance)
        } else {
            SortKey::opaque(self.material_id)
        }
    }
}

// ============================================================================
// 排序策略
// ============================================================================

/// 排序策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortStrategy {
    /// 按材质排序（最小化状态切换）
    ByMaterial,
    /// 按深度排序（从前到后）
    ByDepthFrontToBack,
    /// 按深度排序（从后到前，用于透明物体）
    ByDepthBackToFront,
    /// 混合排序（材质 + 深度）
    Hybrid,
}

// ============================================================================
// 材质排序器
// ============================================================================

/// 材质排序器
pub struct MaterialSorter {
    /// 渲染项列表
    items: Vec<RenderItem>,
}

impl MaterialSorter {
    /// 创建新的材质排序器
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// 添加渲染项
    pub fn add_item(&mut self, item: RenderItem) {
        self.items.push(item);
    }

    /// 排序并返回渲染项
    pub fn sort(&mut self) -> Vec<RenderItem> {
        // 按材质ID排序
        self.items.sort_by(|a, b| {
            a.material_id
                .cmp(&b.material_id)
        });

        // 分离不透明和透明对象
        let (opaque, transparent): (Vec<_>, Vec<_>) = self
            .items
            .iter()
            .partition(|item| !item.transparent);

        let mut result = opaque;
        result.extend(transparent);

        result.into_iter().cloned().collect()
    }

    /// 清空渲染项
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// 获取渲染项数量
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for MaterialSorter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 深度排序器
// ============================================================================

/// 深度排序器
pub struct DepthSorter {
    /// 渲染项列表
    items: Vec<RenderItem>,
    /// 排序顺序
    front_to_back: bool,
}

impl DepthSorter {
    /// 创建新的深度排序器
    pub fn new(front_to_back: bool) -> Self {
        Self {
            items: Vec::new(),
            front_to_back,
        }
    }

    /// 添加渲染项
    pub fn add_item(&mut self, item: RenderItem) {
        self.items.push(item);
    }

    /// 排序并返回渲染项
    pub fn sort(&mut self) -> Vec<RenderItem> {
        if self.front_to_back {
            // 从前到后排序（近的先渲染）
            self.items
                .sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        } else {
            // 从后到前排序（远的先渲染）
            self.items
                .sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
        }

        self.items.clone()
    }

    /// 清空渲染项
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// 获取渲染项数量
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// ============================================================================
// 混合排序器
// ============================================================================

/// 混合排序器（材质 + 深度）
pub struct HybridSorter {
    /// 渲染项列表
    items: Vec<RenderItem>,
}

impl HybridSorter {
    /// 创建新的混合排序器
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// 添加渲染项
    pub fn add_item(&mut self, item: RenderItem) {
        self.items.push(item);
    }

    /// 排序并返回渲染项
    pub fn sort(&mut self) -> Vec<RenderItem> {
        // 分离不透明和透明对象
        let (opaque, transparent): (Vec<_>, Vec<_>) = self
            .items
            .drain(..)
            .partition(|item| !item.transparent);

        // 不透明对象：按材质分组，组内按深度从前到后排序
        let mut opaque_groups: HashMap<u32, Vec<RenderItem>> = HashMap::new();
        for item in opaque {
            opaque_groups
                .entry(item.material_id)
                .or_insert_with(Vec::new)
                .push(item);
        }

        let mut sorted_opaque: Vec<RenderItem> = opaque_groups
            .into_iter()
            .flat_map(|(_, mut items)| {
                items.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
                items
            })
            .collect();

        // 透明对象：按材质分组，组内按深度从后到前排序
        let mut transparent_groups: HashMap<u32, Vec<RenderItem>> = HashMap::new();
        for item in transparent {
            transparent_groups
                .entry(item.material_id)
                .or_insert_with(Vec::new)
                .push(item);
        }

        let mut sorted_transparent: Vec<RenderItem> = transparent_groups
            .into_iter()
            .flat_map(|(_, mut items)| {
                items.sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
                items
            })
            .collect();

        // 合并：不透明在前，透明在后
        sorted_opaque.append(&mut sorted_transparent);

        // 更新内部列表
        self.items = sorted_opaque.clone();
        sorted_opaque
    }

    /// 清空渲染项
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// 获取渲染项数量
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for HybridSorter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 渲染队列
// ============================================================================

/// 渲染队列
pub struct RenderQueue {
    /// 不透明对象
    opaque_items: Vec<RenderItem>,
    /// 透明对象
    transparent_items: Vec<RenderItem>,
    /// 排序策略
    strategy: SortStrategy,
}

impl RenderQueue {
    /// 创建新的渲染队列
    pub fn new(strategy: SortStrategy) -> Self {
        Self {
            opaque_items: Vec::new(),
            transparent_items: Vec::new(),
            strategy,
        }
    }

    /// 添加渲染项
    pub fn add_item(&mut self, item: RenderItem) {
        if item.transparent {
            self.transparent_items.push(item);
        } else {
            self.opaque_items.push(item);
        }
    }

    /// 排序并获取所有渲染项
    pub fn sort(&mut self) -> Vec<RenderItem> {
        match self.strategy {
            SortStrategy::ByMaterial => {
                self.opaque_items.sort_by_key(|item| item.material_id);
                self.transparent_items
                    .sort_by_key(|item| item.material_id);
            }
            SortStrategy::ByDepthFrontToBack => {
                self.opaque_items
                    .sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
                self.transparent_items
                    .sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
            }
            SortStrategy::ByDepthBackToFront => {
                self.opaque_items
                    .sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
                self.transparent_items
                    .sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
            }
            SortStrategy::Hybrid => {
                // 不透明：材质优先，然后深度从前到后
                self.opaque_items
                    .sort_by(|a, b| {
                        a.material_id
                            .cmp(&b.material_id)
                            .then_with(|| a.distance.partial_cmp(&b.distance).unwrap())
                    });

                // 透明：材质优先，然后深度从后到前
                self.transparent_items
                    .sort_by(|a, b| {
                        a.material_id
                            .cmp(&b.material_id)
                            .then_with(|| b.distance.partial_cmp(&a.distance).unwrap())
                    });
            }
        }

        // 合并：不透明在前，透明在后
        let mut result = std::mem::take(&mut self.opaque_items);
        result.append(&mut self.transparent_items);

        result
    }

    /// 清空队列
    pub fn clear(&mut self) {
        self.opaque_items.clear();
        self.transparent_items.clear();
    }

    /// 获取不透明对象数量
    pub fn opaque_count(&self) -> usize {
        self.opaque_items.len()
    }

    /// 获取透明对象数量
    pub fn transparent_count(&self) -> usize {
        self.transparent_items.len()
    }

    /// 获取总数量
    pub fn len(&self) -> usize {
        self.opaque_items.len() + self.transparent_items.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.opaque_items.is_empty() && self.transparent_items.is_empty()
    }
}

// ============================================================================
// 排序统计
// ============================================================================

/// 排序统计
#[derive(Debug, Clone, Copy)]
pub struct SortingStatistics {
    /// 排序前项数
    pub items_before: usize,
    /// 排序后项数
    pub items_after: usize,
    /// 排序耗时（微秒）
    pub sort_time_us: u64,
    /// 材质切换次数（估算）
    pub material_switches: u32,
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_sorter() {
        let mut sorter = MaterialSorter::new();

        sorter.add_item(RenderItem::new(1, 100, 1, 10.0, false));
        sorter.add_item(RenderItem::new(2, 50, 2, 20.0, false));
        sorter.add_item(RenderItem::new(3, 100, 3, 30.0, false));

        let sorted = sorter.sort();
        assert_eq!(sorted[0].material_id, 50);
        assert_eq!(sorted[1].material_id, 100);
        assert_eq!(sorted[2].material_id, 100);
    }

    #[test]
    fn test_depth_sorter() {
        let mut sorter = DepthSorter::new(true); // 从前到后

        sorter.add_item(RenderItem::new(1, 100, 1, 30.0, false));
        sorter.add_item(RenderItem::new(2, 100, 2, 10.0, false));
        sorter.add_item(RenderItem::new(3, 100, 3, 20.0, false));

        let sorted = sorter.sort();
        assert_eq!(sorted[0].distance, 10.0);
        assert_eq!(sorted[1].distance, 20.0);
        assert_eq!(sorted[2].distance, 30.0);
    }

    #[test]
    fn test_hybrid_sorter() {
        let mut sorter = HybridSorter::new();

        // 添加不透明对象
        sorter.add_item(RenderItem::new(1, 100, 1, 10.0, false));
        sorter.add_item(RenderItem::new(2, 50, 2, 20.0, false));

        // 添加透明对象
        sorter.add_item(RenderItem::new(3, 100, 3, 30.0, true));
        sorter.add_item(RenderItem::new(4, 100, 4, 10.0, true));

        let sorted = sorter.sort();

        // 不透明对象应该在前面
        assert!(!sorted[0].transparent);
        assert!(!sorted[1].transparent);

        // 透明对象应该在后面
        assert!(sorted[2].transparent);
        assert!(sorted[3].transparent);

        // 透明对象应该从后到前排序
        assert!(sorted[2].distance > sorted[3].distance);
    }

    #[test]
    fn test_render_queue() {
        let mut queue = RenderQueue::new(SortStrategy::Hybrid);

        queue.add_item(RenderItem::new(1, 100, 1, 10.0, false));
        queue.add_item(RenderItem::new(2, 50, 2, 20.0, false));
        queue.add_item(RenderItem::new(3, 100, 3, 30.0, true));

        assert_eq!(queue.opaque_count(), 2);
        assert_eq!(queue.transparent_count(), 1);

        let sorted = queue.sort();
        assert_eq!(sorted.len(), 3);
    }

    #[test]
    fn test_sort_key() {
        let key1 = SortKey::opaque(100);
        let key2 = SortKey::opaque(50);
        let key3 = SortKey::transparent(100, 10.0);

        assert!(key2 < key1); // 材质ID小的排在前面
        assert!(key1 < key3); // 不透明排在透明前面
    }
}
