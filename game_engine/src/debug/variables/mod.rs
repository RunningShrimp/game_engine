// 变量监视模块
//
// 提供变量的查看、监视和修改功能

pub mod monitor;

pub use monitor::{
    Scope, ScopeKind, Variable, VariableFormatter, VariableMonitor, VariableReference,
    VariableStats, VariableType, WatchItem,
};
