use std::fmt::Display;

#[derive(Debug, poise::ChoiceParameter)]
pub enum LLMModels {
    #[name = "mistral"]
    Mistral,
    #[name = "caveman"]
    Caveman,
    #[name = "racist"]
    Racist,
    #[name = "lobotomy"]
    Lobotomy,
    #[name = "greentext"]
    Greentext,
}

impl Display for LLMModels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMModels::Mistral => write!(f, "dolphin-mistral"),
            LLMModels::Caveman => write!(f, "caveman-mistral"),
            LLMModels::Racist => write!(f, "racist-mistral"),
            LLMModels::Lobotomy => write!(f, "qwen:0.5b-chat-v1.5-q2_K"),
            LLMModels::Greentext => write!(f, "greentext-mistral"),
        }
    }
}

#[derive(Debug, poise::ChoiceParameter)]
pub enum ImageModels {
    #[name = "SDXLTurbo"]
    SDXLTurbo,
    #[name = "StableCascade (SLOW)"]
    StableCascade,
}

impl Display for ImageModels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageModels::SDXLTurbo => write!(f, "SDXL Turbo"),
            ImageModels::StableCascade => write!(f, "Stable Cascade"),
        }
    }
}
