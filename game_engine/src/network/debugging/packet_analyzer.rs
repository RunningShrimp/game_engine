//  网络数据包分析模块
// 
//  实现网络数据包捕获、解析、过滤和可视化分析功能。
// 
//  ## 功能特性
// 
//  - 实时数据包捕获
//  - 多协议数据包解析
//  - 数据包内容可视化
//  - 数据包过滤和搜索
//  - 数据包时间线分析
//  - 自定义数据包处理器

use crate::core::utils::current_timestamp_ms;
use crate::network::{NetworkMessage, NetworkError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// 网络数据包分析器
pub struct NetworkPacketAnalyzer {
    /// 是否启用
    enabled: bool,
    /// 数据包历史
    packet_history: VecDeque<AnalyzedPacket>,
    /// 最大历史长度
    max_history_size: usize,
    /// 分析器配置
    config: AnalyzerConfig,
    /// 协议解析器
    protocol_parsers: HashMap<PacketProtocol, Box<dyn PacketParser>>,
    /// 数据包过滤器
    packet_filters: Vec<Box<dyn PacketFilter>>,
    /// 数据包处理器
    packet_processors: Vec<Box<dyn PacketProcessor>>,
    /// 统计信息
    statistics: PacketAnalysisStatistics,
    /// 时间线分析器
    timeline_analyzer: TimelineAnalyzer,
    /// 内容分析器
    content_analyzer: ContentAnalyzer,
    /// 最后更新时间
    #[allow(dead_code)]
    last_update: Instant,
    /// 捕获计数器
    capture_counter: u64,
}

/// 分析后的数据包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedPacket {
    /// 数据包ID
    pub packet_id: u64,
    /// 时间戳
    pub timestamp_ms: u64,
    /// 源地址
    pub source_address: String,
    /// 目标地址
    pub destination_address: String,
    /// 协议类型
    pub protocol: PacketProtocol,
    /// 数据包大小
    pub size: usize,
    /// 原始数据
    pub raw_data: Vec<u8>,
    /// 解析后的内容
    pub parsed_content: ParsedContent,
    /// 数据包类型
    pub packet_type: PacketType,
    /// 优先级
    pub priority: PacketPriority,
    /// 方向（发送/接收）
    pub direction: PacketDirection,
    /// 延迟信息（如果有）
    pub latency_info: Option<LatencyInfo>,
    /// 分析标记
    pub analysis_flags: AnalysisFlags,
    /// 自定义标签
    pub tags: Vec<String>,
}

/// 数据包协议
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PacketProtocol {
    /// 未知协议
    Unknown,
    /// TCP
    TCP,
    /// UDP,
    UDP,
    /// 自定义协议
    Custom,
}

/// 解析后的内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedContent {
    /// 消息类型
    pub message_type: Option<String>,
    /// 消息字段
    pub fields: HashMap<String, ParsedField>,
    /// 错误信息（如果有）
    pub parse_error: Option<String>,
    /// 解析深度
    pub parse_depth: u8,
}

/// 解析字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParsedField {
    /// 字符串值
    String(String),
    /// 整数值
    Integer(i64),
    /// 浮点数值
    Float(f64),
    /// 布尔值
    Boolean(bool),
    /// 字节数组
    Bytes(Vec<u8>),
    /// 嵌套对象
    Object(HashMap<String, ParsedField>),
    /// 数组
    Array(Vec<ParsedField>),
}

/// 数据包类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Hash)]
pub enum PacketType {
    /// 未知类型
    Unknown,
    /// 连接请求
    Connect,
    /// 断开连接
    Disconnect,
    /// 状态同步
    StateSync,
    /// 输入数据
    Input,
    /// RPC调用
    Rpc,
    /// 心跳
    Heartbeat,
    /// 时间同步
    TimeSync,
    /// 事件同步
    EventSync,
    /// 自定义类型
    Custom,
}

/// 数据包优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PacketPriority {
    /// 低优先级
    Low = 1,
    /// 中优先级
    Medium = 2,
    /// 高优先级
    High = 3,
    /// 关键优先级
    Critical = 4,
}

/// 数据包方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Hash)]
pub enum PacketDirection {
    /// 发送
    Outgoing,
    /// 接收
    Incoming,
    /// 未知方向
    Unknown,
}

/// 延迟信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyInfo {
    /// 发送时间戳
    pub send_timestamp_ms: u64,
    /// 接收时间戳
    pub receive_timestamp_ms: u64,
    /// 往返时间（毫秒）
    pub round_trip_time_ms: f32,
    /// 单向延迟（毫秒）
    pub one_way_latency_ms: f32,
}

/// 分析标记
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisFlags {
    /// 是否异常
    pub is_anomaly: bool,
    /// 是否重传
    pub is_retransmission: bool,
    /// 是否乱序
    pub is_out_of_order: bool,
    /// 是否损坏
    pub is_corrupted: bool,
    /// 是否被压缩
    pub is_compressed: bool,
    /// 是否被加密
    pub is_encrypted: bool,
    /// 是否包含错误
    pub contains_error: bool,
    /// 是否包含警告
    pub contains_warning: bool,
}

/// 分析器配置
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// 最大历史长度
    pub max_history_size: usize,
    /// 是否启用深度解析
    pub enable_deep_parsing: bool,
    /// 是否启用内容分析
    pub enable_content_analysis: bool,
    /// 是否启用时间线分析
    pub enable_timeline_analysis: bool,
    /// 更新间隔（毫秒）
    pub update_interval_ms: u64,
    /// 自动检测协议
    pub auto_detect_protocol: bool,
    /// 保存原始数据
    pub save_raw_data: bool,
    /// 最大原始数据大小
    pub max_raw_data_size: usize,
    /// 异常检测阈值
    pub anomaly_threshold: f32,
}

