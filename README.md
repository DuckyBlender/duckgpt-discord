# Discord GPT4V Bot

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

4. Run the bot:

   ```bash
   cargo run --release
   ```

## TODO

- [x] Implement typing indicator while the bot generates a response
- [x] Create an embed for the bot's reply
- [x] Improve error handling and error messages
- [ ] Add support for multiple images in a single message
- [ ] More bot configuration options

## License

This project is licensed under the [GNU GPLv3](https://choosealicense.com/licenses/gpl-3.0/) license.
