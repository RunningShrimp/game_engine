//! 2D Platformer Game Example
//!
//! A complete 2D platformer game demonstrating:
//! - Entity Component System (ECS)
//! - Physics integration
//! - Animation system
//! - Input handling
//! - Game state management
//! - Level design

use game_engine::prelude::*;

/// Game states for the 2D platformer
#[derive(Clone, Debug, PartialEq)]
enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
    Victory,
}

/// Player component
#[derive(Component)]
struct Player {
    speed: f32,
    jump_force: f32,
    is_grounded: bool,
    health: u32,
    coins_collected: u32,
}

/// Enemy component
#[derive(Component)]
struct Enemy {
    patrol_range: f32,
    speed: f32,
    direction: f32,
}

/// Coin component
#[derive(Component)]
struct Coin {
    value: u32,
    collected: bool,
}

/// Platform component
#[derive(Component)]
struct Platform {
    is_moving: bool,
    move_speed: f32,
    move_range: f32,
}

/// Level manager
struct LevelManager {
    current_level: u32,
    player_spawn: Vector3,
    total_coins: u32,
}

impl LevelManager {
    fn new() -> Self {
        Self {
            current_level: 1,
            player_spawn: Vector3::new(0.0, 2.0, 0.0),
            total_coins: 10,
        }
    }

    fn load_level(&mut self, level_id: u32, world: &mut World) {
        // Clear existing entities
        world.clear();

        // Create player
        let player = Entity::new("Player");
        player.add_component(Transform {
            position: self.player_spawn,
            rotation: Quaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        });
        player.add_component(Sprite::new("sprites/player.png"));
        player.add_component(RigidBody::dynamic());
        player.add_component(Collider::box_shape(Vector2::new(0.8, 1.8)));
        player.add_component(Player {
            speed: 5.0,
            jump_force: 8.0,
            is_grounded: false,
            health: 100,
            coins_collected: 0,
        });
        player.add_component(Animator::new("animations/player.anim"));
        world.add_entity(player);

        // Create ground
        for x in -10..=10 {
            let ground = Entity::new(&format!("Ground_{}", x));
            ground.add_component(Transform {
                position: Vector3::new(x as f32, -1.0, 0.0),
                rotation: Quaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
            });
            ground.add_component(Sprite::new("sprites/ground.png"));
            ground.add_component(RigidBody::static_body());
            ground.add_component(Collider::box_shape(Vector2::new(1.0, 1.0)));
            ground.add_component(Platform {
                is_moving: false,
                move_speed: 0.0,
                move_range: 0.0,
            });
            world.add_entity(ground);
        }

        // Create platforms
        let platform_positions = [
            Vector3::new(3.0, 1.0, 0.0),
            Vector3::new(6.0, 2.5, 0.0),
            Vector3::new(9.0, 4.0, 0.0),
            Vector3::new(3.0, 5.0, 0.0),
        ];

        for (i, pos) in platform_positions.iter().enumerate() {
            let platform = Entity::new(&format!("Platform_{}", i));
            platform.add_component(Transform {
                position: *pos,
                rotation: Quaternion::identity(),
                scale: Vector3::new(2.0, 0.3, 1.0),
            });
            platform.add_component(Sprite::new("sprites/platform.png"));
            platform.add_component(RigidBody::static_body());
            platform.add_component(Collider::box_shape(Vector2::new(2.0, 0.3)));
            platform.add_component(Platform {
                is_moving: i % 2 == 0,
                move_speed: 2.0,
                move_range: 3.0,
            });
            world.add_entity(platform);
        }

        // Create coins
        let coin_positions = [
            Vector3::new(3.0, 2.0, 0.0),
            Vector3::new(6.0, 3.5, 0.0),
            Vector3::new(9.0, 5.0, 0.0),
            Vector3::new(3.0, 6.0, 0.0),
            Vector3::new(0.0, 3.0, 0.0),
        ];

        for (i, pos) in coin_positions.iter().enumerate() {
            let coin = Entity::new(&format!("Coin_{}", i));
            coin.add_component(Transform {
                position: *pos,
                rotation: Quaternion::identity(),
                scale: Vector3::new(0.5, 0.5, 0.5),
            });
            coin.add_component(Sprite::new("sprites/coin.png"));
            coin.add_component(Collider::circle_shape(0.25));
            coin.add_component(Coin {
                value: 10,
                collected: false,
            });
            world.add_entity(coin);
        }

        // Create enemies
        let enemy_positions = [
            Vector3::new(5.0, 0.0, 0.0),
            Vector3::new(8.0, 1.5, 0.0),
        ];

        for (i, pos) in enemy_positions.iter().enumerate() {
            let enemy = Entity::new(&format!("Enemy_{}", i));
            enemy.add_component(Transform {
                position: *pos,
                rotation: Quaternion::identity(),
                scale: Vector3::new(0.8, 0.8, 1.0),
            });
            enemy.add_component(Sprite::new("sprites/enemy.png"));
            enemy.add_component(RigidBody::dynamic());
            enemy.add_component(Collider::box_shape(Vector2::new(0.8, 0.8)));
            enemy.add_component(Enemy {
                patrol_range: 3.0,
                speed: 2.0,
                direction: 1.0,
            });
            enemy.add_component(Animator::new("animations/enemy.anim"));
            world.add_entity(enemy);
        }

        // Create goal
        let goal = Entity::new("Goal");
        goal.add_component(Transform {
            position: Vector3::new(0.0, 7.0, 0.0),
            rotation: Quaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        });
        goal.add_component(Sprite::new("sprites/goal.png"));
        goal.add_component(Collider::box_shape(Vector2::new(1.0, 1.0)));
        goal.add_component(Trigger::new());
        world.add_entity(goal);

        // Create camera
        let camera = Entity::new("Camera");
        camera.add_component(Transform {
            position: Vector3::new(0.0, 2.0, 10.0),
            rotation: Quaternion::identity(),
            scale: Vector3::one(),
        });
        camera.add_component(Camera::perspective(60.0, 16.0 / 9.0, 0.1, 100.0));
        camera.add_component(CameraFollow::new(player));
        world.add_entity(camera);

        // Create UI
        let ui = Entity::new("UI");
        ui.add_component(UI {
            health: 100,
            coins: 0,
            level: level_id,
        });
        world.add_entity(ui);
    }
}

