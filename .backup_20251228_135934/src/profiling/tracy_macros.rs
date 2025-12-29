//  Tracy Profiler 宏
//
//  提供宏来减少条件编译的重复代码。

/// 简化Tracy特性相关的条件编译
///
/// 使用此宏可以避免在每个方法中重复#[cfg(feature = "tracy")]注解。
#[macro_export]
macro_rules! tracy_enabled {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracy")]
        $($tt)*
    }
}

/// 简化非Tracy特性相关的条件编译
#[macro_export]
macro_rules! tracy_disabled {
    ($($tt:tt)*) => {
        #[cfg(not(feature = "tracy"))]
        $($tt)*
    }
}

/// 执行代码块仅在Tracy启用时
#[macro_export]
macro_rules! if_tracy {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracy")]
        {
            $($tt)*
        }
    }
}

/// 执行代码块仅在Tracy禁用时
#[macro_export]
macro_rules! if_not_tracy {
    ($($tt:tt)*) => {
        #[cfg(not(feature = "tracy"))]
        {
            $($tt)*
        }
    }
}

/// Tracy字段定义宏
#[macro_export]
macro_rules! tracy_field {
    ($vis:vis $name:ident : $ty:ty) => {
        #[cfg(feature = "tracy")]
        $vis $name: $ty
    };
}

/// Tracy忽略值宏（用于未使用的变量）
#[macro_export]
macro_rules! ignore_if_not_tracy {
    ($($val:expr),*) => {
        #[cfg(not(feature = "tracy"))]
        {
            let _ = ($($val),*);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_macro_compilation() {
        // 测试宏能够正确编译
        tracy_enabled!({
            println!("Tracy enabled branch");
        });

        tracy_disabled!({
            println!("Tracy disabled branch");
        });

        if_tracy! {
            println!("If tracy branch");
        }

        if_not_tracy! {
            println!("If not tracy branch");
        }
    }
}
