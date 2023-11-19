# DuckGPT Discord Bot

This is a personal Discord bot which can generate responses to images using the [OpenAI API](https://openai.com/). It is built using the [Serenity](https://github.com/serenity-rs/serenity) Discord library. This bot is not intended for public use, but feel free to use it as a reference for your own projects.

## Features

- Supports text and image inputs
- Validates file types for image inputs
- Calculates the cost of API usage based on tokens used
- Provides detailed usage statistics

## Usage

To use the chatbot, simply attach an imag in a designated channel. The bot will generate a response based on the input and reply to the message.

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/DuckyBlender/gpt4v-discord
   ```

2. Copy the example environment file:

   ```bash
   cp .env.example .env
   ```

3. Set the required environment variables:

   - `DISCORD_TOKEN`: Your Discord bot token
   - `OPENAI_TOKEN`: Your OpenAI API token

4. Set the channels in the `constants.rs`` file:

   - `MAX_TOKENS`: The maximum number of tokens the bot can use per message
   - `TESTER_ROLE_ID`: The ID of the role which can use the bot

   - `LOW_QUALITY_CHANNEL_ID`: The ID of the channel for low quality image recognition
   - `HIGH_QUALITY_CHANNEL_ID`: The ID of the channel for high quality image recognition

   - `ALLOY_CHANNEL_ID`: The ID of the channel for Alloy TTS
   - `ECHO_CHANNEL_ID`: The ID of the channel for Echo TTS
   - `FABLE_CHANNEL_ID`: The ID of the channel for Fable TTS
   - `ONYX_CHANNEL_ID`: The ID of the channel for Onyx TTS
   - `NOVA_CHANNEL_ID`: The ID of the channel for Nova TTS
   - `SHIMMER_CHANNEL_ID`: The ID of the channel for Shimmer TTS

5. Run the bot:

   ```bash
   cargo run --release
   ```

## TODO

- [x] Implement typing indicator while the bot generates a response
- [x] Create an embed for the bot's reply
- [x] Improve error handling and error messages
- [x] Major refactor
- [x] More bot configuration options
- [ ] Add support for image generation (image.rs)
- [ ] Reply instead of sending a message
- [ ] Add support for multiple images in a single message

## License

This project is licensed under the [GNU GPLv3](https://choosealicense.com/licenses/gpl-3.0/) license.