/// Main game struct
struct PlatformerGame {
    state: GameState,
    level_manager: LevelManager,
}

impl Game for PlatformerGame {
    fn new() -> Self {
        Self {
            state: GameState::Menu,
            level_manager: LevelManager::new(),
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        match self.state {
            GameState::Menu => self.update_menu(ctx),
            GameState::Playing => self.update_playing(ctx),
            GameState::Paused => self.update_paused(ctx),
            GameState::GameOver => self.update_game_over(ctx),
            GameState::Victory => self.update_victory(ctx),
        }
    }

    fn render(&mut self, ctx: &mut Context) {
        // Rendering is handled by the engine
    }
}

impl PlatformerGame {
    fn update_menu(&mut self, ctx: &mut Context) {
        let input = ctx.input();

        if input.is_key_just_pressed(KeyCode::Space) || input.is_key_just_pressed(KeyCode::Enter) {
            self.start_game(ctx);
        }

        // Render menu
        self.draw_menu(ctx);
    }

    fn update_playing(&mut self, ctx: &mut Context) {
        let input = ctx.input();
        let dt = ctx.delta_time();

        // Handle pause
        if input.is_key_just_pressed(KeyCode::Escape) {
            self.state = GameState::Paused;
            return;
        }

        // Update player
        self.update_player(ctx, dt);

        // Update enemies
        self.update_enemies(ctx, dt);

        // Update platforms
        self.update_platforms(ctx, dt);

        // Update coins
        self.update_coins(ctx);

        // Check win/lose conditions
        self.check_game_state(ctx);
    }

