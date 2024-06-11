use asset::asset_mgr::AssetMgr;
use audio::audio_player::AudioPlayer;
use audio::audio_source::{AudioLoader, AudioSource};
use std::path::Path;

fn main() {
    AssetMgr::register_loader(AudioLoader::default());
    let resource = AssetMgr::load(&Path::new("assets/audio/Windless Slopes.ogg"));
    let binding = resource.unwrap();
    let resource = binding.as_any().downcast_ref::<AudioSource>().unwrap();
    let player = AudioPlayer::default();
    player.play(resource);
    std::thread::sleep(std::time::Duration::from_secs(100));
}
