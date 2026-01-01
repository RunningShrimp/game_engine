//! # C# SDK Templates
//!
//! Templates for generating C# SDK code.

/// 生成核心API模板
pub fn core_api_template(namespace: &str, version: &str) -> String {
    format!(
        r#"// Game Engine C# SDK - Core API
// Version: {}

using System;
using System.Runtime.InteropServices;

namespace {}
{{
    /// <summary>
    /// Core game engine API
    /// </summary>
    public static class CoreAPI
    {{
        /// <summary>
        /// Get the current entity ID
        /// </summary>
        public static ulong GetCurrentEntity()
        {{
            return NativeAPI.GetCurrentEntity();
        }}

        /// <summary>
        /// Log a message
        /// </summary>
        public static void Log(string message)
        {{
            NativeAPI.Log(message);
        }}

        /// <summary>
        /// Log a warning
        /// </summary>
        public static void LogWarning(string message)
        {{
            NativeAPI.LogWarning(message);
        }}

        /// <summary>
        /// Log an error
        /// </summary>
        public static void LogError(string message)
        {{
            NativeAPI.LogError(message);
        }}

        /// <summary>
        /// Get delta time
        /// </summary>
        public static float GetDeltaTime()
        {{
            return NativeAPI.GetDeltaTime();
        }}

        /// <summary>
        /// Get fixed delta time
        /// </summary>
        public static float GetFixedDeltaTime()
        {{
            return NativeAPI.GetFixedDeltaTime();
        }}
    }}

    /// <summary>
    /// Native API bindings (P/Invoke)
    /// </summary>
    internal static class NativeAPI
    {{
        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern ulong GetCurrentEntity();

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void Log([MarshalAs(UnmanagedType.LPStr)] string message);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void LogWarning([MarshalAs(UnmanagedType.LPStr)] string message);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void LogError([MarshalAs(UnmanagedType.LPStr)] string message);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern float GetDeltaTime();

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern float GetFixedDeltaTime();
    }}
}}
"#,
        version, namespace
    )
}

/// 生成生命周期钩子模板
pub fn lifecycle_hooks_template(namespace: &str) -> String {
    format!(
        r#"// Game Engine C# SDK - Lifecycle Hooks
// Unity-style lifecycle hooks for C# scripts

using System;

namespace {}
{{
    /// <summary>
    /// Base class for game scripts with lifecycle hooks
    /// </summary>
    public abstract class MonoBehaviour
    {{
        /// <summary>
        /// The entity this script is attached to
        /// </summary>
        public ulong Entity {{ get; internal set; }}

        /// <summary>
        /// Called when the script is first initialized
        /// </summary>
        public virtual void OnStart() {{ }}

        /// <summary>
        /// Called when the script is enabled
        /// </summary>
        public virtual void OnEnable() {{ }}

        /// <summary>
        /// Called when the script is disabled
        /// </summary>
        public virtual void OnDisable() {{ }}

        /// <summary>
        /// Called when the script is destroyed
        /// </summary>
        public virtual void OnDestroy() {{ }}

        /// <summary>
        /// Called every frame
        /// </summary>
        public virtual void OnUpdate() {{ }}

        /// <summary>
        /// Called at fixed time intervals (for physics)
        /// </summary>
        public virtual void OnFixedUpdate() {{ }}

        /// <summary>
        /// Called after all OnUpdate calls
        /// </summary>
        public virtual void OnLateUpdate() {{ }}

        /// <summary>
        /// Called when a collision starts
        /// </summary>
        public virtual void OnCollisionEnter(ulong otherEntity) {{ }}

        /// <summary>
        /// Called while a collision is ongoing
        /// </summary>
        public virtual void OnCollisionStay(ulong otherEntity) {{ }}

        /// <summary>
        /// Called when a collision ends
        /// </summary>
        public virtual void OnCollisionExit(ulong otherEntity) {{ }}

        /// <summary>
        /// Called when a trigger starts
        /// </summary>
        public virtual void OnTriggerEnter(ulong otherEntity) {{ }}

        /// <summary>
        /// Called while a trigger is ongoing
        /// </summary>
        public virtual void OnTriggerStay(ulong otherEntity) {{ }}

        /// <summary>
        /// Called when a trigger ends
        /// </summary>
        public virtual void OnTriggerExit(ulong otherEntity) {{ }}

        /// <summary>
        /// Called when a key is pressed
        /// </summary>
        public virtual void OnKeyDown(string key) {{ }}

        /// <summary>
        /// Called when a key is released
        /// </summary>
        public virtual void OnKeyUp(string key) {{ }}

        /// <summary>
        /// Called when a mouse button is pressed
        /// </summary>
        public virtual void OnMouseDown(string button) {{ }}

        /// <summary>
        /// Called when a mouse button is released
        /// </summary>
        public virtual void OnMouseUp(string button) {{ }}

        /// <summary>
        /// Called when the application is paused
        /// </summary>
        public virtual void OnPause() {{ }}

        /// <summary>
        /// Called when the application is resumed
        /// </summary>
        public virtual void OnResume() {{ }}
    }}
}}
"#,
        namespace
    )
}