    fn update_paused(&mut self, ctx: &mut Context) {
        let input = ctx.input();

        if input.is_key_just_pressed(KeyCode::Escape) {
            self.state = GameState::Playing;
        }

        // Render pause menu
        self.draw_pause_menu(ctx);
    }

    fn update_game_over(&mut self, ctx: &mut Context) {
        let input = ctx.input();

        if input.is_key_just_pressed(KeyCode::Space) || input.is_key_just_pressed(KeyCode::R) {
            self.restart_game(ctx);
        }

        // Render game over screen
        self.draw_game_over(ctx);
    }

    fn update_victory(&mut self, ctx: &mut Context) {
        let input = ctx.input();

        if input.is_key_just_pressed(KeyCode::Space) {
            self.next_level(ctx);
        }

        // Render victory screen
        self.draw_victory(ctx);
    }

    fn start_game(&mut self, ctx: &mut Context) {
        self.state = GameState::Playing;
        self.level_manager.load_level(self.level_manager.current_level, ctx.world());
    }

    fn restart_game(&mut self, ctx: &mut Context) {
        self.level_manager.current_level = 1;
        self.start_game(ctx);
    }

    fn next_level(&mut self, ctx: &mut Context) {
        self.level_manager.current_level += 1;
        self.start_game(ctx);
    }

    fn update_player(&mut self, ctx: &mut Context, dt: f32) {
        let player = ctx.world().get_entity_with_component::<Player>().unwrap();
        let mut transform = player.get_component::<Transform>().unwrap();
        let mut player_data = player.get_component::<Player>().unwrap();
        let input = ctx.input();
        let rigid_body = player.get_component::<RigidBody>().unwrap();

        // Horizontal movement
        let mut move_direction = 0.0;
        if input.is_key_down(KeyCode::A) || input.is_key_down(KeyCode::Left) {
            move_direction = -1.0;
        } else if input.is_key_down(KeyCode::D) || input.is_key_down(KeyCode::Right) {
            move_direction = 1.0;
        }

        let velocity = rigid_body.velocity();
        velocity.x = move_direction * player_data.speed;
        rigid_body.set_velocity(velocity);

        // Jump
        if (input.is_key_just_pressed(KeyCode::Space) ||
            input.is_key_just_pressed(KeyCode::W) ||
            input.is_key_just_pressed(KeyCode::Up)) && player_data.is_grounded {
            rigid_body.apply_impulse(Vector3::new(0.0, player_data.jump_force, 0.0));
            player_data.is_grounded = false;

            // Play jump animation
            if let Some(animator) = player.get_component::<Animator>() {
                animator.play("jump");
            }
        }

        // Animation
        if move_direction.abs() > 0.1 {
            if let Some(animator) = player.get_component::<Animator>() {
                animator.play("run");
            }

            // Flip sprite based on direction
            if move_direction < 0.0 {
                transform.scale.x = -1.0;
            } else {
                transform.scale.x = 1.0;
            }
        } else if player_data.is_grounded {
            if let Some(animator) = player.get_component::<Animator>() {
                animator.play("idle");
            }
        }

        // Check if grounded
        player_data.is_grounded = self.check_grounded(player, ctx.world());

        // Check fall damage
        if transform.position.y < -10.0 {
            player_data.health = 0;
        }
    }

    fn update_enemies(&mut self, ctx: &mut Context, dt: f32) {
        let world = ctx.world();
        let enemies = world.get_entities_with_component::<Enemy>();

        for enemy in enemies {
            let mut enemy_data = enemy.get_component::<Enemy>().unwrap();
            let mut transform = enemy.get_component::<Transform>().unwrap();
            let rigid_body = enemy.get_component::<RigidBody>().unwrap();

            // Patrol behavior
            let start_x = transform.position.x - enemy_data.patrol_range / 2.0;
            let end_x = transform.position.x + enemy_data.patrol_range / 2.0;

            if transform.position.x <= start_x {
                enemy_data.direction = 1.0;
                transform.scale.x = 1.0;
            } else if transform.position.x >= end_x {
                enemy_data.direction = -1.0;
                transform.scale.x = -1.0;
            }

            let velocity = rigid_body.velocity();
            velocity.x = enemy_data.direction * enemy_data.speed;
            rigid_body.set_velocity(velocity);

            // Animation
            if let Some(animator) = enemy.get_component::<Animator>() {
                animator.play("walk");
            }
        }
    }

