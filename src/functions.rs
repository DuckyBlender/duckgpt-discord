use crate::ALLOWED_EXTENSIONS;

pub fn is_image_url(url: &str) -> bool {
    let url = match url::Url::parse(url) {
        Ok(url) => url,
        Err(_) => return false,
    };

    url.scheme() == "http" || url.scheme() == "https" && ALLOWED_EXTENSIONS.iter().any(|ext| url.path().ends_with(ext))
}

pub fn calculate_image_token_cost(width: u32, height: u32, detail: &str) -> u32 {
    const LOW_DETAIL_COST: u32 = 85;
    const HIGH_DETAIL_COST_PER_TILE: u32 = 170;
    const ADDITIONAL_COST: u32 = 85;
    const MAX_DIMENSION: u32 = 2048;
    const SCALE_TO: u32 = 768;
    const TILE_SIZE: u32 = 512;

    match detail {
        "low" => LOW_DETAIL_COST,
        "high" => {
            // Scale the image if either dimension is larger than the maximum allowed.
            let (scaled_width, scaled_height) = if width > MAX_DIMENSION || height > MAX_DIMENSION {
                let aspect_ratio = width as f32 / height as f32;
                if width > height {
                    (
                        MAX_DIMENSION,
                        (MAX_DIMENSION as f32 / aspect_ratio).round() as u32,
                    )
                } else {
                    (
                        (MAX_DIMENSION as f32 * aspect_ratio).round() as u32,
                        MAX_DIMENSION,
                    )
                }
            } else {
                (width, height)
            };

            // Further scale the image so that the shortest side is 768 pixels long.
            let (final_width, final_height) = {
                let aspect_ratio = scaled_width as f32 / scaled_height as f32;
                if scaled_width < scaled_height {
                    (SCALE_TO, (SCALE_TO as f32 / aspect_ratio).round() as u32)
                } else {
                    ((SCALE_TO as f32 * aspect_ratio).round() as u32, SCALE_TO)
                }
            };

            // Calculate the number of 512px tiles needed.
            let tiles_across = (final_width as f32 / TILE_SIZE as f32).ceil() as u32;
            let tiles_down = (final_height as f32 / TILE_SIZE as f32).ceil() as u32;
            let total_tiles = tiles_across * tiles_down;

            // Calculate the final token cost.
            total_tiles * HIGH_DETAIL_COST_PER_TILE + ADDITIONAL_COST
        }
        _ => panic!("Invalid detail level: {}", detail),
    }
}

pub fn convert_tokens_to_cost(
    input_tokens: u32,
    output_tokens: u32,
    width: u32,
    height: u32,
    detail_level: &str,
) -> f32 {
    const COST_PER_INPUT_TOKEN: f32 = 0.01 / 1000.0;
    const COST_PER_OUTPUT_TOKEN: f32 = 0.03 / 1000.0;
    let input_cost = input_tokens as f32 * COST_PER_INPUT_TOKEN;
    let output_cost = output_tokens as f32 * COST_PER_OUTPUT_TOKEN;
    let image_cost =
        calculate_image_token_cost(width, height, detail_level) as f32 * COST_PER_OUTPUT_TOKEN;

    // Calculate the total cost
    let total_cost = input_cost + output_cost + image_cost;

    total_cost
}
