use audio::audio_player::AudioPlayer;
use audio::audio_source::{AudioLoader, AudioSource};
use resource::resource_mgr::ResourceMgr;
use std::path::Path;

fn main() {
    ResourceMgr::register_loader(AudioLoader::default());
    let resource = ResourceMgr::load(&Path::new("assets/audio/Windless Slopes.ogg"));
    let binding = resource.unwrap();
    let resource = binding.as_any().downcast_ref::<AudioSource>().unwrap();
    let player = AudioPlayer::default();
    player.play(resource);
    std::thread::sleep(std::time::Duration::from_secs(1));
    player.pause();
    std::thread::sleep(std::time::Duration::from_secs(10));
    player.resume(resource);
    std::thread::sleep(std::time::Duration::from_secs(10000));
}