/// 生成物理API模板
pub fn physics_api_template(namespace: &str) -> String {
    format!(
        r#"// Game Engine C# SDK - Physics API

using System;
using System.Numerics;

namespace {}
{{
    /// <summary>
    /// Physics API for interacting with the physics system
    /// </summary>
    public static class Physics
    {{
        /// <summary>
        /// Apply a force to a rigid body
        /// </summary>
        public static void AddForce(ulong entity, Vector3 force)
        {{
            NativePhysics.AddForce(entity, force.X, force.Y, force.Z);
        }}

        /// <summary>
        /// Apply a torque to a rigid body
        /// </summary>
        public static void AddTorque(ulong entity, Vector3 torque)
        {{
            NativePhysics.AddTorque(entity, torque.X, torque.Y, torque.Z);
        }}

        /// <summary>
        /// Set the linear velocity of a rigid body
        /// </summary>
        public static void SetLinearVelocity(ulong entity, Vector3 velocity)
        {{
            NativePhysics.SetLinearVelocity(entity, velocity.X, velocity.Y, velocity.Z);
        }}

        /// <summary>
        /// Get the linear velocity of a rigid body
        /// </summary>
        public static Vector3 GetLinearVelocity(ulong entity)
        {{
            NativePhysics.GetLinearVelocity(entity, out float x, out float y, out float z);
            return new Vector3(x, y, z);
        }}

        /// <summary>
        /// Perform a raycast
        /// </summary>
        public static bool Raycast(Vector3 origin, Vector3 direction, float maxDistance, out RaycastHit hit)
        {{
            hit = new RaycastHit();
            return NativePhysics.Raycast(origin.X, origin.Y, origin.Z,
                                         direction.X, direction.Y, direction.Z,
                                         maxDistance, out hit.Entity, out hit.Point.X, out hit.Point.Y, out hit.Point.Z,
                                         out hit.Normal.X, out hit.Normal.Y, out hit.Normal.Z, out hit.Distance);
        }}
    }}

    /// <summary>
    /// Raycast hit information
    /// </summary>
    public struct RaycastHit
    {{
        public ulong Entity;
        public Vector3 Point;
        public Vector3 Normal;
        public float Distance;
    }}

    internal static class NativePhysics
    {{
        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void AddForce(ulong entity, float x, float y, float z);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void AddTorque(ulong entity, float x, float y, float z);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void SetLinearVelocity(ulong entity, float x, float y, float z);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void GetLinearVelocity(ulong entity, out float x, out float y, out float z);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern bool Raycast(float ox, float oy, float oz, float dx, float dy, float dz,
                                            float maxDistance, out ulong entity, out float px, out float py, out float pz,
                                            out float nx, out float ny, out float nz, out float distance);
    }}
}}
"#,
        namespace
    )
}

/// 生成音频API模板
pub fn audio_api_template(namespace: &str) -> String {
    format!(
        r#"// Game Engine C# SDK - Audio API

using System;

namespace {}
{{
    /// <summary>
    /// Audio API for playing sounds and music
    /// </summary>
    public static class Audio
    {{
        /// <summary>
        /// Play a 2D sound
        /// </summary>
        public static void Play2D(string clipPath, float volume = 1.0f)
        {{
            NativeAudio.Play2D(clipPath, volume);
        }}

        /// <summary>
        /// Play a 3D sound at a position
        /// </summary>
        public static void Play3D(string clipPath, System.Numerics.Vector3 position, float volume = 1.0f)
        {{
            NativeAudio.Play3D(clipPath, position.X, position.Y, position.Z, volume);
        }}

        /// <summary>
        /// Stop a sound
        /// </summary>
        public static void Stop(ulong sourceId)
        {{
            NativeAudio.Stop(sourceId);
        }}

        /// <summary>
        /// Set the volume of a sound
        /// </summary>
        public static void SetVolume(ulong sourceId, float volume)
        {{
            NativeAudio.SetVolume(sourceId, volume);
        }}
    }}

    internal static class NativeAudio
    {{
        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void Play2D([MarshalAs(UnmanagedType.LPStr)] string clipPath, float volume);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void Play3D([MarshalAs(UnmanagedType.LPStr)] string clipPath, float x, float y, float z, float volume);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void Stop(ulong sourceId);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void SetVolume(ulong sourceId, float volume);
    }}
}}
"#,
        namespace
    )
}

