use ::resource::resource_loader::ResourceLoader;
use resource::resource;
use std::sync::Arc;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Debug, Clone)]
pub struct AudioSource {
    pub bytes: Arc<[u8]>,
}

impl AsRef<[u8]> for AudioSource {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl resource::Resource for AudioSource {}

#[derive(Default)]
pub struct AudioLoader;

impl ResourceLoader for AudioLoader {
    fn load(&self, path: &str) -> Option<Arc<dyn resource::Resource>> {
        let mut bytes = Vec::new();
        let file = File::open(path).unwrap();
        let mut reader = BufReader::new(file);
        let _ = reader.read_to_end(&mut bytes);
        Some(Arc::new(AudioSource {
            bytes: bytes.into(),
        }))
    }

    fn extensions(&self) -> &[&str] {
        &["mp3", "flac", "wav", "oga", "ogg", "spx"]
    }
}
