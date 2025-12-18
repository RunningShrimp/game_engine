#[cfg(test)]
mod tests {
    use crate::ecs::{PointLight, Sprite, Transform};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_entity_creation() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        assert!(world.get_entity(entity).is_some());
    }

    #[test]
    fn test_component_insertion() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        world.entity_mut(entity).insert(Transform::default());
        assert!(world.get::<Transform>(entity).is_some());

        world.entity_mut(entity).insert(Sprite {
            color: [1.0, 1.0, 1.0, 1.0],
            ..Default::default()
        });
        assert!(world.get::<Sprite>(entity).is_some());

        world.entity_mut(entity).insert(PointLight::default());
        assert!(world.get::<PointLight>(entity).is_some());
    }

    #[test]
    fn test_query() {
        let mut world = World::new();
        world.spawn((Transform::default(), Sprite::default()));
        world.spawn((Transform::default(), Sprite::default()));
        world.spawn(Transform::default());

        let mut query = world.query::<(&Transform, &Sprite)>();
        assert_eq!(query.iter(&world).count(), 2);

        let mut query_single = world.query::<&Transform>();
        assert_eq!(query_single.iter(&world).count(), 3);
    }
}
