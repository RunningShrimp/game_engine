//! SoC检测和功耗管理模块

pub mod detect;
pub mod power;

pub use detect::{SocInfo, SocVendor, detect_soc};
pub use power::{PowerManager, PowerMode, ThermalState};