    fn update_platforms(&mut self, ctx: &mut Context, dt: f32) {
        let world = ctx.world();
        let platforms = world.get_entities_with_component::<Platform>();

        for platform in platforms {
            let mut platform_data = platform.get_component::<Platform>().unwrap();

            if !platform_data.is_moving {
                continue;
            }

            let mut transform = platform.get_component::<Transform>().unwrap();
            let time = ctx.time();

            // Oscillate platform
            transform.position.y += (platform_data.move_speed * dt).sin();

            // Clamp movement range
            let base_y = 1.0;
            transform.position.y = transform.position.y.clamp(
                base_y - platform_data.move_range,
                base_y + platform_data.move_range
            );
        }
    }

    fn update_coins(&mut self, ctx: &mut Context) {
        let world = ctx.world();
        let player = world.get_entity_with_component::<Player>().unwrap();
        let player_transform = player.get_component::<Transform>().unwrap();
        let coins = world.get_entities_with_component::<Coin>();

        for coin in coins {
            let mut coin_data = coin.get_component::<Coin>().unwrap();

            if coin_data.collected {
                continue;
            }

            let coin_transform = coin.get_component::<Transform>().unwrap();
            let distance = player_transform.position.distance(coin_transform.position);

            if distance < 0.5 {
                coin_data.collected = true;

                // Update player score
                if let Some(player_data) = player.get_component::<Player>() {
                    player_data.coins_collected += coin_data.value;
                }

                // Play collection animation
                if let Some(animator) = coin.get_component::<Animator>() {
                    animator.play("collect");
                }

                // Remove coin after animation
                coin.despawn();
            }
        }
    }

    fn check_grounded(&self, player: &Entity, world: &World) -> bool {
        let player_transform = player.get_component::<Transform>().unwrap();
        let player_collider = player.get_component::<Collider>().unwrap();
        let platforms = world.get_entities_with_component::<Platform>();

        for platform in platforms {
            let platform_transform = platform.get_component::<Transform>().unwrap();
            let platform_collider = platform.get_component::<Collider>().unwrap();

            if player_collider.overlaps(platform_collider, player_transform, platform_transform) {
                // Check if player is above the platform
                if player_transform.position.y > platform_transform.position.y + 0.1 {
                    return true;
                }
            }
        }

        false
    }

    fn check_game_state(&mut self, ctx: &mut Context) {
        // Check player death
        if let Some(player) = ctx.world().get_entity_with_component::<Player>() {
            if let Some(player_data) = player.get_component::<Player>() {
                if player_data.health == 0 {
                    self.state = GameState::GameOver;
                    return;
                }
            }
        }

        // Check victory (all coins collected)
        let coins = ctx.world().get_entities_with_component::<Coin>();
        let all_collected = coins.iter().all(|coin| {
            if let Some(coin_data) = coin.get_component::<Coin>() {
                coin_data.collected
            } else {
                true
            }
        });

        if all_collected {
            self.state = GameState::Victory;
        }
    }

    fn draw_menu(&self, ctx: &mut Context) {
        // Draw menu UI
        ctx.ui().text("2D Platformer")
            .position(Vector2::new(0.0, 0.2))
            .size(50.0)
            .color(Color::WHITE)
            .draw();

        ctx.ui().text("Press SPACE to Start")
            .position(Vector2::new(0.0, 0.0))
            .size(30.0)
            .color(Color::YELLOW)
            .draw();

        ctx.ui().text("Controls:")
            .position(Vector2::new(-0.3, -0.2))
            .size(20.0)
            .color(Color::WHITE)
            .draw();

        ctx.ui().text("Arrow Keys / WASD - Move")
            .position(Vector2::new(-0.3, -0.25))
            .size(16.0)
            .color(Color::GRAY)
            .draw();

        ctx.ui().text("Space / W / Up - Jump")
            .position(Vector2::new(-0.3, -0.28))
            .size(16.0)
            .color(Color::GRAY)
            .draw();

        ctx.ui().text("ESC - Pause")
            .position(Vector2::new(-0.3, -0.31))
            .size(16.0)
            .color(Color::GRAY)
            .draw();
    }

