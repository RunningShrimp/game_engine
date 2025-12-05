// 性能监控仪表板JavaScript模块

// 全局变量
let chart = null;
let connectionStatus = 'disconnected';
let autoRefresh = true;
let refreshInterval = null;

// DOM元素引用
const elements = {
    connectionStatus: document.getElementById('connection-status'),
    connectionText: document.getElementById('connection-text'),
    fpsValue: document.getElementById('fps-value'),
    frameTimeValue: document.getElementById('frame-time-value'),
    drawCallsValue: document.getElementById('draw-calls-value'),
    cpuUsageValue: document.getElementById('cpu-usage-value'),
    memoryUsageValue: document.getElementById('memory-usage-value'),
    gpuUsageValue: document.getElementById('gpu-usage-value'),
    physicsTimeValue: document.getElementById('physics-time-value'),
    audioLatencyValue: document.getElementById('audio-latency-value'),
    metricSelect: document.getElementById('metric-select'),
    timeRangeSelect: document.getElementById('time-range'),
    refreshButton: document.getElementById('refresh-chart'),
    autoRefreshCheckbox: document.getElementById('auto-refresh'),
    alertsContainer: document.getElementById('alerts-container'),
    clearAlertsButton: document.getElementById('clear-alerts')
};

// 初始化函数
function init() {
    setupEventListeners();
    updateConnectionStatus('disconnected');
    startAutoRefresh();
    
    // 初始化图表
    const ctx = document.getElementById('performance-chart').getContext('2d');
    chart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: [],
            datasets: [{
                label: '性能指标',
                data: [],
                borderColor: 'rgb(75, 192, 192)',
                backgroundColor: 'rgba(75, 192, 192, 0.2)',
                tension: 0.1
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: {
                    type: 'time',
                    time: {
                        unit: 'second',
                        displayFormats: ['h:mm:ss']
                    }
                },
                y: {
                    beginAtZero: true,
                    title: {
                        display: true,
                        text: '数值'
                    }
                }
            },
            plugins: {
                legend: {
                    display: true,
                    position: 'top'
                },
                tooltip: {
                    mode: 'index',
                    intersect: false
                }
            }
        }
    });
}

// 设置事件监听器
function setupEventListeners() {
    elements.metricSelect.addEventListener('change', onMetricChange);
    elements.timeRangeSelect.addEventListener('change', onTimeRangeChange);
    elements.refreshButton.addEventListener('click', onRefreshChart);
    elements.autoRefreshCheckbox.addEventListener('change', onAutoRefreshToggle);
    elements.clearAlertsButton.addEventListener('click', onClearAlerts);
    
    // 页面卸载时清理
    window.addEventListener('beforeunload', () => {
        if (refreshInterval) {
            clearInterval(refreshInterval);
        }
    });
}

// 更新连接状态
function updateConnectionStatus(status) {
    connectionStatus = status;
    elements.connectionStatus.className = `status-dot ${status}`;
    
    const statusTexts = {
        connected: '已连接',
        disconnected: '连接中...',
        error: '连接错误'
    };
    
    elements.connectionText.textContent = statusTexts[status] || statusTexts.disconnected;
}

// 获取当前选择的指标和时间范围
function getCurrentSelection() {
    return {
        metric: elements.metricSelect.value,
        timeRange: parseInt(elements.timeRangeSelect.value)
    };
}

// 获取API基础URL
function getApiBaseUrl() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.hostname || 'localhost';
    const port = window.location.port || '8080';
    return `${protocol}//${host}:${port}`;
}

// 获取性能指标数据
async function fetchMetrics() {
    try {
        const response = await fetch(`${getApiBaseUrl()}/api/metrics`);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        const data = await response.json();
        updateMetricsDisplay(data);
        updateConnectionStatus('connected');
    } catch (error) {
        console.error('获取指标失败:', error);
        updateConnectionStatus('error');
    }
}

// 更新指标显示
function updateMetricsDisplay(data) {
    // 更新渲染指标
    if (data.render) {
        elements.fpsValue.textContent = data.render.fps.toFixed(1);
        elements.frameTimeValue.textContent = data.render.frame_time.toFixed(2);
        elements.drawCallsValue.textContent = data.render.draw_calls.toLocaleString();
    }
    
    // 更新系统指标
    if (data.system) {
        elements.cpuUsageValue.textContent = data.system.cpu_usage.toFixed(1);
        elements.memoryUsageValue.textContent = data.memory ? data.memory.allocated_mb.toFixed(1) : '--';
        elements.gpuUsageValue.textContent = data.system.gpu_usage.toFixed(1);
    }
    
    // 更新物理指标
    if (data.physics) {
        elements.physicsTimeValue.textContent = data.physics.calc_time.toFixed(2);
    }
    
    // 更新音频指标
    // 注意：当前实现中没有音频延迟指标，这里保留占位符
    elements.audioLatencyValue.textContent = '--';
}

