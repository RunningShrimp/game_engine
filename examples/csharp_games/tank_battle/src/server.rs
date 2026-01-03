//! Tank Battle Server - 专用服务器
//!
//! 多人在线坦克对战游戏专用服务器

use game_engine::prelude::*;
use game_engine::network::{NetworkServer, ServerConfig};
use log::{info, error, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Player {
    id: u32,
    address: SocketAddr,
    team: Team,
    score: i32,
    kills: i32,
    deaths: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Team {
    None,
    Red,
    Blue,
}

struct GameState {
    players: HashMap<u32, Player>,
    game_mode: GameMode,
    score_limit: i32,
    game_duration: Duration,
    start_time: Option<Instant>,
    game_in_progress: bool,
}

#[derive(Debug, Clone, Copy)]
enum GameMode {
    FreeForAll,
    TeamDeathmatch,
}

impl GameState {
    fn new() -> Self {
        Self {
            players: HashMap::new(),
            game_mode: GameMode::FreeForAll,
            score_limit: 50,
            game_duration: Duration::from_secs(600), // 10分钟
            start_time: None,
            game_in_progress: false,
        }
    }

    fn add_player(&mut self, id: u32, address: SocketAddr) {
        let player = Player {
            id,
            address,
            team: Team::None,
            score: 0,
            kills: 0,
            deaths: 0,
        };
        self.players.insert(id, player);
        info!("玩家加入: {} ({})", id, address);
    }

    fn remove_player(&mut self, id: u32) {
        if let Some(player) = self.players.remove(&id) {
            info!("玩家离开: {} ({})", id, player.address);
        }
    }

    fn get_player(&self, id: u32) -> Option<&Player> {
        self.players.get(&id)
    }

    fn get_player_mut(&mut self, id: u32) -> Option<&mut Player> {
        self.players.get_mut(&id)
    }

    fn start_game(&mut self) {
        if !self.game_in_progress {
            self.start_time = Some(Instant::now());
            self.game_in_progress = true;
            info!("游戏开始");
        }
    }

    fn end_game(&mut self, winner: Option<u32>) {
        if self.game_in_progress {
            self.game_in_progress = false;
            if let Some(winner_id) = winner {
                if let Some(winner) = self.players.get(&winner_id) {
                    info!("游戏结束，获胜者: {} (分数: {})", winner_id, winner.score);
                }
            } else {
                info!("游戏结束");
            }

            // 重置分数
            for player in self.players.values_mut() {
                player.score = 0;
                player.kills = 0;
                player.deaths = 0;
            }

            // 5秒后重新开始
            // （在实际实现中，应该使用定时器）
        }
    }

    fn on_player_killed(&mut self, victim: u32, killer: u32) {
        // 更新统计
        if let Some(killer_player) = self.get_player_mut(killer) {
            killer_player.kills += 1;
            killer_player.score += 1;

            info!("玩家 {} 击杀了玩家 {}", killer, victim);

            // 检查胜利条件
            if killer_player.score >= self.score_limit {
                self.end_game(Some(killer));
            }
        }

        if let Some(victim_player) = self.get_player_mut(victim) {
            victim_player.deaths += 1;
        }
    }

    fn get_elapsed_time(&self) -> Option<Duration> {
        self.start_time.map(|t| t.elapsed())
    }

    fn get_remaining_time(&self) -> Option<Duration> {
        self.start_time.map(|t| {
            self.game_duration
                .checked_sub(t.elapsed())
                .unwrap_or(Duration::ZERO)
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("启动坦克对战专用服务器");

    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    let port = if args.len() > 1 {
        args[1].parse::<u16>()?
    } else {
        27015
    };

    info!("监听端口: {}", port);

    // 创建服务器配置
    let server_config = ServerConfig {
        port,
        max_players: 8,
        timeout_seconds: 30,
        tick_rate: 60,
        ..Default::default()
    };

    // 创建网络服务器
    let mut server = NetworkServer::new(server_config)?;

    // 创建游戏状态
    let mut game_state = GameState::new();

    info!("服务器启动成功，等待玩家连接...");

    // 主服务器循环
    let mut last_tick = Instant::now();
    let tick_duration = Duration::from_secs_f64(1.0 / 60.0);

    loop {
        // 处理网络事件
        while let Some(event) = server.poll_event()? {
            match event {
                NetworkEvent::PlayerConnected { id, address } => {
                    game_state.add_player(id, address);

                    // 发送欢迎消息
                    server.send_message(id, ServerMessage::Welcome {
                        player_id: id,
                        map_name: "DefaultMap".to_string(),
                    })?;

                    // 发送当前玩家列表
                    let players: Vec<_> = game_state
                        .players
                        .values()
                        .map(|p| (p.id, p.score))
                        .collect();
                    server.broadcast_message(ServerMessage::PlayerList { players })?;

                    // 如果玩家数量足够，开始游戏
                    if game_state.players.len() >= 2 && !game_state.game_in_progress {
                        game_state.start_game();
                        server.broadcast_message(ServerMessage::GameStart)?;
                    }
                }

                NetworkEvent::PlayerDisconnected { id } => {
                    game_state.remove_player(id);

                    // 广播玩家离开
                    server.broadcast_message(ServerMessage::PlayerDisconnected { player_id: id })?;

                    // 如果玩家太少，结束游戏
                    if game_state.players.len() < 2 && game_state.game_in_progress {
                        game_state.end_game(None);
                        server.broadcast_message(ServerMessage::GameEnd)?;
                    }
                }

                NetworkEvent::MessageReceived { player_id, message } => {
                    handle_client_message(player_id, message, &mut game_state, &server)?;
                }

                NetworkEvent::Error { error } => {
                    warn!("网络错误: {}", error);
                }
            }
        }

        // 游戏逻辑更新
        if game_state.game_in_progress {
            // 检查时间限制
            if let Some(remaining) = game_state.get_remaining_time() {
                if remaining == Duration::ZERO {
                    // 找出最高分玩家
                    let winner = game_state
                        .players
                        .iter()
                        .max_by_key(|(_, p)| p.score)
                        .map(|(id, _)| *id);

                    game_state.end_game(winner);
                    server.broadcast_message(ServerMessage::GameEnd)?;
                }
            }

            // 定期同步游戏状态
            let elapsed = last_tick.elapsed();
            if elapsed >= Duration::from_secs(1) {
                sync_game_state(&game_state, &server)?;
                last_tick = Instant::now();
            }
        }

        // 控制帧率
        std::thread::sleep(tick_duration);
    }
}

fn handle_client_message(
    player_id: u32,
    message: ClientMessage,
    game_state: &mut GameState,
    server: &NetworkServer,
) -> Result<(), Box<dyn std::error::Error>> {
    match message {
        ClientMessage::PlayerUpdate { position, rotation } => {
            // 转发位置更新给其他玩家
            server.broadcast_message_except(
                ServerMessage::PlayerUpdate {
                    player_id,
                    position,
                    rotation,
                },
                player_id,
            )?;
        }

        ClientMessage::PlayerFire { origin, direction } => {
            // 转发射击事件
            server.broadcast_message(ServerMessage::PlayerFire {
                player_id,
                origin,
                direction,
            })?;
        }

        ClientMessage::PlayerHit { victim_id, damage } => {
            // 处理命中
            if game_state.game_in_progress {
                game_state.on_player_killed(victim_id, player_id);

                // 通知所有玩家
                server.broadcast_message(ServerMessage::PlayerKilled {
                    victim_id,
                    killer_id: player_id,
                    damage,
                })?;
            }
        }

        ClientMessage::ChatMessage { content } => {
            // 转发聊天消息
            if let Some(player) = game_state.get_player(player_id) {
                server.broadcast_message(ServerMessage::ChatMessage {
                    player_id,
                    player_name: format!("Player{}", player_id),
                    content,
                })?;
            }
        }

        ClientMessage::RequestRespawn => {
            // 处理重生请求
            server.send_message(
                player_id,
                ServerMessage::RespawnPlayer {
                    position: Vec3::ZERO, // 应该从生成点选择
                },
            )?;
        }

        _ => {
            warn!("未知消息类型来自玩家 {}", player_id);
        }
    }

    Ok(())
}

fn sync_game_state(
    game_state: &GameState,
    server: &NetworkServer,
) -> Result<(), Box<dyn std::error::Error>> {
    // 同步时间
    if let Some(remaining) = game_state.get_remaining_time() {
        server.broadcast_message(ServerMessage::TimeUpdate {
            remaining_seconds: remaining.as_secs(),
        })?;
    }

    // 同步分数
    let scores: Vec<_> = game_state
        .players
        .iter()
        .map(|(id, p)| (*id, p.score, p.kills, p.deaths))
        .collect();

    server.broadcast_message(ServerMessage::ScoreUpdate { scores })?;

    Ok(())
}

// 消息类型定义

#[derive(Debug, Clone)]
enum ClientMessage {
    PlayerUpdate { position: Vec3, rotation: Quaternion },
    PlayerFire { origin: Vec3, direction: Vec3 },
    PlayerHit { victim_id: u32, damage: i32 },
    ChatMessage { content: String },
    RequestRespawn,
}

#[derive(Debug, Clone)]
enum ServerMessage {
    Welcome { player_id: u32, map_name: String },
    PlayerList { players: Vec<(u32, i32)> },
    PlayerDisconnected { player_id: u32 },
    GameStart,
    GameEnd,
    PlayerUpdate { player_id: u32, position: Vec3, rotation: Quaternion },
    PlayerFire { player_id: u32, origin: Vec3, direction: Vec3 },
    PlayerKilled { victim_id: u32, killer_id: u32, damage: i32 },
    TimeUpdate { remaining_seconds: u64 },
    ScoreUpdate { scores: Vec<(u32, i32, i32, i32)> },
    ChatMessage { player_id: u32, player_name: String, content: String },
    RespawnPlayer { position: Vec3 },
}
