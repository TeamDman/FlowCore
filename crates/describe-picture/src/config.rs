use serde::Deserialize;
use serde::Serialize;

use flow_core_config::IConfig;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct DescribePictureConfig {
    pub preferred_vision_model: Option<String>,
}

impl IConfig for DescribePictureConfig {
    const FILE_SLUG: &'static str = "describe-picture";
}
