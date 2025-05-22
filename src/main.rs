mod renderer;
mod ecs;
mod input;
mod audio;
mod resources;
mod engine;
mod demo_scene;
mod physics;

fn main() {
    env_logger::init();
    
    if let Err(e) = engine::Engine::run_application("2D Game Engine") {
        eprintln!("Error running application: {}", e);
    }
}