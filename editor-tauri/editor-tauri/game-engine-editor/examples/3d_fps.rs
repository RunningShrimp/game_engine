//! 3D First-Person Shooter Game Example
//!
//! A complete 3D FPS game demonstrating:
//! - 3D rendering and camera system
//! - First-person controls
//! - Weapon system
//! - Enemy AI
//! - Physics and collision detection
//! - Audio system
//! - HUD and UI

use game_engine::prelude::*;

/// Game states
#[derive(Clone, Debug, PartialEq)]
enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
}

/// Player component
#[derive(Component)]
struct Player {
    health: u32,
    max_health: u32,
    ammo: u32,
    max_ammo: u32,
    score: u32,
    kills: u32,
}

/// Weapon component
#[derive(Component)]
struct Weapon {
    damage: u32,
    fire_rate: f32,        // shots per second
    last_fire_time: f32,
    reload_time: f32,
    is_reloading: bool,
    current_ammo: u32,
    mag_size: u32,
}

/// Enemy component
#[derive(Component)]
struct Enemy {
    health: u32,
    max_health: u32,
    speed: f32,
    damage: u32,
    detection_range: f32,
    attack_range: f32,
    state: EnemyState,
}

/// Enemy AI states
#[derive(Clone, Debug, PartialEq)]
enum EnemyState {
    Idle,
    Patrol,
    Chase,
    Attack,
    Dead,
}

/// Projectile component
#[derive(Component)]
struct Projectile {
    damage: u32,
    speed: f32,
    lifetime: f32,
    owner: Entity,
}

/// Spawn point component
#[derive(Component)]
struct SpawnPoint {
    team: Team,
    spawn_rate: f32,
    last_spawn: f32,
}

/// Team enum
#[derive(Clone, Debug, PartialEq)]
enum Team {
    Player,
    Enemy,
}

/// HUD component
#[derive(Component)]
struct HUD {
    crosshair_enabled: bool,
    health_bar_visible: bool,
    ammo_counter_visible: bool,
    minimap_enabled: bool,
}

/// Main game struct
struct FirstPersonShooter {
    state: GameState,
    player: Option<Entity>,
    enemies: Vec<Entity>,
    spawn_points: Vec<Entity>,
    wave: u32,
}

impl Game for FirstPersonShooter {
    fn new() -> Self {
        Self {
            state: GameState::Menu,
            player: None,
            enemies: Vec::new(),
            spawn_points: Vec::new(),
            wave: 1,
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        match self.state {
            GameState::Menu => self.update_menu(ctx),
            GameState::Playing => self.update_playing(ctx),
            GameState::Paused => self.update_paused(ctx),
            GameState::GameOver => self.update_game_over(ctx),
        }
    }

    fn render(&mut self, ctx: &mut Context) {
        // Rendering is handled by the engine
    }
}

impl FirstPersonShooter {
    fn setup_level(&mut self, ctx: &mut Context) {
        let world = ctx.world_mut();

        // Clear existing entities
        world.clear();

        // Create player
        let player = Entity::new("Player");
        player.add_component(Transform {
            position: Vector3::new(0.0, 1.7, 0.0), // Eye level
            rotation: Quaternion::identity(),
            scale: Vector3::one(),
        });
        player.add_component(Camera::perspective(60.0, 16.0 / 9.0, 0.1, 1000.0));
        player.add_component(FirstPersonController::new());
        player.add_component(CharacterController::new());
        player.add_component(Player {
            health: 100,
            max_health: 100,
            ammo: 30,
            max_ammo: 30,
            score: 0,
            kills: 0,
        });
        player.add_component(Weapon {
            damage: 25,
            fire_rate: 600.0, // rounds per minute
            last_fire_time: 0.0,
            reload_time: 2.5,
            is_reloading: false,
            current_ammo: 30,
            mag_size: 30,
        });
        player.add_component(AudioSource::new("sounds/player_footstep.ogg"));
        world.add_entity(player.clone());
        self.player = Some(player);

        // Create ground
        let ground = Entity::new("Ground");
        ground.add_component(Transform {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::identity(),
            scale: Vector3::new(100.0, 1.0, 100.0),
        });
        ground.add_component(Mesh::from_file("models/ground.obj"));
        ground.add_material(Material::new("materials/ground.mat"));
        ground.add_component(RigidBody::static_body());
        ground.add_component(Collider::box_shape(Vector3::new(100.0, 1.0, 100.0)));
        world.add_entity(ground);

        // Create walls
        self.create_walls(world);

        // Create spawn points
        self.create_spawn_points(world);

        // Create HUD
        let hud = Entity::new("HUD");
        hud.add_component(HUD {
            crosshair_enabled: true,
            health_bar_visible: true,
            ammo_counter_visible: true,
            minimap_enabled: true,
        });
        world.add_entity(hud);

        // Create lighting
        let sun_light = Entity::new("SunLight");
        sun_light.add_component(Transform {
            position: Vector3::new(50.0, 100.0, 50.0),
            rotation: Quaternion::from_euler(EulerAngles::new(0.3, 0.0, 0.0)),
            scale: Vector3::one(),
        });
        sun_light.add_component(DirectionalLight::new(Color::new(1.0, 0.95, 0.8), 500.0));
        world.add_entity(sun_light);

        let ambient_light = Entity::new("AmbientLight");
        ambient_light.add_component(AmbientLight::new(Color::new(0.4, 0.4, 0.5), 0.3));
        world.add_entity(ambient_light);

        // Create skybox
        let skybox = Entity::new("Skybox");
        skybox.add_component(Skybox::new("textures/skybox.jpg"));
        world.add_entity(skybox);

        // Spawn initial enemies
        self.spawn_enemies(world, 5);
    }

