# Audio API

This document describes the audio system API.

## Overview

The audio system handles sound and music playback.

## Key Functions

### Playing Sounds

```rust
use game_engine::audio::AudioEngine;

engine.audio.play_sound("explosion.wav")?;
```

### Playing Music

```rust
engine.audio.play_music("background.mp3", true)?;
```

## See Also

- [Audio System](../audio_system.md)
- [Engine API](./engine.md)
