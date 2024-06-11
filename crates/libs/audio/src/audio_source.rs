use ::asset::asset_loader::AssetLoader;
use asset::asset;
use std::any::Any;
use std::io::Cursor;
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

impl asset::Asset for AudioSource {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait Decodable: Send + Sync + 'static {
    type DecoderItem: rodio::Sample + Send + Sync;
    type Decoder: rodio::Source + Send + Iterator<Item = Self::DecoderItem>;

    fn decoder(&self) -> Self::Decoder;
}

impl Decodable for AudioSource {
    type DecoderItem = <rodio::Decoder<Cursor<AudioSource>> as Iterator>::Item;
    type Decoder = rodio::Decoder<Cursor<AudioSource>>;

    fn decoder(&self) -> Self::Decoder {
        rodio::Decoder::new(Cursor::new(self.clone())).unwrap()
    }
}

#[derive(Default)]
pub struct AudioLoader;

impl AssetLoader for AudioLoader {
    fn load(&self, path: &str) -> Option<Arc<dyn asset::Asset>> {
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