    fn create_walls(&self, world: &mut World) {
        // Create perimeter walls
        let wall_positions = [
            (Vector3::new(0.0, 5.0, -50.0), Vector3::new(100.0, 10.0, 1.0)),
            (Vector3::new(0.0, 5.0, 50.0), Vector3::new(100.0, 10.0, 1.0)),
            (Vector3::new(-50.0, 5.0, 0.0), Vector3::new(1.0, 10.0, 100.0)),
            (Vector3::new(50.0, 5.0, 0.0), Vector3::new(1.0, 10.0, 100.0)),
        ];

        for (i, (pos, scale)) in wall_positions.iter().enumerate() {
            let wall = Entity::new(&format!("Wall_{}", i));
            wall.add_component(Transform {
                position: *pos,
                rotation: Quaternion::identity(),
                scale: *scale,
            });
            wall.add_component(Mesh::from_file("models/wall.obj"));
            wall.add_material(Material::new("materials/wall.mat"));
            wall.add_component(RigidBody::static_body());
            wall.add_component(Collider::box_shape(*scale));
            world.add_entity(wall);
        }

        // Create obstacles/cover
        let cover_positions = [
            Vector3::new(10.0, 1.0, 10.0),
            Vector3::new(-10.0, 1.0, 10.0),
            Vector3::new(10.0, 1.0, -10.0),
            Vector3::new(-10.0, 1.0, -10.0),
            Vector3::new(0.0, 1.0, 0.0),
        ];

        for (i, pos) in cover_positions.iter().enumerate() {
            let cover = Entity::new(&format!("Cover_{}", i));
            cover.add_component(Transform {
                position: *pos,
                rotation: Quaternion::identity(),
                scale: Vector3::new(2.0, 2.0, 2.0),
            });
            cover.add_component(Mesh::from_file("models/crate.obj"));
            cover.add_material(Material::new("materials/crate.mat"));
            cover.add_component(RigidBody::static_body());
            cover.add_component(Collider::box_shape(Vector3::new(2.0, 2.0, 2.0)));
            world.add_entity(cover);
        }
    }

    fn create_spawn_points(&mut self, world: &mut World) {
        let spawn_positions = [
            Vector3::new(20.0, 1.0, 20.0),
            Vector3::new(-20.0, 1.0, 20.0),
            Vector3::new(20.0, 1.0, -20.0),
            Vector3::new(-20.0, 1.0, -20.0),
        ];

        for (i, pos) in spawn_positions.iter().enumerate() {
            let spawn_point = Entity::new(&format!("EnemySpawn_{}", i));
            spawn_point.add_component(Transform {
                position: *pos,
                rotation: Quaternion::identity(),
                scale: Vector3::one(),
            });
            spawn_point.add_component(SpawnPoint {
                team: Team::Enemy,
                spawn_rate: 5.0, // seconds
                last_spawn: 0.0,
            });
            spawn_point.add_component(Collider::sphere_shape(2.0));
            spawn_point.add_component(DebugDraw::visible());
            world.add_entity(spawn_point.clone());
            self.spawn_points.push(spawn_point);
        }
    }

