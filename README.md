
# Weather API Router MCP - The FIRST WASIX-MCP Connector - All 200KB of it!!!

This project defines a Weather API Router as an MCP (Model Context Protocol) packaged as a WASM WASI module. The router provides weather data and supports various functionalities such as fetching weather data, reading resources, and handling prompts.

The MCP router is built using the `wasix:mcp.router@0.0.2` standard, allowing it to interact with a WebAssembly runtime using the WASI-P2 (WASI 2.0) interface.

## Features

- **Weather Data Fetching**: The router provides a tool (`get_weather`) to fetch weather data.
- **Resource Management**: The router offers resources which could be extended to provide historical weather data.
- **Prompts**: The router defines a prompt (`GetWeather`) to fetch weather information based on a location.
- **WASM Support**: The router is packaged as a WASM32-WASIP2 module for seamless integration into a WASI runtime. Testing code shows you how to run it.

## Requirements

- Rust (preferably the latest stable version)
- WASI runtime (such as Wasmtime) for running WebAssembly modules.
- Cargo (Rust's build tool) to build and test the module.

## Installation

1. Clone the repository to your local machine.

    ```bash
    git clone https://github.com/your-repository/weather-api-router-mcp.git
    cd weather-api-router-mcp
    ```

2. Build the WASM module with the following command:

    ```bash
    cargo build --target wasm32-wasip2
    ```

3. Once the build is complete, the WASM module will be located in `target/wasm32-wasip2/debug/mcp_weather_api.wasm`.

4. Build the 200K WASM module for production with the following command [`target/wasm32-wasip2/release/mcp_weather_api.wasm`]:

    ```bash
    cargo build --target wasm32-wasip2 --release
    ```

## Running Tests

Copy the .env-sample into .env and go to [weatherapi.com](https://www.weatherapi.com/) to get your WEATHER_API_KEY.

To ensure that the Weather API Router works correctly, you can run the tests:

```bash
cargo test
```

The test will verify the router's functionalities including:

- Fetching weather data with the `get_weather` tool.
- Retrieving resources like weather data [demo only].
- Handling prompts like `GetWeather` [demo only].

## MCP Router Functions

The router implements the following functionalities:

1. **`name()`**: Returns the name of the router.
2. **`instructions()`**: Returns a description of what the router does.
3. **`capabilities()`**: Returns the server's capabilities, such as supported tools, resources, and prompts.
4. **`list_tools()`**: Lists the tools available in the router, e.g., `WeatherFetcher`.
5. **`call_tool()`**: Executes a tool by its name with given arguments and returns the result.
6. **`list_resources()`**: Lists the available resources, such as weather data.
7. **`read_resource()`**: Fetches the contents of a specified resource.
8. **`list_prompts()`**: Lists the available prompts, such as `GetWeather`.
9. **`get_prompt()`**: Fetches the details of a specified prompt.

## Example Usage

After building the WASM module, you can interact with it via a Wasmtime runtime. The `test_weather_api_router()` function demonstrates how the router can be tested, including fetching weather data, reading resources, and handling prompts.

```rust
// Example usage of calling the Weather API Router from a WASI runtime:
let router = McpSecrets::instantiate(&mut store, &component, &linker);
let tools = router.call_list_tools(&mut store).unwrap();
```

## Contribution

If you want to contribute to the development of this project, feel free to fork the repository and submit a pull request. Please make sure to write tests for any new features or bug fixes.

## License

This project is licensed under the MIT License.
