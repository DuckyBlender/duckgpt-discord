use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    pub detail: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct UsageStats {
    pub uses: u32,
    pub cost: u32,
}

