
# Implementing ReAct Agentic Pattern from Scratch in Rust

[中文(Chinese)](./README-cn.md)

Using frameworks like ***Langchain*** or ***LlamaIndex***, it's easy to build various **AI Agents**. However, these frameworks often hide much of the underlying logic, which makes building agents with them feel both ***familiar and unfamiliar*** at the same time.

So, in order to understand the underlying mechanisms of agents and to gain a deeper technical insight, we will implement a "***ReAct Agentic Pattern Agent***" from scratch.

Unlike many blog examples that use the "**LLM+Calculator**" approach, our agent will perform meaningful function calls with real-world utility, making this implementation more applicable to real development scenarios. This agent accomplishes the following:

**Given a location by the user, the agent outputs the current weather for that area.**

## Prerequisites

### 1. API Key & Base URL

Since **OpenAI** is not accessible in mainland China, I used **Alibaba DashScope** and **OpenRouter** in OpenAI-compatible mode for development and testing.

You’ll need to create a `.env` file in the root directory of your project with the following keys: "**API Key**", "**base url**", and "**model name**".

```ini
OPENAI_API_KEY="sk-...."                           # dashcope/openrouter api key
OPENAI_BASE_URL="https://..../api/v1"              # proper base url
LLM_MODEL="..."                                    # model name to use
```

Example for **OpenRouter**:

```ini
OPENAI_API_KEY="sk-or-v1-052d2a82....d8611631"
OPENAI_BASE_URL="https://openrouter.ai/api/v1"
LLM_MODEL="meta-llama/llama-3.3-8b-instruct:free"
```

Example for **Alibaba DashScope**:

```ini
OPENAI_API_KEY="sk-dce6c....6dcb8"
OPENAI_BASE_URL="https://dashscope.aliyuncs.com/compatible-mode/v1"
LLM_MODEL="qwen-plus"
```

### 2. OpenCageData Geocoding API Key

This API provides the ***latitude and longitude*** for a given location. You can register and obtain a Geocoding API Key at https://opencagedata.com/. It allows 2,500 free API calls per day, which is sufficient for most purposes.

You can find your API key in the [OpenCage Account Dashboard](https://opencagedata.com/dashboard#geocoding).

### 3. OpenWeatherMap API Key

This API fetches current weather based on ***latitude and longitude***. Register and log in at [OpenWeatherMap](https://openweathermap.org/), then create an API key at https://home.openweathermap.org/api_keys. You get 1,000 free API calls per day.

After completing the steps above, your `.env` file should look like this:

```ini
OPENAI_API_KEY="sk-or-v1-052d2a....cd8611631"
OPENAI_BASE_URL="https://openrouter.ai/api/v1"
LLM_MODEL="meta-llama/llama-3.3-8b-instruct:free"

OPENCAGEDATA_API_KEY="cd90a...b2d9995"
OPENWEATHERMAP_API_KEY="f8ec...477"
```

## Build and Run

```sh
cargo run -- Paris
```

If everything goes well, you'll see output like the following, ending with the final answer:

```sh
prompt: What is the weather like in Beijing today?

Paused: {
  "state": "pause",
  "thought": "To find out the current weather in Beijing, I first need to get the latitude and longitude of Beijing, then use the get_weather tool with these coordinates.",
  "action": {
    "tool": "get_geo_location",
    "input": {
      "city": "Beijing"
    }
  }
}

... [tool execution logs, weather info, and final answer]

Final answer:
------------------------------------------------------------
The current weather in Beijing is 27.99°C, with the unit of measurement being Celsius.
```
