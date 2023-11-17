# Discord AI Chatbot

This is a Discord chatbot that uses the OpenAI GPT-4 model to generate responses to user messages. The bot can process both text and image inputs and provide relevant and creative replies.

## Features

- Supports text and image inputs
- Validates file types for image inputs
- Calculates the cost of API usage based on tokens used
- Provides detailed usage statistics

## Usage

To use the chatbot, simply attach an image or type a message in a designated channel. The bot will generate a response based on the input and reply to the message.

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

- Implement typing indicator while the bot generates a response
- Add support for multiple images in a single message
- Create an embed for the bot's reply
- Improve error handling and error messages

## License

This project is licensed under the [GNU GPLv3](https://choosealicense.com/licenses/gpl-3.0/) license.
