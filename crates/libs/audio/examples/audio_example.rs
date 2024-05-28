use std::path::Path;

use audio::audio_source::{AudioLoader, AudioSource};
use resource::resource_mgr::ResourceMgr;
use audio::audio_player::AudioPlayer;

fn main() {
    let mut mgr = ResourceMgr::new();
    mgr.register_loader(AudioLoader::default());
    let loader = mgr.get_asset_loader_with_extension("ogg");
    println!("{}", loader.unwrap().extensions()[0]);
    let resource = mgr.load(&Path::new("assets/audio/breakout_collision.ogg"));
    let binding = resource.unwrap();
    let resource = binding.as_any().downcast_ref::<AudioSource>().unwrap();
    println!("{}", resource.bytes.len());
    AudioPlayer::default().play(resource.clone());
}
