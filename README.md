# DuckGPT Discord Bot

## Description

This is a AI Discord bot which currently supports Mistral. It requires Ollama to run in the background. This bot is mainly for fun and learning purposes. It is probably not very useful for anything else.

If you want to test out the bot here is a link to a discord server: https://discord.gg/tFzrEesfNJ

## Features
- Supports variations of Mistral (and three custom prompt models)
- Supports Tinyllama

## Todo

- [ ] More models
- [ ] Image recognition

## Prerequisites

- Ollama (so at least 8GB of RAM)
- Rust

## Installation

1. Clone this repository: `git clone https://github.com/DuckyBlender/duckgpt`
2. Navigate to the cloned repository: `cd duckgpt`
3. Install the caveman and racist model model:
4. Install Ollama following the instructions on its [official website](https://ollama.ai/).
5. Download the following models and create custom models

bash
```
ollama pull dolphin-mistral
ollama pull tinyllama
ollama pull tinyllama:1.1b-chat-v0.6-q2_K
ollama create caveman-mistral -f ./custom_models/caveman/Modelfile
ollama create racist-mistral -f ./custom_models/racist/Modelfile
ollama create greentext-mistral -f ./custom_models/greentext/Modelfile
```

One-liner:
bash
```
ollama pull dolphin-mistral && ollama pull tinyllama && ollama pull tinyllama:1.1b-chat-v0.6-q2_K && ollama create caveman-mistral -f ./custom_models/caveman/Modelfile && ollama create racist-mistral -f ./custom_models/racist/Modelfile && ollama create greentext-mistral -f ./custom_models/greentext/Modelfile
```

## Usage

1. Set the .env from the .env.example
2. Make sure ollama is running in the background
3. Run the bot with `cargo run --release`

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License

[MIT](https://choosealicense.com/licenses/mit/)