/// 数据包分析统计
#[derive(Debug, Clone)]
pub struct PacketAnalysisStatistics {
    /// 总分析包数
    pub total_packets_analyzed: u64,
    /// 按协议分类的统计
    pub protocol_stats: HashMap<PacketProtocol, u64>,
    /// 按类型分类的统计
    pub type_stats: HashMap<PacketType, u64>,
    /// 按方向分类的统计
    pub direction_stats: HashMap<PacketDirection, u64>,
    /// 平均包大小
    pub average_packet_size: f32,
    /// 最大包大小
    pub max_packet_size: usize,
    /// 最小包大小
    pub min_packet_size: usize,
    /// 异常包数
    pub anomaly_count: u64,
    /// 重传包数
    pub retransmission_count: u64,
    /// 乱序包数
    pub out_of_order_count: u64,
    /// 损坏包数
    pub corrupted_count: u64,
    /// 解析错误数
    pub parse_error_count: u64,
    /// 分析开始时间
    pub analysis_start_time: Instant,
}

/// 时间线分析器
#[derive(Debug)]
struct TimelineAnalyzer {
    /// 时间线数据
    timeline_data: VecDeque<TimelineEntry>,
    /// 最大时间线长度
    max_timeline_length: usize,
    /// 时间窗口大小（毫秒）
    #[allow(dead_code)]
    time_window_ms: u64,
    /// 流量模式
    traffic_patterns: Vec<TrafficPattern>,
}

/// 时间线条目
#[derive(Debug, Clone)]
pub struct TimelineEntry {
    /// 时间戳
    timestamp: Instant,
    /// 包计数
    packet_count: u64,
    /// 字节数
    byte_count: u64,
    /// 平均包大小
    #[allow(dead_code)]
    average_packet_size: f32,
    /// 协议分布
    protocol_distribution: HashMap<PacketProtocol, u64>,
}

/// 流量模式
#[derive(Debug, Clone)]
pub struct TrafficPattern {
    /// 模式类型
    #[allow(dead_code)]
    pattern_type: PatternType,
    /// 开始时间
    #[allow(dead_code)]
    start_time: Instant,
    /// 结束时间
    #[allow(dead_code)]
    end_time: Instant,
    /// 强度（0-1）
    #[allow(dead_code)]
    intensity: f32,
    /// 描述
    #[allow(dead_code)]
    description: String,
}

/// 模式类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PatternType {
    /// 正常流量
    #[allow(dead_code)]
    Normal,
    /// 流量突发
    #[allow(dead_code)]
    Burst,
    /// 流量低谷
    #[allow(dead_code)]
    Lull,
    /// 异常流量
    #[allow(dead_code)]
    Anomalous,
    /// 周期性流量
    #[allow(dead_code)]
    Periodic,
}

/// 内容分析器
#[derive(Debug)]
struct ContentAnalyzer {
    /// 字符频率统计
    char_frequency: HashMap<char, u64>,
    /// 字节模式统计
    byte_patterns: HashMap<[u8; 4], u64>,
    /// 常见字符串
    common_strings: HashMap<String, u64>,
    /// 压缩检测器
    compression_detector: CompressionDetector,
    /// 加密检测器
    encryption_detector: EncryptionDetector,
}

/// 压缩检测器
#[derive(Debug)]
struct CompressionDetector {
    /// 压缩算法检测
    #[allow(dead_code)]
    compression_algorithms: Vec<CompressionAlgorithm>,
    /// 检测阈值
    #[allow(dead_code)]
    detection_threshold: f32,
}

/// 压缩算法
#[derive(Debug, Clone, Copy)]
enum CompressionAlgorithm {
    Gzip,
    Deflate,
    Lz4,
    Zstd,
}

/// 加密检测器
#[derive(Debug)]
struct EncryptionDetector {
    /// 随机性检测器
    randomness_detector: RandomnessDetector,
    /// 模式检测器
    pattern_detector: PatternDetector,
}

/// 随机性检测器
#[derive(Debug)]
struct RandomnessDetector {
    /// 熵阈值
    entropy_threshold: f32,
}

/// 模式检测器
#[derive(Debug)]
struct PatternDetector {
    /// 常见模式
    common_patterns: Vec<Vec<u8>>,
}

/// 数据包解析器特征
pub trait PacketParser: Send + Sync {
    /// 解析数据包
    fn parse(&self, data: &[u8]) -> Result<ParsedContent, String>;
    /// 获取支持的协议
    fn supported_protocol(&self) -> PacketProtocol;
    /// 克隆解析器
    fn clone_box(&self) -> Box<dyn PacketParser>;
}

/// 数据包过滤器特征
pub trait PacketFilter: Send + Sync {
    /// 检查数据包是否通过过滤器
    fn filter(&self, packet: &AnalyzedPacket) -> bool;
    /// 获取过滤器名称
    fn name(&self) -> &str;
    /// 克隆过滤器
    fn clone_box(&self) -> Box<dyn PacketFilter>;
}

/// 数据包处理器特征
pub trait PacketProcessor: Send + Sync {
    /// 处理数据包
    fn process(&mut self, packet: &mut AnalyzedPacket);
    /// 获取处理器名称
    fn name(&self) -> &str;
    /// 克隆处理器
    fn clone_box(&self) -> Box<dyn PacketProcessor>;
}

