// Minimal fixed-timestep game loop helpers
// TODO: implement full deterministic fixed-step loop per design docs

use std::time::Duration;

/// Run a fixed-step update callback for `iterations` steps, sleeping between steps.
/// This is a small placeholder to satisfy module linkage; replace with production-quality logic.
pub fn run_fixed_steps<F: FnMut()>(mut update: F, fixed_step: Duration, iterations: usize) {
    for _ in 0..iterations {
        update();
        std::thread::sleep(fixed_step);
    }
}
