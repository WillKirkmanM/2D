use hecs::{Entity, World};
use glam::Vec2;

#[derive(Clone, Copy)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

#[derive(Clone)]
pub struct Sprite {
    pub texture_name: String,
    pub width: f32,
    pub height: f32,
}

pub struct RigidBody {
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub mass: f32,
}

pub struct Collider {
    pub width: f32,
    pub height: f32,
    pub is_trigger: bool,
}

pub struct EcsWorld {
    pub world: World,
}

impl EcsWorld {
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }
    
    pub fn create_entity(&mut self) -> Entity {
        self.world.spawn(())
    }
    
    pub fn create_sprite_entity(&mut self, 
                               texture_name: &str, 
                               position: Vec2, 
                               width: f32, 
                               height: f32) -> Entity {
        self.world.spawn((
            Transform {
                position,
                rotation: 0.0,
                scale: Vec2::new(1.0, 1.0),
            },
            Sprite {
                texture_name: texture_name.to_string(),
                width,
                height,
            },
        ))
    }
    
    pub fn add_rigid_body(&mut self, entity: Entity, velocity: Vec2, mass: f32) {
        if let Err(e) = self.world.insert_one(
            entity,
            RigidBody {
                velocity,
                acceleration: Vec2::ZERO,
                mass,
            },
        ) {
            println!("Failed to add RigidBody to entity: {:?}", e);
        }
    }
    
    pub fn add_collider(&mut self, entity: Entity, width: f32, height: f32, is_trigger: bool) {
        if let Err(e) = self.world.insert_one(
            entity,
            Collider {
                width,
                height,
                is_trigger,
            },
        ) {
            println!("Failed to add Collider to entity: {:?}", e);
        }
    }
    
    pub fn update(&mut self) {
        println!("--- Entity Positions ---");
        for (entity, transform) in self.world.query::<&Transform>().iter() {
            println!("Entity {:?} at position ({}, {}), scale: ({}, {})", 
                entity, transform.position.x, transform.position.y,
                transform.scale.x, transform.scale.y);
        }
        
        for (_id, (rigid_body, transform)) in self.world.query_mut::<(&RigidBody, &mut Transform)>() {
            transform.position += rigid_body.velocity;
        }
        
        let mut collisions = Vec::new();
        
        let mut collider_entities = Vec::new();
        for (entity, (transform, collider)) in self.world.query::<(&Transform, &Collider)>().iter() {
            collider_entities.push((entity, transform.position, collider.width, collider.height));
        }
        
        for i in 0..collider_entities.len() {
            for j in (i + 1)..collider_entities.len() {
                let (entity_a, pos_a, width_a, height_a) = collider_entities[i];
                let (entity_b, pos_b, width_b, height_b) = collider_entities[j];
                
                let half_width_a = width_a / 2.0;
                let half_height_a = height_a / 2.0;
                let half_width_b = width_b / 2.0;
                let half_height_b = height_b / 2.0;
                
                let min_x_a = pos_a.x - half_width_a;
                let max_x_a = pos_a.x + half_width_a;
                let min_y_a = pos_a.y - half_height_a;
                let max_y_a = pos_a.y + half_height_a;
                
                let min_x_b = pos_b.x - half_width_b;
                let max_x_b = pos_b.x + half_width_b;
                let min_y_b = pos_b.y - half_height_b;
                let max_y_b = pos_b.y + half_height_b;
                
                if max_x_a > min_x_b && min_x_a < max_x_b && max_y_a > min_y_b && min_y_a < max_y_b {
                    collisions.push((entity_a, entity_b));
                }
            }
        }
        
        for (entity_a, entity_b) in collisions {
            println!("Collision between entities {:?} and {:?}", entity_a, entity_b);
        }
    }
    
    pub fn get_renderables(&self) -> Vec<(Entity, Transform, Sprite)> {
        let mut renderables = Vec::new();
        for (entity, (transform, sprite)) in self.world.query::<(&Transform, &Sprite)>().iter() {
            renderables.push((entity, *transform, sprite.clone()));
        }
        renderables
    }
}