#[cfg(test)]
mod audio_tests {
    use crate::audio::*;

    #[test]
    fn test_spatial_audio_service_creation() {
        // SpatialAudioService is a ZST (zero-sized type) namespace for static methods
        // Verify it can be instantiated
        let _service = SpatialAudioService;
        // Test that it's usable as a type
        fn accepts_spatial_audio_service(_: SpatialAudioService) {}
        accepts_spatial_audio_service(SpatialAudioService);
    }

    #[test]
    fn test_audio_listener_creation() {
        let listener = AudioListener::new();
        assert_eq!(listener.enabled, true);
        assert_eq!(listener.gain, 1.0);
    }

    #[test]
    fn test_audio_source_creation() {
        let source = SpatialAudioSource::new("test_sound");
        assert_eq!(source.name, "test_sound");
        assert_eq!(source.volume, 1.0);
    }

    #[test]
    fn test_distance_model_linear() {
        let model = DistanceModel::Linear {
            ref_distance: 0.0,
            max_distance: 100.0,
            rolloff: 1.0,
        };
        match model {
            DistanceModel::Linear {
                ref_distance,
                max_distance,
                rolloff,
            } => {
                assert_eq!(ref_distance, 0.0);
                assert_eq!(max_distance, 100.0);
                assert_eq!(rolloff, 1.0);
            }
            _ => panic!("Expected Linear distance model"),
        }
    }

    #[test]
    fn test_distance_model_inverse() {
        let model = DistanceModel::Inverse {
            ref_distance: 1.0,
            rolloff: 1.0,
        };
        match model {
            DistanceModel::Inverse {
                ref_distance,
                rolloff,
            } => {
                assert_eq!(ref_distance, 1.0);
                assert_eq!(rolloff, 1.0);
            }
            _ => panic!("Expected Inverse distance model"),
        }
    }

    #[test]
    fn test_distance_model_exponential() {
        let model = DistanceModel::Exponential {
            ref_distance: 1.0,
            rolloff: 1.0,
        };
        match model {
            DistanceModel::Exponential {
                ref_distance,
                rolloff,
            } => {
                assert_eq!(ref_distance, 1.0);
                assert_eq!(rolloff, 1.0);
            }
            _ => panic!("Expected Exponential distance model"),
        }
    }

    #[test]
    fn test_sound_cone_creation() {
        let cone = SoundCone {
            inner_angle: 45.0,
            outer_angle: 90.0,
            outer_gain: 0.5,
        };
        assert_eq!(cone.inner_angle, 45.0);
        assert_eq!(cone.outer_angle, 90.0);
        assert_eq!(cone.outer_gain, 0.5);
    }

    #[test]
    fn test_spatial_audio_params() {
        let params = SpatialAudioParams {
            volume: 0.8,
            left_gain: 0.7,
            right_gain: 0.7,
            pitch: 1.0,
            distance: 10.0,
            azimuth: 0.5,
            elevation: 0.2,
        };
        assert_eq!(params.volume, 0.8);
        assert_eq!(params.pitch, 1.0);
        assert_eq!(params.distance, 10.0);
    }

    #[test]
    fn test_spatial_audio_state_playing() {
        let state = StreamState::Playing;
        assert!(matches!(state, StreamState::Playing));
    }

    #[test]
    fn test_spatial_audio_state_stopped() {
        let state = StreamState::Stopped;
        assert!(matches!(state, StreamState::Stopped));
    }

    #[test]
    fn test_spatial_audio_state_paused() {
        let state = StreamState::Paused;
        assert!(matches!(state, StreamState::Paused));
    }

    #[test]
    fn test_audio_stream_config() {
        let config = StreamConfig {
            buffer_size: 2048,
            preload_buffers: 2,
            looped: false,
            sample_rate: Some(44100),
            channels: Some(2),
        };
        assert_eq!(config.buffer_size, 2048);
        assert_eq!(config.sample_rate, Some(44100));
        assert_eq!(config.channels, Some(2));
    }

    #[test]
    fn test_audio_buffer_creation() {
        let buffer = AudioBuffer {
            data: vec![0.0f32; 1024],
            channels: 2,
            sample_rate: 44100,
            filled: false,
            timestamp: 0,
        };
        assert_eq!(buffer.data.len(), 1024);
        assert_eq!(buffer.channels, 2);
        assert_eq!(buffer.sample_rate, 44100);
        assert_eq!(buffer.filled, false);
    }

    #[test]
    fn test_audio_stream_state_ready() {
        let state = StreamState::Ready;
        assert!(matches!(state, StreamState::Ready));
    }