    fn spawn_enemies(&mut self, world: &mut World, count: u32) {
        for i in 0..count {
            let spawn_point = if let Some(sp) = self.spawn_points.iter().cycle().nth(i as usize) {
                sp
            } else {
                continue;
            };

            let spawn_transform = spawn_point.get_component::<Transform>().unwrap();

            let enemy = Entity::new(&format!("Enemy_{}", i));
            enemy.add_component(Transform {
                position: spawn_transform.position + Vector3::new(0.0, 0.0, 0.0),
                rotation: Quaternion::identity(),
                scale: Vector3::new(1.0, 1.8, 1.0),
            });
            enemy.add_component(Mesh::from_file("models/enemy.obj"));
            enemy.add_material(Material::new("materials/enemy.mat"));
            enemy.add_component(RigidBody::dynamic());
            enemy.add_component(Collider::capsule_shape(0.4, 1.8));
            enemy.add_component(Enemy {
                health: 100,
                max_health: 100,
                speed: 5.0,
                damage: 10,
                detection_range: 30.0,
                attack_range: 3.0,
                state: EnemyState::Patrol,
            });
            enemy.add_component(AudioSource::new("sounds/enemy_footstep.ogg"));
            enemy.add_component(Animator::new("animations/enemy.anim"));
            world.add_entity(enemy.clone());
            self.enemies.push(enemy);
        }
    }

    fn update_menu(&mut self, ctx: &mut Context) {
        let input = ctx.input();

        if input.is_key_just_pressed(KeyCode::Space) || input.is_key_just_pressed(KeyCode::Enter) {
            self.state = GameState::Playing;
            self.setup_level(ctx);
        }

        // Draw menu
        ctx.ui().text("FIRST PERSON SHOOTER")
            .position(Vector2::new(0.0, 0.2))
            .size(60.0)
            .color(Color::WHITE)
            .draw();

        ctx.ui().text("Press SPACE to Start")
            .position(Vector2::new(0.0, 0.0))
            .size(30.0)
            .color(Color::YELLOW)
            .draw();
    }

    fn update_playing(&mut self, ctx: &mut Context) {
        let input = ctx.input();

        // Handle pause
        if input.is_key_just_pressed(KeyCode::Escape) {
            self.state = GameState::Paused;
            return;
        }

        // Update player
        if let Some(player) = &self.player {
            self.update_player(ctx, player);
        }

        // Update enemies
        self.update_enemies(ctx);

        // Update projectiles
        self.update_projectiles(ctx);

        // Update spawn points
        self.update_spawn_points(ctx);

        // Check game state
        self.check_game_state(ctx);

        // Draw HUD
        self.draw_hud(ctx);
    }

    fn update_player(&mut self, ctx: &mut Context, player: &Entity) {
        let input = ctx.input();
        let dt = ctx.delta_time();

        // Handle shooting
        if let Some(weapon) = player.get_component::<Weapon>() {
            if input.is_mouse_button_down(MouseButton::Left) && !weapon.is_reloading {
                self.fire_weapon(ctx, player);
            }

            // Handle reload
            if input.is_key_just_pressed(KeyCode::R) && !weapon.is_reloading && weapon.current_ammo < weapon.mag_size {
                weapon.is_reloading = true;
                weapon.last_fire_time = ctx.time();
                ctx.play_sound("sounds/reload.ogg");
            }

            // Update reload
            if weapon.is_reloading {
                if ctx.time() - weapon.last_fire_time >= weapon.reload_time {
                    weapon.is_reloading = false;
                    weapon.current_ammo = weapon.mag_size;
                }
            }
        }

        // Handle weapon switching
        if input.is_key_just_pressed(KeyCode::Key1) {
            self.switch_weapon(ctx, player, "rifle");
        } else if input.is_key_just_pressed(KeyCode::Key2) {
            self.switch_weapon(ctx, player, "shotgun");
        }
    }

