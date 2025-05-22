use crate::physics::PhysicsSystem;
use crate::renderer::Renderer;
use crate::{demo_scene::load_demo_scene, };
use crate::input::InputManager;
use crate::audio::AudioSystem;
use crate::resources::ResourceManager;
use crate::ecs::EcsWorld;

use std::sync::Arc;
use std::mem::ManuallyDrop;
use winit::{
    application::ApplicationHandler, error::EventLoopError, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowAttributes, WindowId}
};

struct EngineApp<'a> {
    engine: Option<Engine<'a>>,
    window_title: String,
    window_storage: Option<ManuallyDrop<Arc<Window>>>,
    ecs_world: Option<EcsWorld>,
}

impl<'a> EngineApp<'a> {
    fn new(title: String) -> Self {
        Self {
            engine: None,
            window_title: title,
            window_storage: None,
            ecs_world: None,
        }
    }
}

impl<'a> ApplicationHandler<()> for EngineApp<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.engine.is_none() {
            let window_attributes = WindowAttributes::default()
                .with_title(&self.window_title)
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Failed to create window"),
            );

            let window_for_renderer = window.clone();
            self.window_storage = Some(ManuallyDrop::new(window_for_renderer));
            
            let window_arc = unsafe { ManuallyDrop::take(self.window_storage.as_mut().unwrap()) };
            let mut renderer = futures::executor::block_on(
                Renderer::new(window_arc.clone())
            );

            let white_pixels = create_solid_texture(255, 255, 255, 255, 32, 32);
            let red_pixels = create_solid_texture(255, 0, 0, 255, 32, 32);
            
            renderer.load_texture("player", &white_pixels, 32, 32);
            renderer.load_texture("enemy", &red_pixels, 32, 32);

            let mut world = EcsWorld::new();
            
            load_demo_scene(&mut world);

            let engine_instance = Engine {
                window_title: self.window_title.clone(),
                window: Some(window),
                renderer: Some(renderer),
                input_manager: InputManager::new(),
                physics_system: PhysicsSystem::new(),
                audio_system: AudioSystem::new(),
                resource_manager: ResourceManager::new(),
                ecs_world: world,
            };
            
            self.engine = Some(engine_instance);

            let _ = ManuallyDrop::new(window_arc);
            log::info!("Engine initialized and window created.");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(engine) = self.engine.as_mut() {
            engine.input_manager.process_window_event(&event);

            match event {
                WindowEvent::CloseRequested => {
                    log::info!("Close requested. Exiting.");
                    event_loop.exit();
                }
                WindowEvent::Resized(new_size) => {
                    if new_size.width > 0 && new_size.height > 0 {
                        log::info!("Window resized to: {:?}", new_size);
                        if let Some(renderer) = &mut engine.renderer {
                            renderer.resize(new_size.width, new_size.height);
                        }
                        engine.window.as_ref().unwrap().request_redraw();
                    }
                }
                WindowEvent::RedrawRequested => {
                    engine.update();
                    engine.render();
                }
                _ => { /* Other window events processed by input_manager or ignored */ }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(engine) = &self.engine {
            if let Some(window) = &engine.window {
                window.request_redraw();
            }
        }
    }
    
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        log::info!("Application exiting.");
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        let _ = (event_loop, cause);
    }
    
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }
    
    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }
    
    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
    
    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
    
}

impl<'a> Drop for EngineApp<'a> {
    fn drop(&mut self) {
        if let Some(mut window) = self.window_storage.take() {
            unsafe {
                ManuallyDrop::drop(&mut window);
            }
        }
    }
}

pub struct Engine<'a> {
    window_title: String,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer<'a>>,
    input_manager: InputManager,
    physics_system: PhysicsSystem,
    audio_system: AudioSystem,
    resource_manager: ResourceManager,
    ecs_world: EcsWorld,
}
impl<'a> Engine<'a> {
    pub async fn new(window: Arc<Window>, window_title: String) -> Self {
        let mut renderer = Renderer::new(window.clone()).await;

    const WHITE_PIXEL: [u8; 4] = [255, 255, 255, 255];
    renderer.load_texture("player", &WHITE_PIXEL, 1, 1);
    renderer.load_texture("enemy",  &WHITE_PIXEL, 1, 1);
        
        let white_texture = create_solid_texture(255, 255, 255, 255, 32, 32);
        let red_texture = create_solid_texture(255, 0, 0, 255, 32, 32);
        
        renderer.load_texture("player", &white_texture, 32, 32);
        renderer.load_texture("enemy", &red_texture, 32, 32);
        
        Self {
            window_title,
            window: Some(window),
            renderer: Some(renderer),
            input_manager: InputManager::new(),
            physics_system: PhysicsSystem::new(),
            audio_system: AudioSystem::new(),
            resource_manager: ResourceManager::new(),
            ecs_world: EcsWorld::new(),
        }
    }

    pub fn run_application(title: &str) -> Result<(), EventLoopError> {
        log::info!("Starting engine application: {}", title);
        
        let event_loop = EventLoop::new().expect("Failed to create event loop");
        let mut app = EngineApp::new(title.to_string());
        
        event_loop.run_app(&mut app)
    }

    fn update(&mut self) {
        self.ecs_world.update();
        self.physics_system.update();
    }

    fn render(&mut self) {
        if let Some(renderer) = &mut self.renderer {
            renderer.render(&self.ecs_world);
        }
    }
}

fn create_solid_texture(r: u8, g: u8, b: u8, a: u8, width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for _ in 0..(width * height) {
        data.push(r);
        data.push(g);
        data.push(b);
        data.push(a);
    }
    data
}