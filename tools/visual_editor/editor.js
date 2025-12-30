// 游戏引擎AI可视化编辑器 - JavaScript实现

// 全局状态
let behaviorTree = {
    version: "1.0",
    tree: "editor_tree",
    nodes: []
};

let influenceGrid = [];
let nodeIdCounter = 0;

// 初始化
document.addEventListener('DOMContentLoaded', function() {
    initBehaviorTreeCanvas();
    drawUtilityCurve();
});

// ==================== 标签页切换 ====================

function switchTab(tabId) {
    // 隐藏所有标签页内容
    document.querySelectorAll('.tab-content').forEach(content => {
        content.classList.remove('active');
    });

    // 移除所有标签的active类
    document.querySelectorAll('.tab').forEach(tab => {
        tab.classList.remove('active');
    });

    // 显示选中的标签页
    document.getElementById(tabId).classList.add('active');

    // 激活对应的标签
    event.target.classList.add('active');
}

// ==================== 行为树编辑器 ====================

function addNode() {
    const nodeType = document.getElementById('nodeType').value;
    const nodeId = `node_${nodeIdCounter++}`;

    const node = {
        id: nodeId,
        type: nodeType,
        name: `${nodeType.charAt(0).toUpperCase() + nodeType.slice(1)} Node`,
        children: [],
        config: {}
    };

    behaviorTree.nodes.push(node);
    updateBehaviorNodeList();
    drawBehaviorTree();
    updateBehaviorTreeJSON();
    showNotification(`✅ 已添加 ${nodeType} 节点`);
}

function updateBehaviorNodeList() {
    const nodeList = document.getElementById('behaviorNodeList');
    nodeList.innerHTML = '';

    const rootNode = behaviorTree.nodes.find(n => n.type === 'sequence' || n.type === 'selector');

    if (rootNode) {
        nodeList.innerHTML += `<li class="node-item" data-id="${rootNode.id}">🌳 ${rootNode.name}</li>`;
    }

    behaviorTree.nodes.forEach(node => {
        if (node !== rootNode) {
            const emoji = getNodeEmoji(node.type);
            nodeList.innerHTML += `<li class="node-item" data-id="${node.id}">${emoji} ${node.name}</li>`;
        }
    });
}

function getNodeEmoji(type) {
    const emojis = {
        'sequence': '➡️',
        'selector': '🔀',
        'parallel': '⚡',
        'inverter': '🔄',
        'condition': '❓',
        'action': '⚡',
        'root': '🏠'
    };
    return emojis[type] || '📦';
}

function updateBehaviorTreeJSON() {
    const json = JSON.stringify(behaviorTree, null, 2);
    document.getElementById('behaviorTreeJson').value = json;
}

function copyBehaviorTreeJSON() {
    const json = document.getElementById('behaviorTreeJson').value;
    navigator.clipboard.writeText(json).then(() => {
        showNotification('📋 JSON已复制到剪贴板');
    });
}