/// 生成网络API模板
pub fn network_api_template(namespace: &str) -> String {
    format!(
        r#"// Game Engine C# SDK - Network API

using System;

namespace {}
{{
    /// <summary>
    /// Network API for network communication
    /// </summary>
    public static class Network
    {{
        /// <summary>
        /// Connect to a server
        /// </summary>
        public static bool Connect(string url)
        {{
            return NativeNetwork.Connect(url);
        }}

        /// <summary>
        /// Send data over the network
        /// </summary>
        public static bool Send(byte[] data)
        {{
            return NativeNetwork.Send(data, data.Length);
        }}

        /// <summary>
        /// Receive data from the network
        /// </summary>
        public static byte[] Receive()
        {{
            // TODO: Implement receive
            return new byte[0];
        }}
    }}

    internal static class NativeNetwork
    {{
        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern bool Connect([MarshalAs(UnmanagedType.LPStr)] string url);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern bool Send(byte[] data, int length);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern bool Receive(byte[] buffer, int length, out int received);
    }}
}}
"#,
        namespace
    )
}

/// 生成输入API模板
pub fn input_api_template(namespace: &str) -> String {
    format!(
        r#"// Game Engine C# SDK - Input API

namespace {}
{{
    /// <summary>
    /// Input API for keyboard and mouse input
    /// </summary>
    public static class Input
    {{
        /// <summary>
        /// Check if a key is currently pressed
        /// </summary>
        public static bool GetKey(string key)
        {{
            return NativeInput.GetKey(key);
        }}

        /// <summary>
        /// Check if a key was pressed this frame
        /// </summary>
        public static bool GetKeyDown(string key)
        {{
            return NativeInput.GetKeyDown(key);
        }}

        /// <summary>
        /// Check if a key was released this frame
        /// </summary>
        public static bool GetKeyUp(string key)
        {{
            return NativeInput.GetKeyUp(key);
        }}

        /// <summary>
        /// Get mouse position
        /// </summary>
        public static System.Numerics.Vector2 GetMousePosition()
        {{
            NativeInput.GetMousePosition(out float x, out float y);
            return new System.Numerics.Vector2(x, y);
        }}
    }}

    internal static class NativeInput
    {{
        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern bool GetKey([MarshalAs(UnmanagedType.LPStr)] string key);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern bool GetKeyDown([MarshalAs(UnmanagedType.LPStr)] string key);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern bool GetKeyUp([MarshalAs(UnmanagedType.LPStr)] string key);

        [DllImport("game_engine", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void GetMousePosition(out float x, out float y);
    }}
}}
"#,
        namespace
    )
}

/// 生成ECS API模板
pub fn ecs_api_template(namespace: &str) -> String {
    format!(
        r#"// Game Engine C# SDK - ECS API

namespace {}
{{
    /// <summary>
    /// ECS API for entity and component operations
    /// </summary>
    public static class ECS
    {{
        /// <summary>
        /// Get a component from an entity
        /// </summary>
        public static T GetComponent<T>(ulong entity) where T : class
        {{
            // TODO: Implement component retrieval
            return null;
        }}

        /// <summary>
        /// Add a component to an entity
        /// </summary>
        public static void AddComponent<T>(ulong entity, T component) where T : class
        {{
            // TODO: Implement component addition
        }}

        /// <summary>
        /// Remove a component from an entity
        /// </summary>
        public static void RemoveComponent<T>(ulong entity) where T : class
        {{
            // TODO: Implement component removal
        }}
    }}
}}
"#,
        namespace
    )
}

/// 生成资源API模板
pub fn resource_api_template(namespace: &str) -> String {
    format!(
        r#"// Game Engine C# SDK - Resource API

namespace {}
{{
    /// <summary>
    /// Resource API for loading and managing game resources
    /// </summary>
    public static class Resources
    {{
        /// <summary>
        /// Load a resource
        /// </summary>
        public static T Load<T>(string path) where T : class
        {{
            // TODO: Implement resource loading
            return null;
        }}

        /// <summary>
        /// Unload a resource
        /// </summary>
        public static void Unload(string path)
        {{
            // TODO: Implement resource unloading
        }}
    }}
}}
"#,
        namespace
    )
}

/// 生成项目文件模板
pub fn project_file_template(namespace: &str) -> String {
    format!(
        r#"<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
    <LangVersion>latest</LangVersion>
    <Nullable>enable</Nullable>
    <AllowUnsafeBlocks>true</AllowUnsafeBlocks>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="System.Numerics.Vectors" Version="4.5.0" />
  </ItemGroup>

</Project>
"#
    )
}

/// 生成README模板
pub fn readme_template(namespace: &str) -> String {
    format!(
        r#"# Game Engine C# SDK

This SDK provides C# bindings for the game engine, allowing you to write game scripts in C#.

## Features

- Unity-style lifecycle hooks (OnStart, OnUpdate, etc.)
- Physics API
- Audio API
- Network API
- Input API
- ECS API
- Resource API

## Usage

1. Add this SDK to your C# project
2. Inherit from `MonoBehaviour` for your scripts
3. Override lifecycle methods as needed

Example:

```csharp
using {};

public class PlayerController : MonoBehaviour
{{
    public override void OnStart()
    {{
        CoreAPI.Log("Player controller started");
    }}

    public override void OnUpdate()
    {{
        if (Input.GetKey("Space"))
        {{
            // Jump
        }}
    }}
}}
```

## API Documentation

See individual API files for detailed documentation.
"#,
        namespace
    )
}
