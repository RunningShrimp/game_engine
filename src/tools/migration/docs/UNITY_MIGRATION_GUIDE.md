# Unity Migration Guide

This guide explains how to migrate Unity projects to the game engine.

## Overview

The migration tools provide automated conversion of:
- **Scenes**: GameObject hierarchy → Entity-Component system
- **Assets**: Textures, models, audio, materials
- **Scripts**: C# → Lua/Rust
- **Prefabs**: Reusable entity templates

## Prerequisites

1. **Unity Project**: A working Unity project
2. **Engine Installation**: Latest engine version
3. **Backup**: Always backup your project before migration

## Quick Start

```bash
# Run migration
cargo run --bin migrate -- --source /path/to/unity/project --output /path/to/output

# The migration will:
# 1. Parse Unity project structure
# 2. Migrate scenes to entity format
# 3. Convert assets to engine formats
# 4. Convert scripts to Lua
# 5. Generate migration report
```

## Supported Features

### Scenes

✅ **Supported:**
- GameObject hierarchy
- Transform components (position, rotation, scale)
- Parent-child relationships
- Active/inactive state
- Layer tags
- Static flags

⚠️ **Partial Support:**
- Canvas and UI elements (requires manual adjustment)
- Terrain data (needs conversion)
- Lightmapping (manual setup required)

❌ **Not Supported:**
- GameObject linking (cross-scene references)
- Scene loading/unloading scripts
- Addressables asset system

### Components

| Unity Component | Engine Component | Notes |
|----------------|------------------|-------|
| Transform | Transform | Direct mapping |
| MeshRenderer | MeshRenderer | Material conversion required |
| MeshFilter | MeshFilter | Direct mapping |
| BoxCollider | BoxCollider | Direct mapping |
| SphereCollider | SphereCollider | Direct mapping |
| Rigidbody | Rigidbody | Physics parameters differ |
| Camera | Camera | API differences |
| Light | Light | API differences |
| AudioSource | AudioSource | Direct mapping |
| Animator | Animator | Requires manual setup |

### Assets

✅ **Textures:**
- PNG, JPG, TGA → PNG (lossless)
- PSD → PNG (flattened)
- Compressed textures → Decompressed PNG

✅ **Models:**
- FBX → glTF 2.0
- OBJ → glTF 2.0

✅ **Audio:**
- All formats → WAV (uncompressed)
- Quality preservation

⚠️ **Materials:**
- Basic structure converted
- Shader code requires manual porting
- Material parameters mapped

### Scripts

✅ **Language Conversion:**
- C# → Lua (default)
- C# → Rust (experimental)

⚠️ **API Mapping:**
- Common APIs are auto-mapped
- Custom APIs require manual conversion
- Unity-specific features need alternative approaches

## Migration Process

### 1. Scene Migration

**Input:** `.unity` scene files
**Output:** Entity definitions in RON format

**Steps:**
1. Parse YAML scene format
2. Convert GameObjects to Entities
3. Map components
4. Preserve hierarchy
5. Handle prefabs instances

**Example:**

Unity:
```yaml
GameObject:
  m_Name: "Player"
  m_Component:
  - {fileID: 123456}  # Transform
  - {fileID: 234567}  # Rigidbody
```

Output:
```rust
Entity(
    name: "Player",
    components: [
        Transform(
            position: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
        ),
        Rigidbody(
            mass: 1.0,
            use_gravity: true,
        ),
    ],
)
```

### 2. Asset Conversion

**Textures:**
```
Assets/Textures/player.png → converted_assets/textures/player.png
```

**Models:**
```
Assets/Models/car.fbx → converted_assets/models/car.gltf
```

**Audio:**
```
Assets/Audio/jump.wav → converted_assets/audio/jump.wav
```

### 3. Script Migration

**Unity C#:**
```csharp
public class PlayerController : MonoBehaviour {
    public float speed = 5.0f;

    void Update() {
        float move = Input.GetAxis("Horizontal");
        transform.position += new Vector3(move * speed * Time.deltaTime, 0, 0);
    }
}
```

**Engine Lua:**
```lua
local PlayerController = {}

function PlayerController.on_update(self, dt)
    local move = input:get_axis("Horizontal")
    local pos = self.entity:get_position()
    pos.x = pos.x + move * self.speed * dt
    self.entity:set_position(pos)
end

return PlayerController
```

## Post-Migration Tasks

### 1. Review Scenes

- [ ] Check entity hierarchy
- [ ] Verify component data
- [ ] Adjust transform values
- [ ] Test physics colliders