impl NetworkPacketAnalyzer {
    /// 创建新的网络数据包分析器
    pub fn new() -> Self {
        Self::with_config(AnalyzerConfig::default())
    }

    /// 创建带配置的网络数据包分析器
    pub fn with_config(config: AnalyzerConfig) -> Self {
        let mut analyzer = Self {
            enabled: true,
            packet_history: VecDeque::with_capacity(config.max_history_size),
            max_history_size: config.max_history_size,
            config,
            protocol_parsers: HashMap::new(),
            packet_filters: Vec::new(),
            packet_processors: Vec::new(),
            statistics: PacketAnalysisStatistics {
                total_packets_analyzed: 0,
                protocol_stats: HashMap::new(),
                type_stats: HashMap::new(),
                direction_stats: HashMap::new(),
                average_packet_size: 0.0,
                max_packet_size: 0,
                min_packet_size: usize::MAX,
                anomaly_count: 0,
                retransmission_count: 0,
                out_of_order_count: 0,
                corrupted_count: 0,
                parse_error_count: 0,
                analysis_start_time: Instant::now(),
            },
            timeline_analyzer: TimelineAnalyzer::new(),
            content_analyzer: ContentAnalyzer::new(),
            last_update: Instant::now(),
            capture_counter: 0,
        };

        // 注册默认解析器
        analyzer.register_default_parsers();
        
        // 注册默认过滤器
        analyzer.register_default_filters();
        
        // 注册默认处理器
        analyzer.register_default_processors();

        analyzer
    }

    /// 分析数据包
    pub fn analyze_packet(
        &mut self,
        data: &[u8],
        source_address: &str,
        destination_address: &str,
        direction: PacketDirection,
    ) -> Result<AnalyzedPacket, NetworkError> {
        if !self.enabled {
            return Err(NetworkError::ReceiveError("Analyzer disabled".to_string()));
        }

        self.capture_counter += 1;
        let packet_id = self.capture_counter;
        let timestamp_ms = current_timestamp_ms();
        let size = data.len();

        // 检测协议
        let protocol = if self.config.auto_detect_protocol {
            self.detect_protocol(data)
        } else {
            PacketProtocol::Unknown
        };

        // 解析内容
        let parsed_content = if let Some(parser) = self.protocol_parsers.get(&protocol) {
            parser.parse(data).unwrap_or_else(|error| ParsedContent {
                message_type: None,
                fields: HashMap::new(),
                parse_error: Some(error),
                parse_depth: 0,
            })
        } else {
            ParsedContent {
                message_type: None,
                fields: HashMap::new(),
                parse_error: Some("No parser for protocol".to_string()),
                parse_depth: 0,
            }
        };

        // 确定数据包类型
        let packet_type = self.determine_packet_type(&parsed_content);

        // 确定优先级
        let priority = self.determine_packet_priority(packet_type, size);

        // 创建分析标记
        let analysis_flags = self.create_analysis_flags(data, &parsed_content);

        // 保存原始数据（如果需要）
        let raw_data = if self.config.save_raw_data {
            let max_size = self.config.max_raw_data_size;
            if data.len() <= max_size {
                data.to_vec()
            } else {
                data[..max_size].to_vec()
            }
        } else {
            Vec::new()
        };

        // 创建分析后的数据包
        let mut packet = AnalyzedPacket {
            packet_id,
            timestamp_ms,
            source_address: source_address.to_string(),
            destination_address: destination_address.to_string(),
            protocol,
            size,
            raw_data,
            parsed_content,
            packet_type,
            priority,
            direction,
            latency_info: None,
            analysis_flags,
            tags: Vec::new(),
        };

        // 应用数据包处理器
        for processor in &mut self.packet_processors {
            processor.process(&mut packet);
        }

        // 应用过滤器
        let passes_filters = self.packet_filters.iter().all(|filter| filter.filter(&packet));
        if !passes_filters {
            return Err(NetworkError::ReceiveError("Packet filtered out".to_string()));
        }

        // 添加到历史
        self.packet_history.push_back(packet.clone());

        // 限制历史长度
        while self.packet_history.len() > self.max_history_size {
            self.packet_history.pop_front();
        }

        // 更新统计信息
        self.update_statistics(&packet);

        // 更新时间线分析器
        if self.config.enable_timeline_analysis {
            self.timeline_analyzer.add_packet(&packet);
        }

        // 更新内容分析器
        if self.config.enable_content_analysis {
            self.content_analyzer.analyze_packet(&packet);
        }

        Ok(packet)
    }

    /// 分析网络消息
    pub fn analyze_network_message(
        &mut self,
        message: &NetworkMessage,
        source_address: &str,
        destination_address: &str,
        direction: PacketDirection,
    ) -> Result<AnalyzedPacket, NetworkError> {
        // 序列化消息
        let data = bincode_compat::serialize(message).map_err(|e| Box::new(e))
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        self.analyze_packet(&data, source_address, destination_address, direction)
    }

    /// 搜索数据包
    pub fn search_packets(&self, query: &PacketSearchQuery) -> Vec<AnalyzedPacket> {
        self.packet_history
            .iter()
            .filter(|packet| self.matches_query(packet, query))
            .cloned()
            .collect()
    }

