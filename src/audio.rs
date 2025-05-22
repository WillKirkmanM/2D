use rodio::{OutputStream, Sink};
use std::collections::HashMap;

pub struct AudioSystem {
    _stream: Option<OutputStream>,
    sink: Option<Sink>,
    sound_effects: HashMap<String, Vec<u8>>,
}

impl AudioSystem {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).ok();
        
        Self {
            _stream: Some(stream),
            sink,
            sound_effects: HashMap::new(),
        }
    }
    
    pub fn load_sound(&mut self, name: &str, path: &str) {
        let file = std::fs::read(path).expect("Failed to read audio file");
        self.sound_effects.insert(name.to_string(), file);
    }
    
    pub fn play_sound(&self, name: &str) {
        if let Some(data) = self.sound_effects.get(name) {
            if let Some(sink) = &self.sink {
                let cursor = std::io::Cursor::new(data.clone());
                let source = rodio::Decoder::new(cursor).unwrap();
                sink.append(source);
            }
        }
    }
}