# DuckGPT Discord Bot

## Description

DuckGPT is a discord bot that uses the Ollama API to generate text. It supports the Mistral model and three custom models. It also has a custom comfyui-rs library which the bot can send image requests to.
If you want to test out the bot here is a link to a discord server: <https://discord.gg/tFzrEesfNJ>

## Disclaimer

This code is a complete mess. This is purely for educational purposes and should not be used in a production environment.

## Features

- Supports variations of Mistral (and three custom prompt models)
- Supports Tinyllama
- Supports "cloning" images. (generate a text prompt from an image and then generate an image from that text prompt)

## Todo

- [x] Image generation
- [x] More image generation options (normal SDXL, etc.)
- [ ] More /img arguments (size, steps, etc.)
- [ ] Handle long responses (or limit the response length to 4096 characters)
- [ ] Dynamic message editing (make sure not to get rate limited tho)
- [x] Image recognition (LLaVa)
- [ ] Release the comfyui-rs library as a crate on crates.io

## Prerequisites

- Ollama (at least 8GB of RAM because we're running a 7B Q4 model)
- Rust

## Installation

1. Clone this repository: `git clone https://github.com/DuckyBlender/duckgpt-discord`
2. Navigate to the cloned repository: `cd duckgpt-discord`
3. Install the caveman and racist model model:
4. Install Ollama following the instructions on its [official website](https://ollama.ai/).
5. Download the following models and create custom models

```bash
ollama pull dolphin-mistral
ollama pull tinyllama
ollama pull qwen:0.5b-chat-v1.5-q2_K
ollama pull llava:7b
ollama create caveman-mistral -f ./custom_models/caveman/Modelfile
ollama create racist-mistral -f ./custom_models/racist/Modelfile
ollama create greentext-mistral -f ./custom_models/greentext/Modelfile
```

One-liner:

```bash
ollama pull dolphin-mistral && ollama pull tinyllama && ollama pull qwen:0.5b-chat-v1.5-q2_K && ollama pull llava:7b && ollama create caveman-mistral -f ./custom_models/caveman/Modelfile && ollama create racist-mistral -f ./custom_models/racist/Modelfile && ollama create greentext-mistral -f ./custom_models/greentext/Modelfile
```

## Usage

1. Set the .env from the .env.example
2. Make sure ollama is running in the background
3. Run the bot with `cargo run --release`

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License

[MIT](https://choosealicense.com/licenses/mit/)