### 2. Verify Assets

- [ ] Check texture quality
- [ ] Verify model geometry
- [ ] Test audio playback
- [ ] Review material conversions

### 3. Port Scripts

- [ ] Review auto-converted code
- [ ] Fix compilation errors
- [ ] Port custom logic
- [ ] Test gameplay functionality

### 4. Adjust Materials

- [ ] Recreate shaders
- [ ] Tune material parameters
- [ ] Set up lighting
- [ ] Configure render settings

### 5. Test Thoroughly

- [ ] Scene loading
- [ ] Physics simulation
- [ ] Audio playback
- [ ] UI interactions
- [ ] Performance profiling

## Common Issues

### Issue: Script Not Working

**Symptoms:** Auto-converted Lua script fails

**Solution:**
1. Check API mappings
2. Review manual change requirements
3. Port complex logic manually
4. Test incremental changes

### Issue: Material Looks Different

**Symptoms:** Converted materials don't match Unity appearance

**Solution:**
1. Unity and engine use different shading models
2. Manually recreate shaders
3. Adjust material parameters
4. Use similar textures

### Issue: Physics Behaves Differently

**Symptoms:** Objects fall/move differently

**Solution:**
1. Physics engines have different implementations
2. Adjust gravity, mass, drag values
3. Recreate complex joint constraints
4. Tune collider shapes

### Issue: Missing Prefab References

**Symptoms:** Prefab instances not linked correctly

**Solution:**
1. Prefabs become entity templates
2. Instantiate manually in code
3. Use engine's template system

## API Reference

### Unity → Engine API Mapping

| Unity API | Engine API | Notes |
|-----------|------------|-------|
| `transform.position` | `entity:get_position()` | Method call |
| `transform.rotation` | `entity:get_rotation()` | Quaternion format |
| `rigidbody.AddForce()` | `rigidbody:apply_force()` | Same API |
| `Debug.Log()` | `print()` | No console context |
| `Input.GetAxis()` | `input:get_axis()` | Different axis names |
| `GameObject.Find()` | `world:find_entity()` | Returns entity ID |
| `Time.deltaTime` | `dt` parameter | Passed to update |

## Best Practices

### Before Migration

1. **Clean Up Project**
   - Remove unused assets
   - Fix broken references
   - Organize folder structure

2. **Document Custom Systems**
   - Note custom components
   - Document editor tools
   - List third-party plugins

3. **Test in Unity**
   - Verify everything works
   - Note expected behaviors
   - Take screenshots for reference

### During Migration

1. **Iterative Approach**
   - Migrate one scene at a time
   - Test each migration
   - Fix issues before continuing

2. **Version Control**
   - Commit after each successful step
   - Keep migration scripts
   - Document manual changes

3. **Backup Frequently**
   - Keep original project intact
   - Save intermediate results
   - Document conversion decisions

### After Migration

1. **Performance Testing**
   - Profile frame rate
   - Check memory usage
   - Optimize hot paths

2. **Functionality Testing**
   - Test all gameplay features
   - Verify UI interactions
   - Check edge cases

3. **Polish**
   - Adjust visuals
   - Tune gameplay
   - Optimize assets

## Limitations

### Not Supported

- **Editor Tools**: Custom editors, inspectors, gizmos
- **Asset Bundles**: Use engine's packaging system instead
- **Unity Analytics**: Use alternative analytics
- **Unity Ads**: Not supported
- **Unity IAP**: Not supported
- **NavMesh**: Use engine's navigation system
- **Timeline**: Manual sequencing required
- **Cinemachine**: Manual camera control

### Partial Support

- **Shaders**: HLSL → GLSL conversion required
- **Animation**: State machines need recreation
- **UI**: Canvas-based UI needs redesign
- **Particles**: Custom particle systems required
- **Audio**: 3D spatial audio differs

## Getting Help

### Documentation
- Engine API docs: `/docs/api/`
- Plugin development: `/docs/plugins/`
- Scripting reference: `/docs/scripting/`

### Community
- Forum: [community.example.com]
- Discord: [discord.gg/engine]
- GitHub Issues: [github.com/engine/issues]

### Professional Support
- Email: support@example.com
- Migration services available
- Custom training options

## Additional Resources

- [Scripting API Reference](../scripting/README.md)
- [Component Reference](../components/README.md)
- [Asset Pipeline Guide](../assets/README.md)
- [Performance Tuning](../performance/README.md)

## Changelog

### v1.0.0 (2024-01-01)
- Initial migration tools
- Basic scene conversion
- Asset format conversion
- C# to Lua script migration