    /// 获取数据包历史
    pub fn get_packet_history(&self) -> Vec<AnalyzedPacket> {
        self.packet_history.iter().cloned().collect()
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> &PacketAnalysisStatistics {
        &self.statistics
    }

    /// 获取时间线数据
    pub fn get_timeline_data(&self) -> Vec<TimelineEntry> {
        self.timeline_analyzer.get_timeline_data()
    }

    /// 获取流量模式
    pub fn get_traffic_patterns(&self) -> Vec<TrafficPattern> {
        self.timeline_analyzer.get_traffic_patterns()
    }

    /// 注册协议解析器
    pub fn register_parser(&mut self, parser: Box<dyn PacketParser>) {
        let protocol = parser.supported_protocol();
        self.protocol_parsers.insert(protocol, parser);
    }

    /// 添加数据包过滤器
    pub fn add_filter(&mut self, filter: Box<dyn PacketFilter>) {
        self.packet_filters.push(filter);
    }

    /// 添加数据包处理器
    pub fn add_processor(&mut self, processor: Box<dyn PacketProcessor>) {
        self.packet_processors.push(processor);
    }

    /// 清除过滤器
    pub fn clear_filters(&mut self) {
        self.packet_filters.clear();
    }

    /// 清除处理器
    pub fn clear_processors(&mut self) {
        self.packet_processors.clear();
    }

    /// 重置分析器
    pub fn reset(&mut self) {
        self.packet_history.clear();
        self.statistics = PacketAnalysisStatistics::default();
        self.timeline_analyzer.reset();
        self.content_analyzer.reset();
        self.capture_counter = 0;
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 设置启用状态
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 检查是否活跃
    pub fn is_active(&self) -> bool {
        self.enabled && !self.packet_history.is_empty()
    }

    // 私有方法

    /// 注册默认解析器
    fn register_default_parsers(&mut self) {
        // 注册网络消息解析器
        self.register_parser(Box::new(NetworkMessageParser::new()));
        
        // 可以添加更多解析器
        // self.register_parser(Box::new(TcpParser::new()));
        // self.register_parser(Box::new(UdpParser::new()));
    }

    /// 注册默认过滤器
    fn register_default_filters(&mut self) {
        // 可以添加默认过滤器
        // self.add_filter(Box::new(SizeFilter::new(0, 1500)));
        // self.add_filter(Box::new(ProtocolFilter::new(vec![PacketProtocol::TCP, PacketProtocol::UDP])));
    }

    /// 注册默认处理器
    fn register_default_processors(&mut self) {
        // 添加异常检测处理器
        self.add_processor(Box::new(AnomalyDetectionProcessor::new()));
        
        // 添加标签处理器
        self.add_processor(Box::new(TaggingProcessor::new()));
    }

    /// 检测协议
    fn detect_protocol(&self, data: &[u8]) -> PacketProtocol {
        // 简单的协议检测逻辑
        if data.len() < 4 {
            return PacketProtocol::Unknown;
        }

        // 检查是否是网络消息
        if let Ok(_) = bincode_compat::deserialize(data) {
            return PacketProtocol::Custom;
        }

        // 可以添加更多协议检测逻辑
        // 检查TCP/UDP头部等

        PacketProtocol::Unknown
    }

    /// 确定数据包类型
    fn determine_packet_type(&self, content: &ParsedContent) -> PacketType {
        if let Some(message_type) = &content.message_type {
            match message_type.as_str() {
                "Connect" => PacketType::Connect,
                "Disconnect" => PacketType::Disconnect,
                "StateSync" => PacketType::StateSync,
                "Input" => PacketType::Input,
                "Rpc" => PacketType::Rpc,
                "Heartbeat" => PacketType::Heartbeat,
                "TimeSync" => PacketType::TimeSync,
                "EventSync" => PacketType::EventSync,
                _ => PacketType::Custom,
            }
        } else {
            PacketType::Unknown
        }
    }

    /// 确定数据包优先级
    fn determine_packet_priority(&self, packet_type: PacketType, size: usize) -> PacketPriority {
        match packet_type {
            PacketType::Connect | PacketType::Disconnect => PacketPriority::High,
            PacketType::Input => PacketPriority::Critical,
            PacketType::Heartbeat => PacketPriority::Low,
            PacketType::TimeSync => PacketPriority::High,
            _ => {
                // 基于大小确定优先级
                if size < 100 {
                    PacketPriority::Medium
                } else if size < 500 {
                    PacketPriority::Low
                } else {
                    PacketPriority::Medium
                }
            }
        }
    }

    /// 创建分析标记
    fn create_analysis_flags(&self, data: &[u8], content: &ParsedContent) -> AnalysisFlags {
        AnalysisFlags {
            is_anomaly: self.detect_anomaly(data),
            is_retransmission: false, // 需要更多上下文信息
            is_out_of_order: false,   // 需要更多上下文信息
            is_corrupted: content.parse_error.is_some(),
            is_compressed: self.detect_compression(data),
            is_encrypted: self.detect_encryption(data),
            contains_error: content.fields.contains_key("error"),
            contains_warning: content.fields.contains_key("warning"),
        }
    }

    /// 检测异常
    fn detect_anomaly(&self, data: &[u8]) -> bool {
        // 简单的异常检测：检查数据包大小是否异常
        let average_size = self.statistics.average_packet_size;
        let size = data.len() as f32;
        
        // 如果大小偏离平均值超过3个标准差，认为是异常
        if average_size > 0.0 {
            let deviation = (size - average_size).abs() / average_size;
            deviation > self.config.anomaly_threshold
        } else {
            false
        }
    }

    /// 检测压缩
    fn detect_compression(&self, data: &[u8]) -> bool {
        // 简单的压缩检测
        data.len() > 4 && (data[0] == 0x1f && data[1] == 0x8b) || // Gzip
                           (data[0] == 0x78 && (data[1] == 0x9c || data[1] == 0xda)) // Deflate
    }

    /// 检测加密
    fn detect_encryption(&self, data: &[u8]) -> bool {
        // 简单的加密检测：检查数据的随机性
        if data.len() < 8 {
            return false;
        }

        // 计算熵
        let mut frequency = [0u8; 256];
        for &byte in data {
            frequency[byte as usize] += 1;
        }

        let len = data.len() as f32;
        let mut entropy = 0.0;
        for &count in &frequency {
            if count > 0 {
                let probability = count as f32 / len;
                entropy -= probability * probability.log2();
            }
        }

        // 如果熵接近8.0，可能是加密数据
        entropy > 7.5
    }

    /// 更新统计信息
    fn update_statistics(&mut self, packet: &AnalyzedPacket) {
        self.statistics.total_packets_analyzed += 1;
        
        // 更新协议统计
        *self.statistics.protocol_stats.entry(packet.protocol).or_insert(0) += 1;
        
        // 更新类型统计
        *self.statistics.type_stats.entry(packet.packet_type).or_insert(0) += 1;
        
        // 更新方向统计
        *self.statistics.direction_stats.entry(packet.direction).or_insert(0) += 1;
        
        // 更新大小统计
        let size = packet.size as f32;
        if self.statistics.total_packets_analyzed == 1 {
            self.statistics.average_packet_size = size;
            self.statistics.max_packet_size = packet.size;
            self.statistics.min_packet_size = packet.size;
        } else {
            let n = self.statistics.total_packets_analyzed as f32;
            self.statistics.average_packet_size = 
                (self.statistics.average_packet_size * (n - 1.0) + size) / n;
            self.statistics.max_packet_size = self.statistics.max_packet_size.max(packet.size);
            self.statistics.min_packet_size = self.statistics.min_packet_size.min(packet.size);
        }
        
        // 更新异常统计
        if packet.analysis_flags.is_anomaly {
            self.statistics.anomaly_count += 1;
        }
        if packet.analysis_flags.is_retransmission {
            self.statistics.retransmission_count += 1;
        }
        if packet.analysis_flags.is_out_of_order {
            self.statistics.out_of_order_count += 1;
        }
        if packet.analysis_flags.is_corrupted {
            self.statistics.corrupted_count += 1;
        }
        if packet.parsed_content.parse_error.is_some() {
            self.statistics.parse_error_count += 1;
        }
    }

    /// 检查数据包是否匹配查询
    fn matches_query(&self, packet: &AnalyzedPacket, query: &PacketSearchQuery) -> bool {
        // 检查时间范围
        if let Some(start_time) = query.start_time_ms {
            if packet.timestamp_ms < start_time {
                return false;
            }
        }
        if let Some(end_time) = query.end_time_ms {
            if packet.timestamp_ms > end_time {
                return false;
            }
        }

        // 检查协议
        if let Some(protocols) = &query.protocols {
            if !protocols.contains(&packet.protocol) {
                return false;
            }
        }

        // 检查数据包类型
        if let Some(packet_types) = &query.packet_types {
            if !packet_types.contains(&packet.packet_type) {
                return false;
            }
        }

        // 检查方向
        if let Some(directions) = &query.directions {
            if !directions.contains(&packet.direction) {
                return false;
            }
        }

        // 检查大小范围
        if let Some(min_size) = query.min_size {
            if packet.size < min_size {
                return false;
            }
        }
        if let Some(max_size) = query.max_size {
            if packet.size > max_size {
                return false;
            }
        }

        // 检查地址
        if let Some(source_address) = &query.source_address {
            if !packet.source_address.contains(source_address) {
                return false;
            }
        }
        if let Some(destination_address) = &query.destination_address {
            if !packet.destination_address.contains(destination_address) {
                return false;
            }
        }

        // 检查内容
        if let Some(content_search) = &query.content_search {
            let content = format!("{:?}", packet.parsed_content);
            if !content.contains(content_search) {
                return false;
            }
        }

        // 检查标签
        if let Some(required_tags) = &query.tags {
            if !required_tags.iter().all(|tag| packet.tags.contains(tag)) {
                return false;
            }
        }

        true
    }
}

/// 数据包搜索查询
#[derive(Debug, Clone)]
pub struct PacketSearchQuery {
    /// 开始时间（毫秒）
    pub start_time_ms: Option<u64>,
    /// 结束时间（毫秒）
    pub end_time_ms: Option<u64>,
    /// 协议过滤
    pub protocols: Option<Vec<PacketProtocol>>,
    /// 数据包类型过滤
    pub packet_types: Option<Vec<PacketType>>,
    /// 方向过滤
    pub directions: Option<Vec<PacketDirection>>,
    /// 最小大小
    pub min_size: Option<usize>,
    /// 最大大小
    pub max_size: Option<usize>,
    /// 源地址过滤
    pub source_address: Option<String>,
    /// 目标地址过滤
    pub destination_address: Option<String>,
    /// 内容搜索
    pub content_search: Option<String>,
    /// 标签过滤
    pub tags: Option<Vec<String>>,
}

/// 网络消息解析器
#[derive(Debug)]
struct NetworkMessageParser {
    #[allow(dead_code)]
    name: String,
}

impl NetworkMessageParser {
    fn new() -> Self {
        Self {
            name: "NetworkMessageParser".to_string(),
        }
    }
}

impl PacketParser for NetworkMessageParser {
    fn parse(&self, data: &[u8]) -> Result<ParsedContent, String> {
        match bincode_compat::deserialize(data).map(|(msg, _)| msg) {
            Ok(message) => {
                let mut fields = HashMap::new();
                let message_type = match &message {
                    NetworkMessage::Connect { .. } => {
                        fields.insert("type".to_string(), ParsedField::String("Connect".to_string()));
                        Some("Connect".to_string())
                    }
                    NetworkMessage::Disconnect { .. } => {
                        fields.insert("type".to_string(), ParsedField::String("Disconnect".to_string()));
                        Some("Disconnect".to_string())
                    }
                    NetworkMessage::StateSync { tick, .. } => {
                        fields.insert("type".to_string(), ParsedField::String("StateSync".to_string()));
                        fields.insert("tick".to_string(), ParsedField::Integer(*tick as i64));
                        Some("StateSync".to_string())
                    }
                    NetworkMessage::Input { tick, .. } => {
                        fields.insert("type".to_string(), ParsedField::String("Input".to_string()));
                        fields.insert("tick".to_string(), ParsedField::Integer(*tick as i64));
                        Some("Input".to_string())
                    }
                    NetworkMessage::Rpc { id, method, .. } => {
                        fields.insert("type".to_string(), ParsedField::String("Rpc".to_string()));
                        fields.insert("id".to_string(), ParsedField::Integer(*id as i64));
                        fields.insert("method".to_string(), ParsedField::String(method.clone()));
                        Some("Rpc".to_string())
                    }
                    NetworkMessage::Heartbeat { timestamp } => {
                        fields.insert("type".to_string(), ParsedField::String("Heartbeat".to_string()));
                        fields.insert("timestamp".to_string(), ParsedField::Integer(*timestamp as i64));
                        Some("Heartbeat".to_string())
                    }
                    NetworkMessage::TimeSyncRequest { .. } => {
                        fields.insert("type".to_string(), ParsedField::String("TimeSyncRequest".to_string()));
                        Some("TimeSyncRequest".to_string())
                    }
                    NetworkMessage::TimeSyncResponse { .. } => {
                        fields.insert("type".to_string(), ParsedField::String("TimeSyncResponse".to_string()));
                        Some("TimeSyncResponse".to_string())
                    }
                    NetworkMessage::EventSync { .. } => {
                        fields.insert("type".to_string(), ParsedField::String("EventSync".to_string()));
                        Some("EventSync".to_string())
                    }
                    NetworkMessage::RpcResponse { .. } => {
                        fields.insert("type".to_string(), ParsedField::String("RpcResponse".to_string()));
                        Some("RpcResponse".to_string())
                    }
                };

                Ok(ParsedContent {
                    message_type,
                    fields,
                    parse_error: None,
                    parse_depth: 1,
                })
            }
            Err(e) => Err(format!("Failed to parse NetworkMessage: {}", e)),
        }
    }

    fn supported_protocol(&self) -> PacketProtocol {
        PacketProtocol::Custom
    }

    fn clone_box(&self) -> Box<dyn PacketParser> {
        Box::new(Self::new())
    }
}

/// 异常检测处理器
#[derive(Debug)]
struct AnomalyDetectionProcessor {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    threshold: f32,
}

impl AnomalyDetectionProcessor {
    fn new() -> Self {
        Self {
            name: "AnomalyDetectionProcessor".to_string(),
            threshold: 3.0, // 3个标准差
        }
    }
}

impl PacketProcessor for AnomalyDetectionProcessor {
    fn process(&mut self, _packet: &mut AnalyzedPacket) {
        // 这里可以实现更复杂的异常检测逻辑
        // 目前只使用简单的基于大小的异常检测
        
        // 可以添加更多异常检测规则
        // 例如：检测异常高频的数据包、异常的协议组合等
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn clone_box(&self) -> Box<dyn PacketProcessor> {
        Box::new(Self::new())
    }
}

/// 标签处理器
#[derive(Debug)]
struct TaggingProcessor {
    name: String,
}

impl TaggingProcessor {
    fn new() -> Self {
        Self {
            name: "TaggingProcessor".to_string(),
        }
    }
}

impl PacketProcessor for TaggingProcessor {
    fn process(&mut self, packet: &mut AnalyzedPacket) {
        // 根据数据包类型添加标签
        match packet.packet_type {
            PacketType::Connect => packet.tags.push("connection".to_string()),
            PacketType::Disconnect => packet.tags.push("disconnection".to_string()),
            PacketType::StateSync => packet.tags.push("state".to_string()),
            PacketType::Input => packet.tags.push("input".to_string()),
            PacketType::Rpc => packet.tags.push("rpc".to_string()),
            PacketType::Heartbeat => packet.tags.push("heartbeat".to_string()),
            PacketType::TimeSync => packet.tags.push("time_sync".to_string()),
            PacketType::EventSync => packet.tags.push("event".to_string()),
            _ => {}
        }

        // 根据大小添加标签
        if packet.size > 1000 {
            packet.tags.push("large".to_string());
        } else if packet.size < 100 {
            packet.tags.push("small".to_string());
        }

        // 根据分析标记添加标签
        if packet.analysis_flags.is_anomaly {
            packet.tags.push("anomaly".to_string());
        }
        if packet.analysis_flags.is_compressed {
            packet.tags.push("compressed".to_string());
        }
        if packet.analysis_flags.is_encrypted {
            packet.tags.push("encrypted".to_string());
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn clone_box(&self) -> Box<dyn PacketProcessor> {
        Box::new(Self::new())
    }
}

// 时间线分析器实现
impl TimelineAnalyzer {
    fn new() -> Self {
        Self {
            timeline_data: VecDeque::with_capacity(1000),
            max_timeline_length: 1000,
            time_window_ms: 60000, // 1分钟窗口
            traffic_patterns: Vec::new(),
        }
    }

    fn add_packet(&mut self, packet: &AnalyzedPacket) {
        let timestamp = Instant::now();
        
        // 查找或创建时间线条目
        if let Some(entry) = self.timeline_data.back_mut() {
            let entry_age = timestamp.duration_since(entry.timestamp).as_millis() as u64;
            if entry_age < 1000 { // 1秒内的数据合并到同一条目
                entry.packet_count += 1;
                entry.byte_count += packet.size as u64;
                *entry.protocol_distribution.entry(packet.protocol).or_insert(0) += 1;
                return;
            }
        }

        // 创建新的时间线条目
        let mut protocol_distribution = HashMap::new();
        protocol_distribution.insert(packet.protocol, 1);

        self.timeline_data.push_back(TimelineEntry {
            timestamp,
            packet_count: 1,
            byte_count: packet.size as u64,
            average_packet_size: packet.size as f32,
            protocol_distribution,
        });

        // 限制时间线长度
        while self.timeline_data.len() > self.max_timeline_length {
            self.timeline_data.pop_front();
        }

        // 分析流量模式
        self.analyze_traffic_patterns();
    }

    fn get_timeline_data(&self) -> Vec<TimelineEntry> {
        self.timeline_data.iter().cloned().collect()
    }

    fn get_traffic_patterns(&self) -> Vec<TrafficPattern> {
        self.traffic_patterns.clone()
    }

    fn analyze_traffic_patterns(&mut self) {
        // 简单的流量模式分析
        if self.timeline_data.len() < 10 {
            return;
        }

        let recent_entries: Vec<_> = self.timeline_data.iter().rev().take(10).collect();
        let packet_counts: Vec<u64> = recent_entries.iter().map(|e| e.packet_count).collect();

        if packet_counts.is_empty() {
            return;
        }

        let average = packet_counts.iter().sum::<u64>() as f32 / packet_counts.len() as f32;
        let max = *packet_counts.iter().max().expect("packet_counts is not empty");
        let min = *packet_counts.iter().min().expect("packet_counts is not empty");

        let now = Instant::now();

        // 检测流量突发
        if max as f32 > average * 2.0 {
            self.traffic_patterns.push(TrafficPattern {
                pattern_type: PatternType::Burst,
                start_time: now - Duration::from_secs(10),
                end_time: now,
                intensity: (max as f32 - average) / average,
                description: "Traffic burst detected".to_string(),
            });
        }

        // 检测流量低谷
        if (min as f32) < average * 0.5 {
            self.traffic_patterns.push(TrafficPattern {
                pattern_type: PatternType::Lull,
                start_time: now - Duration::from_secs(10),
                end_time: now,
                intensity: (average - min as f32) / average,
                description: "Traffic lull detected".to_string(),
            });
        }

        // 限制模式数量
        if self.traffic_patterns.len() > 100 {
            self.traffic_patterns.drain(0..50);
        }
    }

    fn reset(&mut self) {
        self.timeline_data.clear();
        self.traffic_patterns.clear();
    }
}

// 内容分析器实现
impl ContentAnalyzer {
    fn new() -> Self {
        Self {
            char_frequency: HashMap::new(),
            byte_patterns: HashMap::new(),
            common_strings: HashMap::new(),
            compression_detector: CompressionDetector::new(),
            encryption_detector: EncryptionDetector::new(),
        }
    }

    fn analyze_packet(&mut self, packet: &AnalyzedPacket) {
        // 分析字符频率
        for &byte in &packet.raw_data {
            if let Some(ch) = char::from_u32(byte as u32) {
                *self.char_frequency.entry(ch).or_insert(0) += 1;
            }
        }

        // 分析字节模式
        if packet.raw_data.len() >= 4 {
            for i in 0..=packet.raw_data.len() - 4 {
                let pattern = [
                    packet.raw_data[i],
                    packet.raw_data[i + 1],
                    packet.raw_data[i + 2],
                    packet.raw_data[i + 3],
                ];
                *self.byte_patterns.entry(pattern).or_insert(0) += 1;
            }
        }

        // 检测压缩
        if self.compression_detector.detect_compression(&packet.raw_data) {
            // 可以添加压缩分析逻辑
        }

        // 检测加密
        if self.encryption_detector.detect_encryption(&packet.raw_data) {
            // 可以添加加密分析逻辑
        }
    }

    fn reset(&mut self) {
        self.char_frequency.clear();
        self.byte_patterns.clear();
        self.common_strings.clear();
    }
}

// 压缩检测器实现
impl CompressionDetector {
    fn new() -> Self {
        Self {
            compression_algorithms: vec![
                CompressionAlgorithm::Gzip,
                CompressionAlgorithm::Deflate,
                CompressionAlgorithm::Lz4,
                CompressionAlgorithm::Zstd,
            ],
            detection_threshold: 0.8,
        }
    }

    fn detect_compression(&self, data: &[u8]) -> bool {
        // 简单的压缩检测
        data.len() > 4 && (data[0] == 0x1f && data[1] == 0x8b) || // Gzip
                           (data[0] == 0x78 && (data[1] == 0x9c || data[1] == 0xda)) // Deflate
    }
}

// 加密检测器实现
impl EncryptionDetector {
    fn new() -> Self {
        Self {
            randomness_detector: RandomnessDetector::new(),
            pattern_detector: PatternDetector::new(),
        }
    }

    fn detect_encryption(&self, data: &[u8]) -> bool {
        self.randomness_detector.detect_randomness(data) ||
        self.pattern_detector.detect_encryption_patterns(data)
    }
}

// 随机性检测器实现
impl RandomnessDetector {
    fn new() -> Self {
        Self {
            entropy_threshold: 7.5,
        }
    }

    fn detect_randomness(&self, data: &[u8]) -> bool {
        if data.len() < 8 {
            return false;
        }

        // 计算熵
        let mut frequency = [0u8; 256];
        for &byte in data {
            frequency[byte as usize] += 1;
        }

        let len = data.len() as f32;
        let mut entropy = 0.0;
        for &count in &frequency {
            if count > 0 {
                let probability = count as f32 / len;
                entropy -= probability * probability.log2();
            }
        }

        entropy > self.entropy_threshold
    }
}

// 模式检测器实现
impl PatternDetector {
    fn new() -> Self {
        Self {
            common_patterns: vec![
                vec![0x89, 0x50, 0x4E, 0x47], // PNG
                vec![0xFF, 0xD8, 0xFF, 0xE0], // JPEG
                vec![0x47, 0x49, 0x46, 0x38], // GIF
            ],
        }
    }

    fn detect_encryption_patterns(&self, data: &[u8]) -> bool {
        // 检查是否包含常见的文件头
        if data.len() >= 4 {
            let header = &data[..4];
            for pattern in &self.common_patterns {
                if header == pattern {
                    return false; // 发现已知模式，可能不是加密数据
                }
            }
        }

        // 如果没有发现已知模式，可能是加密数据
        true
    }
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            max_history_size: 10000,
            enable_deep_parsing: true,
            enable_content_analysis: true,
            enable_timeline_analysis: true,
            update_interval_ms: 100,
            auto_detect_protocol: true,
            save_raw_data: true,
            max_raw_data_size: 1024,
            anomaly_threshold: 3.0,
        }
    }
}

impl Default for NetworkPacketAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PacketAnalysisStatistics {
    fn default() -> Self {
        Self {
            total_packets_analyzed: 0,
            protocol_stats: HashMap::new(),
            type_stats: HashMap::new(),
            direction_stats: HashMap::new(),
            average_packet_size: 0.0,
            max_packet_size: 0,
            min_packet_size: usize::MAX,
            anomaly_count: 0,
            retransmission_count: 0,
            out_of_order_count: 0,
            corrupted_count: 0,
            parse_error_count: 0,
            analysis_start_time: Instant::now(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_analyzer_creation() {
        let analyzer = NetworkPacketAnalyzer::new();
        assert!(analyzer.is_enabled());
        assert_eq!(analyzer.get_statistics().total_packets_analyzed, 0);
    }

    #[test]
    fn test_packet_analysis() {
        let mut analyzer = NetworkPacketAnalyzer::new();
        
        let data = b"test packet data";
        let result = analyzer.analyze_packet(
            data,
            "127.0.0.1:8080",
            "127.0.0.1:12345",
            PacketDirection::Outgoing,
        );
        
        assert!(result.is_ok());
        let packet = result.expect("Test: operation should succeed");
        assert_eq!(packet.size, data.len());
        assert_eq!(packet.direction, PacketDirection::Outgoing);
        assert_eq!(packet.source_address, "127.0.0.1:8080");
        assert_eq!(packet.destination_address, "127.0.0.1:12345");
    }

    #[test]
    fn test_network_message_parsing() {
        let parser = NetworkMessageParser::new();
        let message = NetworkMessage::Heartbeat { timestamp: 12345 };
        let data = bincode_compat::serialize(&message).map_err(|e| Box::new(e)).expect("Test: operation should succeed");
        
        let result = parser.parse(&data);
        assert!(result.is_ok());
        
        let content = result.expect("Test: operation should succeed");
        assert_eq!(content.message_type, Some("Heartbeat".to_string()));
        assert!(content.fields.contains_key("timestamp"));
    }

    #[test]
    fn test_packet_search() {
        let mut analyzer = NetworkPacketAnalyzer::new();
        
        // 添加一些测试数据包
        let data1 = b"test packet 1";
        let data2 = b"test packet 2";
        
        analyzer.analyze_packet(data1, "127.0.0.1:8080", "127.0.0.1:12345", PacketDirection::Outgoing).ok();
        analyzer.analyze_packet(data2, "127.0.0.1:12345", "127.0.0.1:8080", PacketDirection::Incoming).ok();
        
        // 搜索所有出站数据包
        let query = PacketSearchQuery {
            start_time_ms: None,
            end_time_ms: None,
            protocols: None,
            packet_types: None,
            directions: Some(vec![PacketDirection::Outgoing]),
            min_size: None,
            max_size: None,
            source_address: None,
            destination_address: None,
            content_search: None,
            tags: None,
        };
        
        let results = analyzer.search_packets(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].direction, PacketDirection::Outgoing);
    }

    #[test]
    fn test_statistics_update() {
        let mut analyzer = NetworkPacketAnalyzer::new();
        
        let data = b"test packet data";
        analyzer.analyze_packet(data, "127.0.0.1:8080", "127.0.0.1:12345", PacketDirection::Outgoing).ok();
        
        let stats = analyzer.get_statistics();
        assert_eq!(stats.total_packets_analyzed, 1);
        assert_eq!(stats.average_packet_size, data.len() as f32);
        assert_eq!(stats.max_packet_size, data.len());
        assert_eq!(stats.min_packet_size, data.len());
    }
}