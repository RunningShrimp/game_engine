//  ECS 组件和系统宏
//
//  提供宏来减少ECS组件和系统定义中的重复代码。

/// 简化Component实现
///
/// 使用此宏可以快速为类型实现Component trait。
///
/// # 示例
///
/// ```rust
/// use game_engine::ecs_macros::impl_component;
///
/// struct Position {
///     x: f32,
///     y: f32,
///     z: f32,
/// }
///
/// impl_component!(Position, bevy_ecs::component::StorageType::Table);
/// ```
#[macro_export]
macro_rules! impl_component {
    ($type:ty, $storage_type:expr) => {
        impl Component for $type {
            const STORAGE_TYPE: StorageType = $storage_type;
        }
    };
}

/// 批量实现Component
///
/// 为多个类型实现Component trait。
///
/// # 示例
///
/// ```rust
/// use game_engine::ecs_macros::impl_components;
///
/// impl_components! {
///     Position: Table,
///     Velocity: Table,
///     Sprite: SparseSet,
/// }
/// ```
#[macro_export]
macro_rules! impl_components {
    (
        $(
            $type:ident: $storage_type:ident
        ),* $(,)?
    ) => {
        $(
            impl_component!($type, StorageType::$storage_type);
        )*
    };
}

/// 简化System定义
///
/// 使用此宏可以快速定义ECS系统。
#[macro_export]
macro_rules! impl_system {
    (
        $(#[$meta:meta])*
        fn $name:ident $(<$($lifetime:lifetime),+ $(,)?>)? (
            $($param:ident : $param_ty:ty),* $(,)?
        ) $body:block
    ) => {
        $(#[$meta])*
        fn $name $(<$($lifetime),+>)?(
            $($param: $param_ty),*
        ) $body
    };
}

/// 资源查询宏
///
/// 简化Resource查询的重复代码。
#[macro_export]
macro_rules! fetch_resource {
    ($world:expr, $res_type:ty) => {
        $world.get_resource::<$res_type>()
            .expect(concat!("Resource ", stringify!($res_type), " not found"))
    };
}

/// 可选资源查询宏
#[macro_export]
macro_rules! try_fetch_resource {
    ($world:expr, $res_type:ty) => {
        $world.get_resource::<$res_type>()
    };
}

#[cfg(test)]
mod tests {
    use bevy_ecs::{component::{Component, StorageType}, prelude::*};

    #[derive(Component)]
    struct TestComponent;

    #[test]
    fn test_impl_component() {
        // 测试宏能够正确编译
        impl_component!(TestComponent, StorageType::Table);
    }

    #[test]
    fn test_fetch_resource() {
        let mut world = World::new();
        world.insert_resource(42i32);

        let value = fetch_resource!(&world, i32);
        assert_eq!(*value, 42);
    }
}
