# Implementing ReAct Agentic Pattern from Scratch in Rust

基于 ***Langchain*** 或者 ***LlamaIndex* ** 等frameworks，可以很容易地实现各种**AI Agent**, 但是，这些frameworks通常是隐藏很多底层逻辑，这让我们在使用这些frameworks进行AI Agent的实现过程中，有种"***既熟悉又陌生***"的感觉。

所以，为了知道Agent的底层逻辑，进而更深入地了解Agent的技术，我们从零来实现一个"***ReAct Agentic Pattern Agent***".

这里并没有采用其他博客中的"**LLM+Calculator**"的方式，而且让Agent所使用的每个Function Call都能完成实际的功能，让整个实现更贴近实际开发情况。该Agent完成如下功能：

**用户输入地名， Agent输出该地区此时的天气情况**







## 运行前准备

#### 1. API Key & Base URL

因为目前**OpenAI**无法在国内正常访问，所以，我使用了**Alibaba DashScope**和**OpenRouter**的OpenAI 兼容模式进行的开发与测试。

这就需要在项目的root目录下创建.env, 并填写相关的"**API Key**", "**base url**" 和 "**model name**"

```ini
OPENAI_API_KEY="sk-...."                           # dashcope/openrouter api key
OPENAI_BASE_URL="https://..../api/v1"              # proper base url
LLM_MODEL="..."                                    # model name to use
```



基于**OpenRouter**的例子

```ini
OPENAI_API_KEY="sk-or-v1-052d2a82....d8611631"
OPENAI_BASE_URL="https://openrouter.ai/api/v1"
LLM_MODEL="meta-llama/llama-3.3-8b-instruct:free"
```

基于**Alibaba DashScope**的例子

```ini
OPENAI_API_KEY="sk-dce6c....6dcb8"
OPENAI_BASE_URL="https://dashscope.aliyuncs.com/compatible-mode/v1"
LLM_MODEL="qwen-plus"
```

#### 2. OpenCageData Geocoding API Key

这个API能够查询到某个地区所对应的“***经纬度***", 你可以从 https://opencagedata.com/ 进行注册并获取到 Geocoding API Key，它每日有2,500个免费的API调用额度，足够你用。

在[OpenCage Account Dashboard](https://opencagedata.com/dashboard#geocoding)中可以查看Geocoding API Key。




#### 3. OpenWeatherMap API Key

通过"***经纬度***"获取对应地区的当前天气情况。注册并登录[OpenWeatherMap](https://openweathermap.org/)， 在https://home.openweathermap.org/api_keys 页面创建API Key，每日有1,000的免费API调用额度。



做完上面的准备工作后，完整的.env应该长这样：

```ini
OPENAI_API_KEY="sk-or-v1-052d2a....cd8611631"
OPENAI_BASE_URL="https://openrouter.ai/api/v1"
LLM_MODEL="meta-llama/llama-3.3-8b-instruct:free"

OPENCAGEDATA_API_KEY="cd90a...b2d9995"
OPENWEATHERMAP_API_KEY="f8ec...477"

```



## 编译并运行程序

```sh
cargo run -- Paris
```

如果一切顺利，通常会看到类似如下的输出，并在最后看到最终答案。
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

