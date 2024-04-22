use vulkan::MsaaSamples;

#[derive(Clone)]
pub struct Config {
    resolution: Resolution,
    fullscreen: bool,
    vsync: Option<bool>,
    msaa: MsaaSamples,
    env: EnvironmentConfig,
}

impl Config {
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    pub fn fullscreen(&self) -> bool {
        self.fullscreen
    }

    pub fn vsync(&self) -> bool {
        self.vsync.unwrap_or(false)
    }

    pub fn msaa(&self) -> MsaaSamples {
        self.msaa
    }

    pub fn env(&self) -> &EnvironmentConfig {
        &self.env
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            resolution: Default::default(),
            fullscreen: false,
            vsync: Some(false),
            msaa: MsaaSamples::S1,
            env: Default::default(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Resolution {
    width: u32,
    height: u32,
}

impl Resolution {
    pub fn width(self) -> u32 {
        self.width
    }

    pub fn height(self) -> u32 {
        self.height
    }
}

impl Default for Resolution {
    fn default() -> Self {
        Resolution {
            width: 1920,
            height: 1080,
        }
    }
}

#[derive(Clone)]
pub struct EnvironmentConfig {
    path: String,
    resolution: Option<u32>,
}

impl EnvironmentConfig {
    const SKYBOX_DEFAULT_PATH: &'static str = "assets/skybox/skybox.hdr";
    const SKYBOX_DEFAULT_RESOLUTION: u32 = 2048;

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn resolution(&self) -> u32 {
        self.resolution.unwrap_or(Self::SKYBOX_DEFAULT_RESOLUTION)
    }
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            path: String::from(Self::SKYBOX_DEFAULT_PATH),
            resolution: None,
        }
    }
}