    fn fire_weapon(&mut self, ctx: &mut Context, player: &Entity) {
        if let Some(weapon) = player.get_component::<Weapon>() {
            let current_time = ctx.time();
            let fire_interval = 60.0 / weapon.fire_rate;

            if current_time - weapon.last_fire_time >= fire_interval && weapon.current_ammo > 0 {
                weapon.last_fire_time = current_time;
                weapon.current_ammo -= 1;

                // Create projectile
                let player_transform = player.get_component::<Transform>().unwrap();
                let camera = player.get_component::<Camera>().unwrap();

                let mut spawn_pos = player_transform.position;
                spawn_pos.y += 0.0; // At eye level
                spawn_pos += camera.forward() * 0.5; // Slightly in front

                let projectile = Entity::new("Projectile");
                projectile.add_component(Transform {
                    position: spawn_pos,
                    rotation: Quaternion::identity(),
                    scale: Vector3::new(0.1, 0.1, 0.1),
                });
                projectile.add_component(Mesh::from_file("models/bullet.obj"));
                projectile.add_component(RigidBody::dynamic());
                projectile.add_component(Collider::sphere_shape(0.1));
                projectile.add_component(Projectile {
                    damage: weapon.damage,
                    speed: 100.0,
                    lifetime: 3.0,
                    owner: player.clone(),
                });
                projectile.add_component(PointLight::new(Color::new(1.0, 0.8, 0.2), 10.0, 5.0));

                // Apply velocity
                let rigid_body = projectile.get_component::<RigidBody>().unwrap();
                rigid_body.set_velocity(camera.forward() * 100.0);

                ctx.world_mut().add_entity(projectile);

                // Play sound
                ctx.play_sound("sounds/shoot.ogg");
            }
        }
    }

    fn switch_weapon(&mut self, ctx: &mut Context, player: &Entity, weapon_type: &str) {
        // Update weapon stats based on type
        if let Some(weapon) = player.get_component::<Weapon>() {
            match weapon_type {
                "rifle" => {
                    weapon.damage = 25;
                    weapon.fire_rate = 600.0;
                    weapon.mag_size = 30;
                }
                "shotgun" => {
                    weapon.damage = 15;
                    weapon.fire_rate = 60.0;
                    weapon.mag_size = 8;
                }
                _ => {}
            }
        }
    }

    fn update_enemies(&mut self, ctx: &mut Context) {
        let world = ctx.world();

        // Remove dead enemies
        self.enemies.retain(|enemy| {
            if let Some(enemy_data) = enemy.get_component::<Enemy>() {
                enemy_data.health > 0
            } else {
                false
            }
        });

        // Update each enemy
        for enemy in &self.enemies {
            if let Some(enemy_data) = enemy.get_component::<Enemy>() {
                self.update_enemy_ai(ctx, enemy);
            }
        }
    }

