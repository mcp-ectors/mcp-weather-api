

use exports::wasix::mcp::router::{Content, PromptMessageContent, Role, Value};
use wasmtime_wasi::{IoView, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime::{Config, Engine, Store};
use wasmtime::component::{bindgen, Linker, Component};


bindgen!({
    world: "mcp",
});


struct MyState {
    table: ResourceTable,
    ctx: WasiCtx,
}

impl IoView for MyState {
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}
impl WasiView for MyState {
    fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
}

#[test]
fn test_weather_api_router() {
    // Load the wasm file (ensure it's built first)
    let file = "target/wasm32-wasip2/debug/mcp_weather_api.wasm";
    let mut config = Config::default();
    config.async_support(false);


    // Create a Wasmtime engine and store
    let engine = Engine::new(&config).unwrap();
    let wasi = WasiCtxBuilder::new().build();
    let state = MyState {
        ctx: wasi,
        table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, state);
    let component = Component::from_file(&engine, file).unwrap();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker).expect("wasi linker not added");

    let router = Mcp::instantiate(&mut store, &component, &linker);//.unwrap();
    let router = match router {
        Ok(mcp) => mcp,
        Err(err) =>  {eprint!("Error: {:?}",err); Err(err).expect("error")}
    };

    let mcp = router.wasix_mcp_router();
    let name = mcp.call_name(&mut store).unwrap();
    assert_eq!(name, "Weather API Router".to_string());
    let instructions = mcp.call_instructions(&mut store).unwrap();
    assert_eq!(instructions, "This router provides weather data.".to_string());
    let tools = mcp.call_list_tools(&mut store).unwrap();
    assert_eq!(tools.len(), 1);  // Assuming only 1 tool is added in the implementation
    assert_eq!(tools[0].name, "WeatherFetcher");

    // Test the 'call-tool' function
    let value = Value {
        key: "location".to_string(),
        data: "New York".to_string(),
    };
    let tool_result = mcp.call_call_tool(&mut store, "WeatherFetcher", &value).unwrap().unwrap();
    let contents = tool_result.content.clone();
    let content = contents[0].clone();
    let result = match content {
        Content::Text(text_content) => {
            assert_eq!(text_content.text, "Fetching weather data for: New York");
            Ok(())
        }
        _ => Err("Not right content")
    };
    assert!(!result.is_err());


    // Test the 'list-resources' function
    let resources = mcp.call_list_resources(&mut store).unwrap();
    assert_eq!(resources.len(), 1); // Assuming only 1 resource is available
    let resource = resources[0].clone();
    assert_eq!(resource.name, "WeatherDataResource");

    

    // Test the 'read-resource' function
    let read_result = mcp.call_read_resource(&mut store, "weather-data-uri").unwrap();
    assert!(!read_result.unwrap().contents.is_empty());

    // Test the 'list-prompts' function
    let prompts = mcp.call_list_prompts(&mut store).unwrap();
    assert_eq!(prompts.len(), 1); // Assuming only 1 prompt
    let prompt = prompts[0].clone();
    assert_eq!(prompt.name, "GetWeather");

    // Test the 'get-prompt' function
    let prompt_result = mcp.call_get_prompt(&mut store, "GetWeather").unwrap();
    assert_eq!(prompt_result.clone().unwrap().messages.len(), 1);
    let prompt = prompt_result.unwrap().messages[0].clone();
    let content = prompt.content;
    let content_result = match content {
        PromptMessageContent::Text(text_content) => {
                assert_eq!(text_content.text,"Please provide a location to get the weather.".to_string());
                let annot = text_content.annotations.unwrap();
                let audience = annot.audience;
                assert_eq!(audience.unwrap()[0],Role::User);
                let priority = annot.priority;
                assert_eq!(priority.unwrap(), 1.0);
                let timestamp = annot.timestamp;
                assert_eq!(timestamp.unwrap(),"now".to_string());
                Ok("got it")
            }
        PromptMessageContent::Image(_) => Err("Not a text"),
        PromptMessageContent::McpResource(_) => Err("Not a text"),
    };
    assert!(content_result.is_ok());

    
}