// 获取图表数据
async function fetchChartData() {
    const { metric, timeRange } = getCurrentSelection();
    
    try {
        const params = new URLSearchParams({
            metric: metric,
            range: timeRange.toString()
        });
        
        const response = await fetch(`${getApiBaseUrl()}/api/chart-data?${params}`);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        const data = await response.json();
        updateChart(data);
    } catch (error) {
        console.error('获取图表数据失败:', error);
        showErrorMessage('获取图表数据失败');
    }
}

// 更新图表
function updateChart(data) {
    if (!chart || !data.labels || !data.values) {
        return;
    }
    
    chart.data.labels = data.labels;
    chart.data.datasets[0].data = data.values;
    chart.update();
}

// 获取告警信息
async function fetchAlerts() {
    try {
        const response = await fetch(`${getApiBaseUrl()}/api/alerts`);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        const alerts = await response.json();
        updateAlertsDisplay(alerts);
    } catch (error) {
        console.error('获取告警失败:', error);
        showErrorMessage('获取告警信息失败');
    }
}

// 更新告警显示
function updateAlertsDisplay(alerts) {
    if (!alerts || alerts.length === 0) {
        elements.alertsContainer.innerHTML = '<div class="no-alerts">暂无告警</div>';
        return;
    }
    
    const alertsHtml = alerts.map(alert => `
        <div class="alert-item ${alert.severity}">
            <div class="alert-content">
                <div class="alert-severity ${alert.severity}">${getSeverityIcon(alert.severity)} ${alert.severity.toUpperCase()}</div>
                <div class="alert-message">${alert.message}</div>
                <div class="alert-time">${formatTime(alert.timestamp)}</div>
            </div>
        </div>
    `).join('');
    
    elements.alertsContainer.innerHTML = alertsHtml;
}

// 获取严重性图标
function getSeverityIcon(severity) {
    const icons = {
        info: 'ℹ️',
        warning: '⚠️',
        error: '❌',
        critical: '🔥'
    };
    return icons[severity] || 'ℹ️';
}

// 格式化时间
function formatTime(timestamp) {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString('zh-CN', {
        hour12: false,
        minute: '2-digit',
        second: '2-digit'
    });
}

// 事件处理函数
function onMetricChange() {
    if (autoRefresh) {
        fetchChartData();
    }
}

function onTimeRangeChange() {
    if (autoRefresh) {
        fetchChartData();
    }
}

function onRefreshChart() {
    fetchChartData();
}

function onAutoRefreshToggle() {
    autoRefresh = elements.autoRefreshCheckbox.checked;
    if (autoRefresh) {
        startAutoRefresh();
    } else {
        stopAutoRefresh();
    }
}

function onClearAlerts() {
    // 这里应该调用API清除已确认的告警
    console.log('清除告警功能待实现');
}

// 自动刷新
function startAutoRefresh() {
    if (refreshInterval) {
        clearInterval(refreshInterval);
    }
    
    refreshInterval = setInterval(() => {
        fetchMetrics();
        fetchAlerts();
    }, 1000); // 每秒刷新一次
}

function stopAutoRefresh() {
    if (refreshInterval) {
        clearInterval(refreshInterval);
        refreshInterval = null;
    }
}

// 显示错误消息
function showErrorMessage(message) {
    const errorDiv = document.createElement('div');
    errorDiv.className = 'error-message';
    errorDiv.textContent = message;
    errorDiv.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        background: #e74c3c;
        color: white;
        padding: 10px 20px;
        border-radius: 5px;
        z-index: 1000;
        animation: slideIn 0.3s ease-out;
    `;
    
    document.body.appendChild(errorDiv);
    
    // 3秒后自动消失
    setTimeout(() => {
        if (errorDiv.parentNode) {
            errorDiv.parentNode.removeChild(errorDiv);
        }
    }, 3000);
}

// 页面加载完成后初始化
document.addEventListener('DOMContentLoaded', init);

// WebSocket连接（可选，用于实时更新）
function initWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.hostname}:8081/ws`;
    
    const ws = new WebSocket(wsUrl);
    
    ws.onopen = () => {
        updateConnectionStatus('connected');
        console.log('WebSocket连接已建立');
    };
    
    ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        
        if (data.type === 'metrics') {
            updateMetricsDisplay(data.payload);
        } else if (data.type === 'alerts') {
            updateAlertsDisplay(data.payload);
        }
    };
    
    ws.onclose = () => {
        updateConnectionStatus('disconnected');
        console.log('WebSocket连接已关闭');
        
        // 尝试重连
        setTimeout(initWebSocket, 5000);
    };
    
    ws.onerror = (error) => {
        console.error('WebSocket错误:', error);
        updateConnectionStatus('error');
    };
}

// 尝试WebSocket连接
if (window.WebSocket) {
    setTimeout(initWebSocket, 1000);
}