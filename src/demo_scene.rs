use crate::ecs::EcsWorld;
use glam::Vec2;

pub fn load_demo_scene(world: &mut EcsWorld) {
    let player = world.create_sprite_entity(
        "player",
        Vec2::new(400.0, 300.0),
        100.0,
        100.0
    );
    
    world.add_rigid_body(player, Vec2::new(1.0, 0.0), 1.0);
    
    world.add_collider(player, 100.0, 100.0, false);
    
    for i in 0..5 {
        let enemy = world.create_sprite_entity(
            "enemy",
            Vec2::new(-300.0 + (i as f32 * 150.0), 200.0),
            80.0,
            80.0
        );
        world.add_rigid_body(enemy, Vec2::new(0.0, -0.5), 1.0);
        world.add_collider(enemy, 80.0, 80.0, false);
    }
}