    fn draw_pause_menu(&self, ctx: &mut Context) {
        ctx.ui().overlay(Color::rgba(0, 0, 0, 0.5));

        ctx.ui().text("PAUSED")
            .position(Vector2::new(0.0, 0.1))
            .size(50.0)
            .color(Color::WHITE)
            .draw();

        ctx.ui().text("Press ESC to Resume")
            .position(Vector2::new(0.0, 0.0))
            .size(20.0)
            .color(Color::YELLOW)
            .draw();
    }

    fn draw_game_over(&self, ctx: &mut Context) {
        ctx.ui().overlay(Color::rgba(0.5, 0, 0, 0.5));

        ctx.ui().text("GAME OVER")
            .position(Vector2::new(0.0, 0.1))
            .size(50.0)
            .color(Color::RED)
            .draw();

        ctx.ui().text("Press SPACE to Restart")
            .position(Vector2::new(0.0, 0.0))
            .size(20.0)
            .color(Color::WHITE)
            .draw();
    }

    fn draw_victory(&self, ctx: &mut Context) {
        ctx.ui().overlay(Color::rgba(0, 0.5, 0, 0.5));

        ctx.ui().text("VICTORY!")
            .position(Vector2::new(0.0, 0.1))
            .size(50.0)
            .color(Color::GOLD)
            .draw();

        if let Some(player) = ctx.world().get_entity_with_component::<Player>() {
            if let Some(player_data) = player.get_component::<Player>() {
                ctx.ui().text(&format!("Coins: {}", player_data.coins_collected))
                    .position(Vector2::new(0.0, 0.0))
                    .size(30.0)
                    .color(Color::WHITE)
                    .draw();
            }
        }

        ctx.ui().text("Press SPACE for Next Level")
            .position(Vector2::new(0.0, -0.1))
            .size(20.0)
            .color(Color::YELLOW)
            .draw();
    }
}

/// Camera follow component
#[derive(Component)]
struct CameraFollow {
    target: Entity,
    smooth_speed: f32,
}

impl CameraFollow {
    fn new(target: Entity) -> Self {
        Self {
            target,
            smooth_speed: 5.0,
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        let target_transform = self.target.get_component::<Transform>().unwrap();
        let mut camera_transform = self.entity().get_component::<Transform>().unwrap();

        // Smooth follow
        let target_pos = target_transform.position;
        let current_pos = camera_transform.position;
        camera_transform.position = current_pos.lerp(target_pos, self.smooth_speed * ctx.delta_time());
    }
}

/// UI component
#[derive(Component)]
struct UI {
    health: u32,
    coins: u32,
    level: u32,
}

fn main() {
    // Initialize game engine
    let mut engine = GameEngine::new();

    // Configure engine
    engine
        .with_title("2D Platformer")
        .with_resolution(Vector2::new(1280, 720))
        .with_fps(60)
        .with_physics(true)
        .with_audio(true);

    // Create game
    let game = PlatformerGame::new();

    // Run game
    engine.run(game);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_initialization() {
        let game = PlatformerGame::new();
        assert_eq!(game.state, GameState::Menu);
    }

    #[test]
    fn test_level_manager() {
        let manager = LevelManager::new();
        assert_eq!(manager.current_level, 1);
        assert_eq!(manager.total_coins, 10);
    }

    #[test]
    fn test_player_component() {
        let player = Player {
            speed: 5.0,
            jump_force: 8.0,
            is_grounded: false,
            health: 100,
            coins_collected: 0,
        };

        assert_eq!(player.speed, 5.0);
        assert_eq!(player.health, 100);
    }

    #[test]
    fn test_enemy_component() {
        let enemy = Enemy {
            patrol_range: 3.0,
            speed: 2.0,
            direction: 1.0,
        };

        assert_eq!(enemy.patrol_range, 3.0);
        assert_eq!(enemy.speed, 2.0);
    }

    #[test]
    fn test_coin_component() {
        let coin = Coin {
            value: 10,
            collected: false,
        };

        assert_eq!(coin.value, 10);
        assert!(!coin.collected);
    }

    #[test]
    fn test_platform_component() {
        let platform = Platform {
            is_moving: true,
            move_speed: 2.0,
            move_range: 3.0,
        };

        assert!(platform.is_moving);
        assert_eq!(platform.move_speed, 2.0);
    }

    #[test]
    fn test_game_states() {
        assert_eq!(GameState::Menu, GameState::Menu);
        assert_eq!(GameState::Playing, GameState::Playing);
        assert_eq!(GameState::Paused, GameState::Paused);
        assert_eq!(GameState::GameOver, GameState::GameOver);
        assert_eq!(GameState::Victory, GameState::Victory);
    }

    #[test]
    fn test_camera_follow() {
        let target = Entity::new("Player");
        let follow = CameraFollow::new(target);
        assert_eq!(follow.target, target);
        assert_eq!(follow.smooth_speed, 5.0);
    }

    #[test]
    fn test_ui_component() {
        let ui = UI {
            health: 100,
            coins: 0,
            level: 1,
        };

        assert_eq!(ui.health, 100);
        assert_eq!(ui.coins, 0);
        assert_eq!(ui.level, 1);
    }

    #[test]
    fn test_level_loading() {
        let mut manager = LevelManager::new();
        let mut world = World::new();

        manager.load_level(1, &mut world);

        assert_eq!(manager.current_level, 1);
        // Verify entities were created
        assert!(world.get_entities_with_component::<Player>().len() > 0);
        assert!(world.get_entities_with_component::<Coin>().len() > 0);
        assert!(world.get_entities_with_component::<Enemy>().len() > 0);
        assert!(world.get_entities_with_component::<Platform>().len() > 0);
    }

    #[test]
    fn test_player_movement() {
        let player = Player {
            speed: 5.0,
            jump_force: 8.0,
            is_grounded: true,
            health: 100,
            coins_collected: 0,
        };

        assert!(player.is_grounded);
        assert_eq!(player.speed, 5.0);
    }

    #[test]
    fn test_coin_collection() {
        let mut coin = Coin {
            value: 10,
            collected: false,
        };

        assert!(!coin.collected);

        coin.collected = true;
        assert!(coin.collected);
    }

    #[test]
    fn test_enemy_patrol() {
        let mut enemy = Enemy {
            patrol_range: 3.0,
            speed: 2.0,
            direction: 1.0,
        };

        assert_eq!(enemy.direction, 1.0);

        enemy.direction = -1.0;
        assert_eq!(enemy.direction, -1.0);
    }

    #[test]
    fn test_platform_movement() {
        let platform = Platform {
            is_moving: true,
            move_speed: 2.0,
            move_range: 3.0,
        };

        assert!(platform.is_moving);
        assert_eq!(platform.move_speed, 2.0);
        assert_eq!(platform.move_range, 3.0);
    }

    #[test]
    fn test_game_flow() {
        let mut game = PlatformerGame::new();

        // Initial state
        assert_eq!(game.state, GameState::Menu);

        // Start game
        game.state = GameState::Playing;
        assert_eq!(game.state, GameState::Playing);

        // Pause game
        game.state = GameState::Paused;
        assert_eq!(game.state, GameState::Paused);

        // Resume game
        game.state = GameState::Playing;
        assert_eq!(game.state, GameState::Playing);

        // Victory
        game.state = GameState::Victory;
        assert_eq!(game.state, GameState::Victory);
    }
}
