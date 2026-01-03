# C# SDK Documentation

**Version**: 0.1.0
**Engine**: Game Engine
**Target**: .NET 8.0+
**Date**: 2026-01-03

---

## Table of Contents

1. [Overview](#overview)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Core Concepts](#core-concepts)
5. [API Reference](#api-reference)
6. [Components](#components)
7. [Systems](#systems)
8. [Networking](#networking)
9. [Best Practices](#best-practices)
10. [Examples](#examples)
11. [Troubleshooting](#troubleshooting)
12. [Performance Tips](#performance-tips)

---

## Overview

The C# SDK provides a powerful scripting system for the game engine, enabling developers to:

- Write game logic in C#
- Hot-reload scripts during development
- Access engine APIs through a familiar Unity-like interface
- Build cross-platform games with .NET 8.0+

### Key Features

- ✅ **Unity-Compatible API**: Familiar component-based architecture
- ✅ **Hot Reload**: Modify scripts without restarting the game
- ✅ **Type Safety**: Full C# type checking and IntelliSense
- ✅ **Performance**: Native Rust engine with C# scripting layer
- ✅ **Cross-Platform**: Works on Windows, macOS, Linux
- ✅ **Networking**: Built-in multiplayer support with Mirror-like API

### Supported .NET Versions

- .NET 8.0 SDK (recommended)
- .NET 9.0 SDK (latest)
- .NET 7.0 SDK (legacy)

---

## Installation

### Prerequisites

1. **Install .NET SDK 8.0**

```bash
# macOS
brew install --cask dotnet-sdk

# Windows
# Download from https://dotnet.microsoft.com/download

# Linux
wget https://dot.net/v1/dotnet-install.sh
chmod +x dotnet-install.sh
./dotnet-install.sh --channel 8.0
```

2. **Verify Installation**

```bash
dotnet --version
# Should output: 8.0.x
```

### Enable C# Support

In your `Cargo.toml`, add the `csharp` feature:

```toml
[dependencies]
game_engine = { path = "../game_engine", features = ["csharp"] }
```

### Project Structure

```
your_game/
├── scripts/           # C# scripts
│   ├── Components/    # Component scripts
│   ├── Systems/       # System scripts
│   └── Game/          # Game logic
├── src/
│   └── main.rs        # Rust main program
└── Cargo.toml
```

---

## Quick Start

### 1. Create Your First Script

Create `scripts/HelloWorld.cs`:

```csharp
using GameEngine;
using GameEngine.ECS;

public class HelloWorld : Component
{
    private void Start()
    {
        Debug.Log("Hello from C#!");
    }

    private void Update(float deltaTime)
    {
        if (Input.GetKeyDown(KeyCode.Space))
        {
            Debug.Log("Space key pressed!");
        }
    }
}
```

### 2. Load Scripts in Rust

In your `src/main.rs`:

```rust
use game_engine::scripting::csharp::{CSharpRuntime, CSharpConfig};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine instance
    let mut engine = Engine::new()?;

    // Initialize C# runtime
    let scripts_dir = PathBuf::from("scripts");
    let config = CSharpConfig {
        scripts_dir: scripts_dir.clone(),
        enable_hot_reload: true,
        ..Default::default()
    };

    let mut csharp_runtime = CSharpRuntime::new(config)?;
    csharp_runtime.load_scripts(&scripts_dir)?;

    // Add to engine
    engine.add_script_runtime(csharp_runtime);

    // Run game
    engine.run()?;

    Ok(())
}
```

### 3. Attach Script to Entity

```rust
// Create an entity
let entity = engine.create_entity(scene)?;

// Attach C# script
engine.add_csharp_script(entity, "HelloWorld")?;
```

---

## Core Concepts

### Component System

The engine uses an Entity Component System (ECS) similar to Unity:

```csharp
// Component: Data + Behavior
public class Player : Component
{
    public int Health = 100;
    public float Speed = 5.0f;

    private void Update(float deltaTime)
    {
        // Movement logic
        float move = Input.GetAxis("Vertical");
        Transform.position += Transform.forward * move * Speed * deltaTime;
    }
}
```

### Game Loop

```csharp
public class GameLoop : Component
{
    private void Awake()       // Called once when component is created
    private void Start()        // Called once before first Update
    private void Update(float deltaTime)        // Called every frame
    private void FixedUpdate(float fixedDeltaTime)  // Physics update (60 FPS)
    private void LateUpdate(float deltaTime)  // After all Update calls
    private void OnDestroy()    // Called when component is destroyed
}
```

### Transform Hierarchy

```csharp
// Parent-child relationships
Transform parent = this.Transform;
Transform child = transform.Find("ChildObject");

// Get/Set position, rotation, scale
Vector3 position = Transform.position;
Quaternion rotation = Transform.rotation;
Vector3 scale = Transform.localScale;

// Move and rotate
Transform.Translate(Vector3.forward * Speed * Time.deltaTime);
Transform.Rotate(Vector3.up * TurnSpeed * Time.deltaTime);
```

### Time

```csharp
// Time since game start
float time = Time.time;

// Time since last frame
float deltaTime = Time.deltaTime;

// Fixed time step (for physics)
float fixedDeltaTime = Time.fixedDeltaTime;

// Time scale (0 = paused, 1 = normal, 2 = fast)
Time.timeScale = 0.0f; // Pause
```

---

## API Reference

### Input System

```csharp
// Keyboard input
bool isPressed = Input.GetKey(KeyCode.Space);
bool wasPressed = Input.GetKeyDown(KeyCode.Space);
bool wasReleased = Input.GetKeyUp(KeyCode.Space);

// Mouse input
bool isClicked = Input.GetMouseButton(0);
bool wasClicked = Input.GetMouseButtonDown(0);
bool wasReleased = Input.GetMouseButtonUp(0);

// Input axes (WASD, arrows)
float horizontal = Input.GetAxis("Horizontal"); // -1 to 1
float vertical = Input.GetAxis("Vertical");     // -1 to 1

// Mouse position
Vector3 mousePos = Input.mousePosition;

// Mouse delta (movement since last frame)
float mouseX = Input.GetAxis("Mouse X");
float mouseY = Input.GetAxis("Mouse Y");

// Mouse scroll
float scroll = Input.GetAxis("Mouse ScrollWheel");
```

### Physics System

```csharp
// Character Controller (for player movement)
CharacterController controller = GetComponent<CharacterController>();
controller.Move(direction * speed * Time.deltaTime);
bool isGrounded = controller.IsGrounded;
controller.Jump(jumpForce);

// Rigidbody (for physics objects)
Rigidbody rb = GetComponent<Rigidbody>();
rb.velocity = Vector3.forward * speed;
rb.AddForce(Vector3.up * jumpForce);
rb.mass = 10.0f;

// Collider (for collision detection)
Collider collider = GetComponent<Collider>();
Collider[] overlaps = Physics.OverlapSphere(position, radius);

// Raycasting
Ray ray = new Ray(origin, direction);
if (Physics.Raycast(ray, out RaycastHit hit, maxDistance))
{
    Debug.Log("Hit: " + hit.collider.name);
    Vector3 point = hit.point;
    Vector3 normal = hit.normal;
}

// Trigger detection
private void OnTriggerEnter(Collider other)
{
    if (other.CompareTag("Pickup"))
    {
        CollectItem(other);
    }
}

private void OnTriggerExit(Collider other)
{
    Debug.Log("Exited trigger: " + other.name);
}

// Collision detection
private void OnCollisionEnter(Collision collision)
{
    Debug.Log("Collided with: " + collision.collider.name);
}
```

### Audio System

```csharp
// Play audio clip
AudioSource audioSource = GetComponent<AudioSource>();
audioSource.Play();
audioSource.PlayOneShot(clip);

// 3D spatial audio
AudioSource.PlayClipAtPoint(clip, position);

// Audio control
audioSource.volume = 0.5f;
audioSource.pitch = 1.0f;
audioSource.loop = true;
audioSource.Stop();

// Audio listener (usually on camera)
AudioListener listener = GetComponent<AudioListener>();
```

### UI System

```csharp
// Canvas
UICanvas canvas = GetComponent<UICanvas>();
canvas.renderMode = RenderMode.ScreenSpaceOverlay;

// UI Elements
UIButton button = GetComponent<UIButton>();
button.onClick.AddListener(OnButtonClick);

UIText text = GetComponent<UIText>();
text.text = "Score: " + score;

UIImage image = GetComponent<UIImage>();
image.color = Color.red;

// UI Updates (from scripts)
UI.UpdateScore(score);
UI.ShowMessage("Level Complete!");
UI.ShowDamageEffect();
```

### Coroutines

```csharp
// Start coroutine
StartCoroutine(MyCoroutine());

// Define coroutine
private IEnumerator MyCoroutine()
{
    Debug.Log("Coroutine started");

    // Wait for seconds
    yield return new WaitForSeconds(2.0f);

    Debug.Log("Waited 2 seconds");

    // Wait for frames
    yield return new WaitForFrames(60);

    // Wait until condition
    yield return new WaitUntil(() => Health <= 0);

    // Wait while condition
    yield return new WaitForSeconds(3.0f);

    // Nested coroutine
    yield return StartCoroutine(AnotherCoroutine());

    Debug.Log("Coroutine finished");
}

// Stop coroutine
StopCoroutine(MyCoroutine());
StopAllCoroutines();
```

---

## Components

### Built-in Components

#### Transform

```csharp
Transform transform = Transform;

// Position
Vector3 position = transform.position;
transform.position = new Vector3(0, 1, 0);

// Rotation
Quaternion rotation = transform.rotation;
transform.rotation = Quaternion.Euler(0, 90, 0);

// Scale
Vector3 scale = transform.localScale;
transform.localScale = new Vector3(1, 2, 1);

// Hierarchy
Transform parent = transform.parent;
transform.SetParent(newParent);

int childCount = transform.childCount;
Transform firstChild = transform.GetChild(0);

Transform found = transform.Find("ChildName");
```

#### Camera

```csharp
Camera camera = GetComponent<Camera>();

// Projection
camera.fieldOfView = 60.0f; // FOV
camera.nearClipPlane = 0.1f;
camera.farClipPlane = 1000.0f;

// View
Matrix4x4 viewMatrix = camera.worldToCameraMatrix;
Matrix4x4 projMatrix = camera.projectionMatrix;

// Raycasting
Ray ray = camera.ScreenPointToRay(Input.mousePosition);
```

#### Renderer

```csharp
MeshRenderer renderer = GetComponent<MeshRenderer>();

// Material
Material material = renderer.material;
material.color = Color.red;
material.mainTexture = texture;
material.SetFloat("_Metallic", 0.5f);

// Enable/disable
renderer.enabled = false; // Hide
renderer.enabled = true;  // Show
```

#### Light

```csharp
Light light = GetComponent<Light>();

// Type
light.type = LightType.Directional;
light.type = LightType.Point;
light.type = LightType.Spot;

// Properties
light.color = Color.white;
light.intensity = 1.0f;
light.range = 10.0f;
light.spotAngle = 45.0f;
```

### Creating Custom Components

```csharp
using GameEngine;
using GameEngine.ECS;

public class MyComponent : Component
{
    // Public fields (visible in Inspector)
    [Header("Settings")]
    public float MyValue = 10.0f;

    [Range(0, 100)]
    public int MyRange = 50;

    public Color MyColor = Color.blue;

    // Private fields
    private float internalState;

    // Lifecycle methods
    private void Awake()
    {
        // Called when component is created
    }

    private void Start()
    {
        // Called before first Update
    }

    private void Update(float deltaTime)
    {
        // Called every frame
    }

    private void FixedUpdate(float fixedDeltaTime)
    {
        // Called at fixed time step
    }

    private void LateUpdate(float deltaTime)
    {
        // Called after all Update calls
    }

    private void OnDestroy()
    {
        // Called when component is destroyed
    }

    // Custom methods
    public void DoSomething()
    {
        Debug.Log("Doing something...");
    }
}
```

---

## Systems

### Creating Systems

Systems are singleton managers that handle specific aspects of the game:

```csharp
using GameEngine;
using GameEngine.ECS;
using System.Collections.Generic;

public class GameManager : MonoBehaviour
{
    // Singleton instance
    public static GameManager Instance { get; private set; }

    private void Awake()
    {
        // Singleton pattern
        if (Instance != null && Instance != this)
        {
            Destroy(gameObject);
            return;
        }
        Instance = this;
    }

    private void Start()
    {
        // Initialize game
        StartGame();
    }

    public void StartGame()
    {
        Debug.Log("Game started!");
    }

    public void EndGame()
    {
        Debug.Log("Game over!");
    }
}
```

### Accessing Systems

```csharp
// Find system
GameManager gameManager = FindObjectOfType<GameManager>();

// Use singleton
GameManager.Instance.StartGame();
```

---

## Networking

### Network Behaviour

```csharp
using GameEngine;
using GameEngine.ECS;
using GameEngine.Network;

[NetworkBehaviour]
public class NetworkedPlayer : NetworkBehaviour
{
    // Synced variables
    [SyncVar]
    public Vector3 Position;

    [SyncVar]
    public int Health;

    // Check ownership
    private void Update()
    {
        if (!IsLocalPlayer)
        {
            // Interpolate position
            Transform.position = Vector3.Lerp(
                Transform.position,
                Position,
                Time.deltaTime * 10.0f
            );
            return;
        }

        // Local player input
        HandleInput();
    }

    // Command: Client → Server
    [Command]
    private void CmdUpdatePosition(Vector3 newPos)
    {
        Position = newPos; // Synced to all clients
    }

    // ClientRpc: Server → All Clients
    [ClientRpc]
    private void RpcShowEffect()
    {
        // Play effect on all clients
        Instantiate(EffectPrefab, Transform.position);
    }

    // Server-only
    [Server]
    private void ServerLogic()
    {
        if (!IsServer) return;
        // Server-side logic
    }

    // Client-only
    [Client]
    private void ClientLogic()
    {
        if (!IsClient) return;
        // Client-side logic
    }
}
```

### Network Manager

```csharp
public class NetworkManager : MonoBehaviour
{
    public int MaxPlayers = 8;
    public int Port = 27015;
    public string ServerAddress = "127.0.0.1";

    public void StartServer()
    {
        NetworkServer.Configure(MaxPlayers, Port);
        NetworkServer.Listen();
    }

    public void StartClient()
    {
        NetworkClient.Connect(ServerAddress, Port);
    }

    public void StopServer()
    {
        NetworkServer.Shutdown();
    }

    public void Disconnect()
    {
        NetworkClient.Disconnect();
    }
}
```

### Network Spawning

```csharp
// Server spawn
[Server]
public void SpawnEnemy()
{
    GameObject enemy = Instantiate(EnemyPrefab, position, rotation);
    NetworkServer.Spawn(enemy);
}

// Server destroy
[Server]
public void KillEnemy(GameObject enemy)
{
    NetworkServer.Destroy(enemy);
}
```

---

## Best Practices

### 1. Use Components Wisely

```csharp
// ✅ Good: Single responsibility
public class Health : Component
{
    public int CurrentHealth { get; set; }
    public void TakeDamage(int damage) { }
}

public class Movement : Component
{
    public void Move(Vector3 direction) { }
}

// ❌ Bad: God component
public class Player : Component
{
    public void Move() { }
    public void Shoot() { }
    public void TakeDamage() { }
    public void UpdateUI() { }
    // ... 100 more methods
}
```

### 2. Cache Component References

```csharp
// ❌ Bad: GetComponent every frame
private void Update()
{
    Rigidbody rb = GetComponent<Rigidbody>();
    rb.velocity = Vector3.forward;
}

// ✅ Good: Cache in Start
private Rigidbody rb;

private void Start()
{
    rb = GetComponent<Rigidbody>();
}

private void Update()
{
    rb.velocity = Vector3.forward;
}
```

### 3. Use Object Pooling

```csharp
// ❌ Bad: Instantiate/Destroy every frame
private void Fire()
{
    GameObject bullet = Instantiate(BulletPrefab);
    Destroy(bullet, 2.0f);
}

// ✅ Good: Use object pool
private void Fire()
{
    GameObject bullet = BulletPool.Get();
    bullet.SetActive(true);
}

private void ReturnBullet(GameObject bullet)
{
    bullet.SetActive(false);
    BulletPool.Return(bullet);
}
```

### 4. Avoid Null Reference Errors

```csharp
// ❌ Bad: Assume component exists
private void Start()
{
    GetComponent<Rigidbody>().velocity = Vector3.forward;
}

// ✅ Good: Check for null
private void Start()
{
    Rigidbody rb = GetComponent<Rigidbody>();
    if (rb != null)
    {
        rb.velocity = Vector3.forward;
    }
    else
    {
        Debug.LogWarning("Rigidbody not found!");
    }
}
```

### 5. Use Coroutines for Delays

```csharp
// ❌ Bad: Busy wait
private void Update()
{
    if (Time.time >= attackTime + 2.0f)
    {
        Attack();
    }
}

// ✅ Good: Use coroutine
private void Attack()
{
    StartCoroutine(AttackCoroutine());
}

private IEnumerator AttackCoroutine()
{
    Attack();
    yield return new WaitForSeconds(2.0f);
}
```

---

## Examples

### Complete Player Controller

```csharp
using GameEngine;
using GameEngine.ECS;

public class PlayerController : Component
{
    [Header("Movement")]
    public float WalkSpeed = 5.0f;
    public float SprintSpeed = 8.0f;
    public float JumpForce = 5.0f;

    [Header("Mouse Look")]
    public float MouseSensitivity = 100.0f;

    private CharacterController controller;
    private Camera camera;
    private float pitch = 0f;
    private float yaw = 0f;

    private void Start()
    {
        controller = GetComponent<CharacterController>();
        camera = FindObjectOfType<Camera>();

        Cursor.LockState = CursorLockMode.Locked;
        Cursor.Visible = false;
    }

    private void Update(float deltaTime)
    {
        HandleMovement();
        HandleMouseLook();
        HandleJump();
    }

    private void HandleMovement()
    {
        Vector3 move = Vector3.zero;

        if (Input.GetKey(KeyCode.W)) move += Vector3.forward;
        if (Input.GetKey(KeyCode.S)) move += Vector3.back;
        if (Input.GetKey(KeyCode.A)) move += Vector3.left;
        if (Input.GetKey(KeyCode.D)) move += Vector3.right;

        float speed = Input.GetKey(KeyCode.LeftShift) ? SprintSpeed : WalkSpeed;
        controller.Move(move.normalized * speed * deltaTime);
    }

    private void HandleMouseLook()
    {
        float mouseX = Input.GetAxis("Mouse X") * MouseSensitivity;
        float mouseY = Input.GetAxis("Mouse Y") * MouseSensitivity;

        yaw += mouseX * Time.deltaTime;
        pitch -= mouseY * Time.deltaTime;
        pitch = Mathf.Clamp(pitch, -89f, 89f);

        Transform.rotation = Quaternion.Euler(0, yaw, 0);
        camera.transform.localRotation = Quaternion.Euler(pitch, 0, 0);
    }

    private void HandleJump()
    {
        if (Input.GetKeyDown(KeyCode.Space) && controller.IsGrounded)
        {
            controller.Jump(JumpForce);
        }
    }
}
```

### Complete Weapon System

```csharp
using GameEngine;
using GameEngine.ECS;
using System.Collections;

public class Weapon : Component
{
    [Header("Stats")]
    public int Damage = 10;
    public float FireRate = 0.1f;
    public int MagazineSize = 30;
    public int ReserveAmmo = 120;

    [Header("Recoil")]
    public float RecoilForce = 0.5f;

    private int currentAmmo;
    private int currentReserve;
    private float lastFireTime;
    private bool isReloading;

    private void Awake()
    {
        currentAmmo = MagazineSize;
        currentReserve = ReserveAmmo;
    }

    private void Update(float deltaTime)
    {
        if (Input.GetMouseButton(0))
        {
            TryFire();
        }

        if (Input.GetKeyDown(KeyCode.R))
        {
            Reload();
        }
    }

    public void TryFire()
    {
        if (Time.time - lastFireTime < FireRate)
        {
            return;
        }

        if (currentAmmo <= 0)
        {
            Reload();
            return;
        }

        Fire();
        lastFireTime = Time.time;
        currentAmmo--;
    }

    private void Fire()
    {
        Ray ray = Camera.main.ScreenPointToRay(Input.mousePosition);

        if (Physics.Raycast(ray, out RaycastHit hit, 1000f))
        {
            Debug.Log("Hit: " + hit.collider.name);

            // Apply damage
            Health health = hit.collider.GetComponent<Health>();
            if (health != null)
            {
                health.TakeDamage(Damage);
            }
        }

        ApplyRecoil();
    }

    private void ApplyRecoil()
    {
        Camera camera = Camera.main;
        camera.transform.Rotate(Vector3.left, RecoilForce);
    }

    public void Reload()
    {
        if (isReloading)
        {
            return;
        }

        StartCoroutine(ReloadCoroutine());
    }

    private IEnumerator ReloadCoroutine()
    {
        isReloading = true;

        yield return new WaitForSeconds(2.0f);

        int needed = MagazineSize - currentAmmo;
        int available = Mathf.Min(needed, currentReserve);

        currentAmmo += available;
        currentReserve -= available;

        isReloading = false;
    }
}
```

---

## Troubleshooting

### Script Not Loading

**Problem**: C# script doesn't execute

**Solutions**:
1. Check file is in `scripts/` directory
2. Check namespace matches file path
3. Check class name matches file name
4. Check for compilation errors in console

### Null Reference Exception

**Problem**: `NullReferenceException` when accessing component

**Solutions**:
1. Always check if `GetComponent()` returns null
2. Use `GetComponent<T>()` instead of `GetComponent(typeof(T))`
3. Add null checks before accessing members

### Performance Issues

**Problem**: Low FPS with many C# scripts

**Solutions**:
1. Avoid `GetComponent()` in Update (cache in Start)
2. Use object pooling for frequently spawned objects
3. Minimize allocations in Update
4. Profile with performance tools

### Hot Reload Not Working

**Problem**: Changes to C# scripts not reflected

**Solutions**:
1. Ensure `enable_hot_reload: true` in CSharpConfig
2. Save the file (Ctrl+S / Cmd+S)
3. Check console for reload messages
4. Restart game if compilation error

### Network Sync Issues

**Problem**: Positions not syncing correctly

**Solutions**:
1. Ensure `[SyncVar]` attribute on synced variables
2. Use `[Command]` for client→server calls
3. Use `[ClientRpc]` for server→client calls
4. Check `IsLocalPlayer` before processing input

---

## Performance Tips

### 1. Minimize Allocations

```csharp
// ❌ Bad: Allocates every frame
private void Update()
{
    List<Enemy> enemies = new List<Enemy>();
}

// ✅ Good: Cache and reuse
private List<Enemy> enemyCache = new List<Enemy>();

private void Update()
{
    enemyCache.Clear();
}
```

### 2. Use Object Pooling

```csharp
// Create pool
private Stack<GameObject> bulletPool = new Stack<GameObject>();

public GameObject GetBullet()
{
    return bulletPool.Count > 0 ? bulletPool.Pop() : Instantiate(BulletPrefab);
}

public void ReturnBullet(GameObject bullet)
{
    bullet.SetActive(false);
    bulletPool.Push(bullet);
}
```

### 3. Avoid Physics Queries

```csharp
// ❌ Bad: Raycast every frame
private void Update()
{
    Physics.Raycast(transform.position, transform.forward, out RaycastHit hit);
}

// ✅ Good: Cache result
private RaycastHit lastHit;
private float lastRaycastTime;

private void Update()
{
    if (Time.time - lastRaycastTime > 0.1f)
    {
        Physics.Raycast(transform.position, transform.forward, out lastHit);
        lastRaycastTime = Time.time;
    }
}
```

### 4. Use FixedUpdate for Physics

```csharp
// ❌ Bad: Physics in Update
private void Update(float deltaTime)
{
    rb.velocity = Vector3.forward * speed;
}

// ✅ Good: Physics in FixedUpdate
private void FixedUpdate(float fixedDeltaTime)
{
    rb.velocity = Vector3.forward * speed;
}
```

---

## Additional Resources

- [FPS Demo Example](../examples/csharp_games/fps_demo/README.md)
- [Tank Battle Example](../examples/csharp_games/tank_battle/README.md)
- [Brick Breaker Example](../examples/csharp_games/brick_breaker/README.md)
- [Unity API Reference](https://docs.unity3d.com/ScriptReference/)
- [C# Language Reference](https://docs.microsoft.com/en-us/dotnet/csharp/)

---

## Changelog

### Version 0.1.0 (2026-01-03)
- Initial release
- Core component system
- Input, physics, audio APIs
- Networking support
- Hot reload functionality
- Three game examples

---

## License

MIT License - See LICENSE file for details

---

**Document Version**: 0.1.0
**Last Updated**: 2026-01-03
**Maintained By**: Game Engine Team

For questions or issues, please visit: https://github.com/yourusername/game_engine
