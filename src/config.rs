#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct Config {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) num_agents: u32,
    pub(crate) random_seed: u64,
    pub(crate) world: WorldConfig,
    pub(crate) agent: AgentConfig,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct WorldConfig {
    pub(crate) decay_rate: f32,
    pub(crate) diffuse_radius: u32,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct AgentConfig {
    pub(crate) speed: f32,
    pub(crate) turning_speed: f32,
    pub(crate) sensor_distance: f32,
    pub(crate) sensor_angle: f32,
    pub(crate) sensor_radius: u32,
    pub(crate) draw_scale: f32,
}

impl Config {
    pub(crate) fn load() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                Self::web_defaults()
            } else {
                Self::load_from_file()
            }
        }
    }

    #[allow(unused)]
    fn web_defaults() -> Self {
        Self {
            width: 720,
            height: 720,
            num_agents: 50000,
            random_seed: 24,
            world: WorldConfig {
                decay_rate: 0.002,
                diffuse_radius: 1,
            },
            agent: AgentConfig {
                speed: 0.2,
                turning_speed: 20.0,
                sensor_distance: 0.06,
                sensor_angle: 25.0,
                sensor_radius: 3,
                draw_scale: 0.0,
            },
        }
    }

    #[allow(unused)]
    fn load_from_file() -> Self {
        serde_json::from_str(
            &std::fs::read_to_string("config.json").expect("Failed to read config file"),
        )
        .expect("Failed to parse config file")
    }
}