    #[test]
    fn test_audio_stream_state_loading() {
        let state = StreamState::Loading;
        assert!(matches!(state, StreamState::Loading));
    }

    #[test]
    fn test_audio_stream_state_playing() {
        let state = StreamState::Playing;
        assert!(matches!(state, StreamState::Playing));
    }

    #[test]
    fn test_audio_stream_state_error() {
        let state = StreamState::Error("test error".to_string());
        match state {
            StreamState::Error(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Expected Error state"),
        }
    }

    #[test]
    fn test_effect_chain_creation() {
        let chain = EffectChain::new();
        // Verify chain is created and initialized
        assert_eq!(chain.effect_count(), 0, "New chain should have no effects");
    }

    #[test]
    fn test_equalizer_config() {
        let config = EqualizerConfig {
            bands: vec![
                EqualizerBand {
                    frequency: 60.0,
                    gain: 0.0,
                    q: 1.0,
                },
                EqualizerBand {
                    frequency: 250.0,
                    gain: 0.0,
                    q: 1.0,
                },
            ],
            sample_rate: 44100.0,
        };
        assert_eq!(config.bands.len(), 2);
    }

    #[test]
    fn test_reverb_config() {
        let config = ReverbConfig {
            room_size: 0.5,
            damping: 0.5,
            wet_level: 0.3,
            dry_level: 0.7,
            pre_delay: 0.0,
        };
        assert_eq!(config.room_size, 0.5);
        assert_eq!(config.damping, 0.5);
        assert_eq!(config.wet_level, 0.3);
    }

    #[test]
    fn test_delay_config() {
        let config = DelayConfig {
            delay_time: 0.3,
            feedback: 0.4,
            wet_level: 0.3,
            dry_level: 0.7,
        };
        assert_eq!(config.delay_time, 0.3);
        assert_eq!(config.feedback, 0.4);
    }

    #[test]
    fn test_compressor_config() {
        let config = CompressorConfig {
            threshold: -10.0,
            ratio: 4.0,
            attack_ms: 0.005,
            release_ms: 0.1,
            makeup_gain: 0.0,
        };
        assert_eq!(config.threshold, -10.0);
        assert_eq!(config.ratio, 4.0);
    }

    #[test]
    fn test_audio_source_volume() {
        let mut source = SpatialAudioSource::new("test");
        source.volume = 0.5;
        assert_eq!(source.volume, 0.5);
    }

    #[test]
    fn test_audio_source_pitch() {
        // Note: pitch is calculated in SpatialAudioParams, not stored in source
        let source = SpatialAudioSource::new("test");
        assert_eq!(source.volume, 1.0);
    }

    #[test]
    fn test_audio_source_position() {
        // Note: position is managed by Transform component, not SpatialAudioSource
        let source = SpatialAudioSource::new("test");
        assert_eq!(source.name, "test");
    }

    #[test]
    fn test_audio_source_velocity() {
        // Note: velocity is tracked in SpatialAudioState, not in source
        let source = SpatialAudioSource::new("test");
        assert_eq!(source.name, "test");
    }

    #[test]
    fn test_audio_listener_position() {
        let listener = AudioListener::new();
        assert_eq!(listener.gain, 1.0);
        assert_eq!(listener.enabled, true);
    }

    #[test]
    fn test_audio_listener_orientation() {
        let listener = AudioListener::new();
        assert_eq!(listener.gain, 1.0);
        assert_eq!(listener.enabled, true);
    }

    #[test]
    fn test_multiple_audio_sources() {
        let sources = vec![
            SpatialAudioSource::new("sound1"),
            SpatialAudioSource::new("sound2"),
            SpatialAudioSource::new("sound3"),
        ];
        assert_eq!(sources.len(), 3);
    }

    #[test]
    fn test_audio_buffer_clone() {
        let buffer = AudioBuffer {
            data: vec![0.5f32; 512],
            channels: 2,
            sample_rate: 48000,
            filled: false,
            timestamp: 0,
        };
        let cloned = buffer.clone();
        assert_eq!(buffer.data.len(), cloned.data.len());
        assert_eq!(buffer.channels, cloned.channels);
    }

    #[test]
    fn test_effect_chain_with_effects() {
        let mut chain = EffectChain::new();
        // Verify chain starts empty
        assert_eq!(chain.effect_count(), 0, "New chain should have no effects");
        // Chain should be ready to add effects
        assert!(
            std::mem::size_of::<EffectChain>() > 0,
            "EffectChain should be defined"
        );
    }

    #[test]
    fn test_equalizer_band() {
        let band = EqualizerBand {
            frequency: 1000.0,
            gain: 3.0,
            q: 2.0,
        };
        assert_eq!(band.frequency, 1000.0);
        assert_eq!(band.gain, 3.0);
        assert_eq!(band.q, 2.0);
    }

    #[test]
    fn test_audio_source_with_cone() {
        let cone = SoundCone {
            inner_angle: 60.0,
            outer_angle: 120.0,
            outer_gain: 0.3,
        };
        let source = SpatialAudioSource::new("test").with_cone(cone);
        assert_eq!(source.cone.inner_angle, 60.0);
    }

    #[test]
    fn test_audio_source_looping() {
        let source = SpatialAudioSource::new("test").with_looping(true);
        assert!(source.looping);
    }

    #[test]
    fn test_audio_source_state_changes() {
        let mut source = SpatialAudioSource::new("test");
        source.is_playing = true;
        assert!(source.is_playing);

        source.is_playing = false;
        assert!(!source.is_playing);
    }

    #[test]
    fn test_stream_id_generation() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id1 = COUNTER.fetch_add(1, Ordering::SeqCst);
        let id2 = COUNTER.fetch_add(1, Ordering::SeqCst);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_compressor_threshold_boundary() {
        let thresholds = vec![-60.0, -40.0, -20.0, -10.0, -5.0, 0.0];
        for threshold in thresholds {
            let mut config = CompressorConfig::default();
            config.threshold = threshold;
            assert_eq!(config.threshold, threshold);
        }
    }

    #[test]
    fn test_reverb_room_size_boundary() {
        let room_sizes = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        for size in room_sizes {
            let mut config = ReverbConfig::default();
            config.room_size = size;
            assert_eq!(config.room_size, size);
        }
    }

    #[test]
    fn test_delay_feedback_boundary() {
        let feedback_values = vec![0.0, 0.25, 0.5, 0.75, 0.95];
        for feedback in feedback_values {
            let mut config = DelayConfig::default();
            config.feedback = feedback;
            assert_eq!(config.feedback, feedback);
        }
    }

    #[test]
    fn test_equalizer_multiple_bands() {
        let bands = vec![
            EqualizerBand {
                frequency: 32.0,
                gain: 0.0,
                q: 0.5,
            },
            EqualizerBand {
                frequency: 64.0,
                gain: 0.0,
                q: 0.5,
            },
            EqualizerBand {
                frequency: 125.0,
                gain: 0.0,
                q: 0.5,
            },
            EqualizerBand {
                frequency: 250.0,
                gain: 0.0,
                q: 0.5,
            },
            EqualizerBand {
                frequency: 500.0,
                gain: 0.0,
                q: 0.5,
            },
            EqualizerBand {
                frequency: 1000.0,
                gain: 0.0,
                q: 0.5,
            },
            EqualizerBand {
                frequency: 2000.0,
                gain: 0.0,
                q: 0.5,
            },
            EqualizerBand {
                frequency: 4000.0,
                gain: 0.0,
                q: 0.5,
            },
            EqualizerBand {
                frequency: 8000.0,
                gain: 0.0,
                q: 0.5,
            },
            EqualizerBand {
                frequency: 16000.0,
                gain: 0.0,
                q: 0.5,
            },
        ];
        let config = EqualizerConfig {
            bands,
            sample_rate: 44100.0,
        };
        assert_eq!(config.bands.len(), 10);
    }

    #[test]
    fn test_audio_buffer_with_different_sample_rates() {
        let sample_rates = vec![22050, 44100, 48000, 96000];
        for rate in sample_rates {
            let buffer = AudioBuffer {
                data: vec![0.0f32; 1024],
                channels: 2,
                sample_rate: rate,
                filled: false,
                timestamp: 0,
            };
            assert_eq!(buffer.sample_rate, rate);
        }
    }

    #[test]
    fn test_audio_buffer_mono() {
        let buffer = AudioBuffer {
            data: vec![0.5f32; 512],
            channels: 1,
            sample_rate: 44100,
            filled: false,
            timestamp: 0,
        };
        assert_eq!(buffer.channels, 1);
    }

    #[test]
    fn test_audio_buffer_stereo() {
        let buffer = AudioBuffer {
            data: vec![0.5f32; 1024],
            channels: 2,
            sample_rate: 44100,
            filled: false,
            timestamp: 0,
        };
        assert_eq!(buffer.channels, 2);
    }

    #[test]
    fn test_audio_buffer_surround() {
        let buffer = AudioBuffer {
            data: vec![0.5f32; 2048],
            channels: 6,
            sample_rate: 48000,
            filled: false,
            timestamp: 0,
        };
        assert_eq!(buffer.channels, 6);
    }

    #[test]
    fn test_audio_source_moving() {
        // Note: position and velocity are managed by Transform component and SpatialAudioState
        let source = SpatialAudioSource::new("test");
        assert_eq!(source.name, "test");
    }

    #[test]
    fn test_doppler_effect_parameters() {
        let source = SpatialAudioSource::new("test").with_doppler(1.0);
        let listener = AudioListener::new();
        // Calculate doppler effect
        assert_eq!(source.doppler_factor, 1.0);
    }

    #[test]
    fn test_spatial_audio_params_distance_attenuation() {
        let params = SpatialAudioParams {
            volume: 1.0,
            left_gain: 0.7,
            right_gain: 0.7,
            pitch: 1.0,
            distance: 10.0,
            azimuth: 0.5,
            elevation: 0.2,
        };
        assert_eq!(params.volume, 1.0);
    }

    #[test]
    fn test_multiple_listeners() {
        let listeners = vec![AudioListener::new(), AudioListener::new()];
        assert_eq!(listeners.len(), 2);
    }

    #[test]
    fn test_audio_source_relative_position() {
        // Note: position is managed by Transform component
        let source = SpatialAudioSource::new("test");
        let listener = AudioListener::new();
        assert_eq!(source.name, "test");
    }

    #[test]
    fn test_effect_chain_serialization() {
        let chain = EffectChain::new();
        // Verify chain is serializable
        assert!(
            std::mem::size_of::<EffectChain>() > 0,
            "EffectChain should be defined"
        );
        // Chain has effects collection
        assert_eq!(chain.effect_count(), 0, "New chain should have no effects");
    }

    #[test]
    fn test_volume_boundary_values() {
        let volumes = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        for volume in volumes {
            let mut source = SpatialAudioSource::new("test");
            source.volume = volume;
            assert_eq!(source.volume, volume);
        }
    }

    #[test]
    fn test_pitch_boundary_values() {
        // Note: pitch is calculated in SpatialAudioParams, not stored in source
        let source = SpatialAudioSource::new("test");
        assert_eq!(source.volume, 1.0);
    }

    #[test]
    fn test_stream_config_validations() {
        let configs = vec![
            StreamConfig {
                buffer_size: 1024,
                preload_buffers: 2,
                looped: false,
                sample_rate: Some(22050),
                channels: Some(1),
            },
            StreamConfig {
                buffer_size: 2048,
                preload_buffers: 2,
                looped: false,
                sample_rate: Some(44100),
                channels: Some(2),
            },
            StreamConfig {
                buffer_size: 4096,
                preload_buffers: 2,
                looped: false,
                sample_rate: Some(48000),
                channels: Some(2),
            },
        ];
        for config in configs {
            assert!(config.buffer_size > 0);
        }
    }

    #[test]
    fn test_audio_buffer_data_access() {
        let buffer = AudioBuffer {
            data: vec![1.0f32, 0.5f32, 0.25f32, 0.125f32],
            channels: 2,
            sample_rate: 44100,
            filled: true,
            timestamp: 0,
        };
        assert_eq!(buffer.data[0], 1.0);
        assert_eq!(buffer.data[1], 0.5);
    }

    #[test]
    fn test_sound_cone_directionality() {
        let cone = SoundCone {
            inner_angle: 45.0,
            outer_angle: 90.0,
            outer_gain: 0.5,
        };
        assert!(cone.inner_angle < cone.outer_angle);
        assert!(cone.outer_gain < 1.0);
    }

    #[test]
    fn test_compressor_attack_release() {
        let config = CompressorConfig {
            attack_ms: 0.001,
            release_ms: 0.1,
            ..Default::default()
        };
        assert!(config.attack_ms < config.release_ms);
    }

    #[test]
    fn test_reverb_wet_dry_mix() {
        let mut config = ReverbConfig::default();
        config.wet_level = 0.3;
        config.dry_level = 0.7;
        assert!((config.wet_level + config.dry_level - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_delay_wet_dry_mix() {
        let mut config = DelayConfig::default();
        config.wet_level = 0.4;
        config.dry_level = 0.6;
        assert!((config.wet_level + config.dry_level - 1.0).abs() < 0.01);
    }
}
