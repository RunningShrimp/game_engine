#[cfg(test)]
mod tests {
    use crate::resources::manager::{Handle, AssetEvent, LoadState};
    use std::sync::{Arc, RwLock};

    // Mock Handle for testing purposes
    fn mock_handle<T: Send + Sync + 'static>(state: LoadState<T>) -> Handle<T> {
        Handle {
            container: Arc::new(crate::resources::manager::AssetContainer {
                state: RwLock::new(state),
            }),
        }
    }

    #[test]
    fn test_asset_loading_simulation() {
        // Simulate the asset server sending a "Loaded" event
        let loaded_handle = mock_handle(LoadState::Loaded(123u32));
        let event = AssetEvent::TextureLoaded(loaded_handle.clone(), 50.0);

        // Verify the event and handle state
        if let AssetEvent::TextureLoaded(h, ms) = event {
            assert!(h.is_loaded());
            assert_eq!(h.get().unwrap(), 123);
            assert_eq!(ms, 50.0);
        } else {
            panic!("Expected TextureLoaded event");
        }
    }

    #[test]
    fn test_asset_failed_simulation() {
        // Simulate the asset server sending a "Failed" event
        let failed_handle: Handle<u32> = mock_handle(LoadState::Failed("File not found".to_string()));
        let event = AssetEvent::TextureFailed(failed_handle.clone(), "File not found".to_string());

        // Verify the event and handle state
        if let AssetEvent::TextureFailed(h, reason) = event {
            assert!(!h.is_loaded());
            assert_eq!(reason, "File not found");
            match &*h.container.state.read().unwrap() {
                LoadState::Failed(r) => assert_eq!(r, "File not found"),
                _ => panic!("Expected Failed state"),
            }
        } else {
            panic!("Expected TextureFailed event");
        }
    }
}