function exportBehaviorTree() {
    const json = JSON.stringify(behaviorTree, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `behavior_tree_${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
    showNotification('💾 行为树已导出');
}

// ==================== 行为树可视化 ====================

function initBehaviorTreeCanvas() {
    const canvas = document.getElementById('behaviorTreeCanvas');
    const container = canvas.parentElement;

    canvas.width = container.clientWidth;
    canvas.height = container.clientHeight;

    drawBehaviorTree();
}

function drawBehaviorTree() {
    const canvas = document.getElementById('behaviorTreeCanvas');
    const ctx = canvas.getContext('2d');

    // 清空画布
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    if (behaviorTree.nodes.length === 0) {
        ctx.fillStyle = 'rgba(255, 255, 255, 0.5)';
        ctx.font = '16px Arial';
        ctx.textAlign = 'center';
        ctx.fillText('添加节点以开始构建行为树', canvas.width / 2, canvas.height / 2);
        return;
    }

    // 简化的树形布局
    const rootNode = behaviorTree.nodes[0];
    const nodePositions = new Map();

    // 计算节点位置
    const levels = calculateTreeLevels(behaviorTree.nodes);
    const levelHeight = canvas.height / (levels.length + 1);

    levels.forEach((nodes, level) => {
        const levelWidth = canvas.width / (nodes.length + 1);
        nodes.forEach((node, index) => {
            const x = levelWidth * (index + 1);
            const y = levelHeight * (level + 1);
            nodePositions.set(node.id, { x, y });
        });
    });

    // 绘制连接线
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.3)';
    ctx.lineWidth = 2;

    behaviorTree.nodes.forEach(node => {
        if (node.children && node.children.length > 0) {
            const parentPos = nodePositions.get(node.id);
            if (parentPos) {
                node.children.forEach(childId => {
                    const childPos = nodePositions.get(childId);
                    if (childPos) {
                        ctx.beginPath();
                        ctx.moveTo(parentPos.x, parentPos.y);
                        ctx.lineTo(childPos.x, childPos.y);
                        ctx.stroke();
                    }
                });
            }
        }
    });

    // 绘制节点
    behaviorTree.nodes.forEach(node => {
        const pos = nodePositions.get(node.id);
        if (pos) {
            drawNode(ctx, pos.x, pos.y, node);
        }
    });
}

function calculateTreeLevels(nodes) {
    // 简化：将所有节点放在不同层级
    const levels = [];
    const levelSize = 3;

    for (let i = 0; i < nodes.length; i += levelSize) {
        levels.push(nodes.slice(i, i + levelSize));
    }

    return levels.length > 0 ? levels : [[nodes[0]]];
}

function drawNode(ctx, x, y, node) {
    const width = 100;
    const height = 40;

    // 节点背景
    const gradient = ctx.createLinearGradient(x - width/2, y - height/2, x + width/2, y + height/2);
    gradient.addColorStop(0, '#4CAF50');
    gradient.addColorStop(1, '#2196F3');

    ctx.fillStyle = gradient;
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.5)';
    ctx.lineWidth = 2;

    // 圆角矩形
    const radius = 5;
    ctx.beginPath();
    ctx.roundRect(x - width/2, y - height/2, width, height, radius);
    ctx.fill();
    ctx.stroke();

    // 节点文本
    ctx.fillStyle = '#fff';
    ctx.font = '12px Arial';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(node.name, x, y);
}

// ==================== 覆盖图可视化 ====================

function generateInfluenceMap() {
    const width = parseInt(document.getElementById('gridWidth').value);
    const height = parseInt(document.getElementById('gridHeight').value);
    const strength = parseFloat(document.getElementById('sourceStrength').value);

    influenceGrid = [];

    // 初始化网格
    for (let y = 0; y < height; y++) {
        influenceGrid[y] = [];
        for (let x = 0; x < width; x++) {
            influenceGrid[y][x] = 0;
        }
    }

    // 添加随机影响源
    const numSources = Math.floor(Math.random() * 5) + 2;
    for (let i = 0; i < numSources; i++) {
        const sx = Math.floor(Math.random() * width);
        const sy = Math.floor(Math.random() * height);
        const svalue = (Math.random() > 0.5 ? 1 : -1) * strength;

        influenceGrid[sy][sx] += svalue;

        // 传播影响
        for (let y = 0; y < height; y++) {
            for (let x = 0; x < width; x++) {
                if (x === sx && y === sy) continue;

                const dist = Math.sqrt((x - sx) ** 2 + (y - sy) ** 2);
                const falloff = Math.exp(-dist / 5);
                influenceGrid[y][x] += svalue * falloff * 0.3;
            }
        }
    }

    renderInfluenceMap();
    calculateStatistics();
    showNotification('🗺️ 覆盖图已生成');
}

function renderInfluenceMap() {
    const container = document.getElementById('gridVisualizer');
    container.innerHTML = '';

    const height = influenceGrid.length;
    const width = height > 0 ? influenceGrid[0].length : 0;

    container.style.gridTemplateColumns = `repeat(${width}, 1fr)`;

    for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
            const value = influenceGrid[y][x];
            const cell = document.createElement('div');
            cell.className = 'grid-cell';
            cell.textContent = value.toFixed(1);

            // 颜色映射
            const normalized = Math.max(-1, Math.min(1, value / 100));
            if (normalized > 0) {
                cell.style.background = `rgba(76, 175, 80, ${normalized})`;
            } else {
                cell.style.background = `rgba(244, 67, 54, ${-normalized})`;
            }

            container.appendChild(cell);
        }
    }
}

function calculateStatistics() {
    if (influenceGrid.length === 0) return;

    const values = influenceGrid.flat();
    const min = Math.min(...values);
    const max = Math.max(...values);
    const mean = values.reduce((a, b) => a + b, 0) / values.length;
    const variance = values.reduce((a, b) => a + (b - mean) ** 2, 0) / values.length;
    const stdDev = Math.sqrt(variance);

    document.getElementById('statMin').textContent = min.toFixed(2);
    document.getElementById('statMax').textContent = max.toFixed(2);
    document.getElementById('statMean').textContent = mean.toFixed(2);
    document.getElementById('statStdDev').textContent = stdDev.toFixed(2);
}

// ==================== GOAP规划器 ====================

function runGOAPPlanning() {
    const actionCount = parseInt(document.getElementById('actionCount').value);
    const worldStateJSON = document.getElementById('worldState').value;

    try {
        const worldState = JSON.parse(worldStateJSON);

        // 模拟规划过程
        const actions = [
            '移动到目标位置',
            '准备武器',
            '攻击目标',
            '寻找掩体',
            '恢复生命值'
        ];

        const plan = [];
        const numActions = Math.floor(Math.random() * actionCount) + 1;

        for (let i = 0; i < numActions; i++) {
            const action = actions[Math.floor(Math.random() * actions.length)];
            plan.push(action);
        }

        // 显示规划结果
        const planList = document.getElementById('goapPlan');
        planList.innerHTML = '';

        if (plan.length > 0) {
            planList.innerHTML = '<li class="node-item">🎯 找到最佳计划:</li>';
            plan.forEach((action, index) => {
                planList.innerHTML += `<li class="node-item">${index + 1}. ${action}</li>`;
            });
        } else {
            planList.innerHTML = '<li class="node-item">❌ 未找到可行计划</li>';
        }

        showNotification('🔍 GOAP规划完成');
    } catch (e) {
        showNotification('❌ 世界状态JSON格式错误');
    }
}

// ==================== 效用曲线 ====================

function drawUtilityCurve() {
    const canvas = document.getElementById('utilityCurveCanvas');
    const ctx = canvas.getContext('2d');
    const container = canvas.parentElement;

    canvas.width = container.clientWidth;
    canvas.height = container.clientHeight;

    const curveType = document.getElementById('curveType').value;
    const slope = parseFloat(document.getElementById('curveSlope').value);
    const exponent = parseFloat(document.getElementById('curveExponent').value);

    // 清空画布
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // 绘制坐标轴
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.3)';
    ctx.lineWidth = 1;

    // X轴
    ctx.beginPath();
    ctx.moveTo(50, canvas.height - 30);
    ctx.lineTo(canvas.width - 30, canvas.height - 30);
    ctx.stroke();

    // Y轴
    ctx.beginPath();
    ctx.moveTo(50, canvas.height - 30);
    ctx.lineTo(50, 30);
    ctx.stroke();

    // 标签
    ctx.fillStyle = 'rgba(255, 255, 255, 0.7)';
    ctx.font = '12px Arial';
    ctx.fillText('Input', canvas.width / 2, canvas.height - 10);

    ctx.save();
    ctx.translate(20, canvas.height / 2);
    ctx.rotate(-Math.PI / 2);
    ctx.fillText('Utility', 0, 0);
    ctx.restore();

    // 绘制曲线
    ctx.strokeStyle = '#4CAF50';
    ctx.lineWidth = 3;
    ctx.beginPath();

    const padding = 50;
    const graphWidth = canvas.width - 2 * padding;
    const graphHeight = canvas.height - 2 * padding;

    for (let px = 0; px <= graphWidth; px++) {
        const input = px / graphWidth;
        let output = 0;

        const x = (input - 0.5) * slope + 0.5;

        switch (curveType) {
            case 'linear':
                output = x;
                break;
            case 'quadratic':
                output = x >= 0 ? Math.pow(x, exponent) : -Math.pow(-x, exponent);
                break;
            case 'logistic':
                output = 1 / (1 + Math.exp(-10 * (x - 0.5)));
                break;
            case 'sinusoidal':
                output = Math.sin(x * Math.PI / 2);
                break;
        }

        const py = graphHeight * (1 - output);

        const canvasX = padding + px;
        const canvasY = padding + py;

        if (px === 0) {
            ctx.moveTo(canvasX, canvasY);
        } else {
            ctx.lineTo(canvasX, canvasY);
        }
    }

    ctx.stroke();
}

// ==================== 工具函数 ====================

function showNotification(message) {
    const notification = document.getElementById('notification');
    notification.textContent = message;
    notification.classList.add('show');

    setTimeout(() => {
        notification.classList.remove('show');
    }, 3000);
}

function saveProject() {
    const project = {
        behaviorTree,
        influenceGrid,
        timestamp: Date.now()
    };

    localStorage.setItem('gameEngineProject', JSON.stringify(project));
    showNotification('💾 项目已保存');
}

function loadProject() {
    const saved = localStorage.getItem('gameEngineProject');
    if (saved) {
        try {
            const project = JSON.parse(saved);
            behaviorTree = project.behaviorTree;
            influenceGrid = project.influenceGrid;

            updateBehaviorNodeList();
            drawBehaviorTree();
            updateBehaviorTreeJSON();
            renderInfluenceMap();
            calculateStatistics();

            showNotification('📂 项目已加载');
        } catch (e) {
            showNotification('❌ 加载项目失败');
        }
    } else {
        showNotification('⚠️ 没有找到已保存的项目');
    }
}

// 窗口大小改变时重绘
window.addEventListener('resize', () => {
    initBehaviorTreeCanvas();
    drawUtilityCurve();
});