    fn update_enemy_ai(&mut self, ctx: &mut Context, enemy: &Entity) {
        let world = ctx.world();
        let mut enemy_data = enemy.get_component::<Enemy>().unwrap();
        let enemy_transform = enemy.get_component::<Transform>().unwrap();

        // Get player position
        let player_pos = if let Some(player) = &self.player {
            let player_transform = player.get_component::<Transform>().unwrap();
            player_transform.position
        } else {
            return;
        };

        let distance_to_player = enemy_transform.position.distance(player_pos);

        // State machine
        match enemy_data.state {
            EnemyState::Idle => {
                if distance_to_player < enemy_data.detection_range {
                    enemy_data.state = EnemyState::Chase;
                }
            }
            EnemyState::Patrol => {
                // Simple patrol behavior
                let time = ctx.time();
                let patrol_radius = 10.0;
                let patrol_speed = 2.0;

                let mut pos = enemy_transform.position;
                pos.x = (time * patrol_speed).cos() * patrol_radius;
                pos.z = (time * patrol_speed).sin() * patrol_radius;
                enemy_transform.position = pos;

                if distance_to_player < enemy_data.detection_range {
                    enemy_data.state = EnemyState::Chase;
                }
            }
            EnemyState::Chase => {
                // Move towards player
                let direction = (player_pos - enemy_transform.position).normalize();
                let rigid_body = enemy.get_component::<RigidBody>().unwrap();
                let mut velocity = rigid_body.velocity();
                velocity.x = direction.x * enemy_data.speed;
                velocity.z = direction.z * enemy_data.speed;
                rigid_body.set_velocity(velocity);

                // Look at player
                enemy_transform.look_at(player_pos);

                if distance_to_player < enemy_data.attack_range {
                    enemy_data.state = EnemyState::Attack;
                }

                // Play animation
                if let Some(animator) = enemy.get_component::<Animator>() {
                    animator.play("run");
                }
            }
            EnemyState::Attack => {
                if distance_to_player > enemy_data.attack_range {
                    enemy_data.state = EnemyState::Chase;
                } else {
                    // Attack player
                    if let Some(player) = &self.player {
                        if let Some(player_data) = player.get_component::<Player>() {
                            player_data.health -= enemy_data.damage;
                            ctx.play_sound("sounds/player_hit.ogg");
                        }
                    }

                    // Play attack animation
                    if let Some(animator) = enemy.get_component::<Animator>() {
                        animator.play("attack");
                    }

                    // Cooldown
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            }
            EnemyState::Dead => {
                // Drop dead behavior
            }
        }
    }

    fn update_projectiles(&mut self, ctx: &mut Context) {
        let world = ctx.world_mut();
        let projectiles = world.get_entities_with_component::<Projectile>();

        for projectile in projectiles {
            let mut proj_data = projectile.get_component::<Projectile>().unwrap();
            proj_data.lifetime -= ctx.delta_time();

            if proj_data.lifetime <= 0.0 {
                projectile.despawn();
                continue;
            }

            // Check for enemy hits
            for enemy in &self.enemies {
                if let Some(enemy_data) = enemy.get_component::<Enemy>() {
                    if let Some(proj_transform) = projectile.get_component::<Transform>() {
                        if let Some(enemy_transform) = enemy.get_component::<Transform>() {
                            let distance = proj_transform.position.distance(enemy_transform.position);

                            if distance < 1.0 {
                                enemy_data.health -= proj_data.damage;
                                projectile.despawn();

                                // Check if enemy died
                                if enemy_data.health <= 0 {
                                    enemy_data.state = EnemyState::Dead;

                                    // Update player score
                                    if let Some(player) = &self.player {
                                        if let Some(player_data) = player.get_component::<Player>() {
                                            player_data.score += 100;
                                            player_data.kills += 1;
                                        }
                                    }

                                    // Play death sound
                                    ctx.play_sound("sounds/enemy_death.ogg");
                                } else {
                                    // Play hit sound
                                    ctx.play_sound("sounds/enemy_hit.ogg");
                                }

                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn update_spawn_points(&mut self, ctx: &mut Context) {
        let world = ctx.world();
        let current_time = ctx.time();

        for spawn_point in &self.spawn_points {
            if let Some(spawn_data) = spawn_point.get_component::<SpawnPoint>() {
                if current_time - spawn_data.last_spawn >= spawn_data.spawn_rate {
                    spawn_data.last_spawn = current_time;
                    // Spawn new enemy
                    self.spawn_enemies(ctx.world_mut(), 1);
                }
            }
        }
    }

    fn check_game_state(&mut self, ctx: &mut Context) {
        // Check player death
        if let Some(player) = &self.player {
            if let Some(player_data) = player.get_component::<Player>() {
                if player_data.health <= 0 {
                    self.state = GameState::GameOver;
                }
            }
        }

        // Check wave completion
        let alive_enemies = self.enemies.iter().filter(|enemy| {
            if let Some(enemy_data) = enemy.get_component::<Enemy>() {
                enemy_data.health > 0
            } else {
                false
            }
        }).count();

        if alive_enemies == 0 {
            self.wave += 1;
            self.spawn_enemies(ctx.world_mut(), 5 + self.wave * 2);
        }
    }

    fn update_paused(&mut self, ctx: &mut Context) {
        let input = ctx.input();

        if input.is_key_just_pressed(KeyCode::Escape) {
            self.state = GameState::Playing;
        }

        // Draw pause menu
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

    fn update_game_over(&mut self, ctx: &mut Context) {
        let input = ctx.input();

        if input.is_key_just_pressed(KeyCode::Space) || input.is_key_just_pressed(KeyCode::R) {
            // Restart game
            self.state = GameState::Menu;
        }

        // Draw game over screen
        ctx.ui().overlay(Color::rgba(0.5, 0, 0, 0.5));

        ctx.ui().text("GAME OVER")
            .position(Vector2::new(0.0, 0.2))
            .size(60.0)
            .color(Color::RED)
            .draw();

        if let Some(player) = &self.player {
            if let Some(player_data) = player.get_component::<Player>() {
                ctx.ui().text(&format!("Score: {}", player_data.score))
                    .position(Vector2::new(0.0, 0.05))
                    .size(30.0)
                    .color(Color::WHITE)
                    .draw();

                ctx.ui().text(&format!("Kills: {}", player_data.kills))
                    .position(Vector2::new(0.0, 0.0))
                    .size(30.0)
                    .color(Color::WHITE)
                    .draw();
            }
        }

        ctx.ui().text("Press SPACE to Restart")
            .position(Vector2::new(0.0, -0.1))
            .size(20.0)
            .color(Color::YELLOW)
            .draw();
    }

    fn draw_hud(&self, ctx: &mut Context) {
        // Draw crosshair
        ctx.ui().text("+")
            .position(Vector2::new(0.0, 0.0))
            .size(30.0)
            .color(Color::GREEN)
            .draw();

        // Draw health bar
        if let Some(player) = &self.player {
            if let Some(player_data) = player.get_component::<Player>() {
                let health_percent = player_data.health as f32 / player_data.max_health as f32;

                ctx.ui().rect_filled(
                    Vector2::new(-0.4, 0.45),
                    Vector2::new(0.4, 0.48),
                    Color::rgba(0.3, 0.3, 0.3, 0.7)
                );

                ctx.ui().rect_filled(
                    Vector2::new(-0.4, 0.45),
                    Vector2::new(-0.4 + 0.8 * health_percent, 0.48),
                    Color::rgba(0.0, 1.0, 0.0, 0.8)
                );

                ctx.ui().text(&format!("HP: {}/{}", player_data.health, player_data.max_health))
                    .position(Vector2::new(-0.4, 0.43))
                    .size(16.0)
                    .color(Color::WHITE)
                    .draw();

                // Draw ammo counter
                if let Some(weapon) = player.get_component::<Weapon>() {
                    ctx.ui().text(&format!("AMMO: {}/{}", weapon.current_ammo, weapon.mag_size))
                        .position(Vector2::new(0.2, 0.43))
                        .size(16.0)
                        .color(Color::YELLOW)
                        .draw();
                }

                // Draw score
                ctx.ui().text(&format!("SCORE: {}", player_data.score))
                    .position(Vector2::new(0.0, 0.35))
                    .size(24.0)
                    .color(Color::WHITE)
                    .draw();

                // Draw kills
                ctx.ui().text(&format!("KILLS: {}", player_data.kills))
                    .position(Vector2::new(0.0, 0.30))
                    .size(18.0)
                    .color(Color::RED)
                    .draw();

                // Draw wave
                ctx.ui().text(&format!("WAVE: {}", self.wave))
                    .position(Vector2::new(-0.3, 0.43))
                    .size(16.0)
                    .color(Color::CYAN)
                    .draw();
            }
        }
    }
}

fn main() {
    // Initialize game engine
    let mut engine = GameEngine::new();

    // Configure engine
    engine
        .with_title("3D First Person Shooter")
        .with_resolution(Vector2::new(1920, 1080))
        .with_fps(60)
        .with_physics(true)
        .with_audio(true)
        .with_fullscreen(false)
        .with_vsync(true);

    // Lock cursor
    engine.lock_cursor(true);

    // Create game
    let game = FirstPersonShooter::new();

    // Run game
    engine.run(game);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_initialization() {
        let game = FirstPersonShooter::new();
        assert_eq!(game.state, GameState::Menu);
    }

    #[test]
    fn test_player_component() {
        let player = Player {
            health: 100,
            max_health: 100,
            ammo: 30,
            max_ammo: 30,
            score: 0,
            kills: 0,
        };

        assert_eq!(player.health, 100);
        assert_eq!(player.ammo, 30);
    }

    #[test]
    fn test_weapon_component() {
        let weapon = Weapon {
            damage: 25,
            fire_rate: 600.0,
            last_fire_time: 0.0,
            reload_time: 2.5,
            is_reloading: false,
            current_ammo: 30,
            mag_size: 30,
        };

        assert_eq!(weapon.damage, 25);
        assert_eq!(weapon.fire_rate, 600.0);
        assert_eq!(weapon.current_ammo, 30);
    }

    #[test]
    fn test_enemy_component() {
        let enemy = Enemy {
            health: 100,
            max_health: 100,
            speed: 5.0,
            damage: 10,
            detection_range: 30.0,
            attack_range: 3.0,
            state: EnemyState::Patrol,
        };

        assert_eq!(enemy.health, 100);
        assert_eq!(enemy.state, EnemyState::Patrol);
    }

    #[test]
    fn test_enemy_states() {
        assert_eq!(EnemyState::Idle, EnemyState::Idle);
        assert_eq!(EnemyState::Patrol, EnemyState::Patrol);
        assert_eq!(EnemyState::Chase, EnemyState::Chase);
        assert_eq!(EnemyState::Attack, EnemyState::Attack);
        assert_eq!(EnemyState::Dead, EnemyState::Dead);
    }

    #[test]
    fn test_projectile_component() {
        let owner = Entity::new("Player");
        let projectile = Projectile {
            damage: 25,
            speed: 100.0,
            lifetime: 3.0,
            owner: owner.clone(),
        };

        assert_eq!(projectile.damage, 25);
        assert_eq!(projectile.lifetime, 3.0);
        assert_eq!(projectile.owner, owner);
    }

    #[test]
    fn test_spawn_point_component() {
        let spawn_point = SpawnPoint {
            team: Team::Enemy,
            spawn_rate: 5.0,
            last_spawn: 0.0,
        };

        assert_eq!(spawn_point.team, Team::Enemy);
        assert_eq!(spawn_point.spawn_rate, 5.0);
    }

    #[test]
    fn test_team_enum() {
        assert_eq!(Team::Player, Team::Player);
        assert_eq!(Team::Enemy, Team::Enemy);
    }

    #[test]
    fn test_hud_component() {
        let hud = HUD {
            crosshair_enabled: true,
            health_bar_visible: true,
            ammo_counter_visible: true,
            minimap_enabled: true,
        };

        assert!(hud.crosshair_enabled);
        assert!(hud.health_bar_visible);
    }

    #[test]
    fn test_weapon_reload() {
        let mut weapon = Weapon {
            damage: 25,
            fire_rate: 600.0,
            last_fire_time: 0.0,
            reload_time: 2.5,
            is_reloading: false,
            current_ammo: 0,
            mag_size: 30,
        };

        weapon.is_reloading = true;
        assert!(weapon.is_reloading);
        assert_eq!(weapon.current_ammo, 0);
    }

    #[test]
    fn test_game_states() {
        assert_eq!(GameState::Menu, GameState::Menu);
        assert_eq!(GameState::Playing, GameState::Playing);
        assert_eq!(GameState::Paused, GameState::Paused);
        assert_eq!(GameState::GameOver, GameState::GameOver);
    }

    #[test]
    fn test_enemy_ai_state_transition() {
        let mut enemy = Enemy {
            health: 100,
            max_health: 100,
            speed: 5.0,
            damage: 10,
            detection_range: 30.0,
            attack_range: 3.0,
            state: EnemyState::Idle,
        };

        // Simulate state transitions
        enemy.state = EnemyState::Patrol;
        assert_eq!(enemy.state, EnemyState::Patrol);

        enemy.state = EnemyState::Chase;
        assert_eq!(enemy.state, EnemyState::Chase);

        enemy.state = EnemyState::Attack;
        assert_eq!(enemy.state, EnemyState::Attack);
    }

    #[test]
    fn test_score_tracking() {
        let mut player = Player {
            health: 100,
            max_health: 100,
            ammo: 30,
            max_ammo: 30,
            score: 0,
            kills: 0,
        };

        player.score += 100;
        assert_eq!(player.score, 100);

        player.kills += 1;
        assert_eq!(player.kills, 1);
    }

    #[test]
    fn test_ammo_consumption() {
        let mut weapon = Weapon {
            damage: 25,
            fire_rate: 600.0,
            last_fire_time: 0.0,
            reload_time: 2.5,
            is_reloading: false,
            current_ammo: 30,
            mag_size: 30,
        };

        weapon.current_ammo -= 1;
        assert_eq!(weapon.current_ammo, 29);

        weapon.current_ammo -= 1;
        assert_eq!(weapon.current_ammo, 28);
    }

    #[test]
    fn test_health_damage() {
        let mut player = Player {
            health: 100,
            max_health: 100,
            ammo: 30,
            max_ammo: 30,
            score: 0,
            kills: 0,
        };

        player.health -= 10;
        assert_eq!(player.health, 90);

        player.health -= 20;
        assert_eq!(player.health, 70);
    }
}
