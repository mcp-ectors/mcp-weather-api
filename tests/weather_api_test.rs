

use std::env;
use dotenvy::dotenv;
use exports::wasix::mcp::router::{Content, PromptMessageContent, Role, Value};
use serde_json::json;
use tracing;
use wasix::mcp;
use wasix::mcp::secrets_store::{HostSecret, Secret, SecretValue, SecretsError};
use wasmtime_wasi::{IoView, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime::{Config, Engine, Store};
use wasmtime::component::{bindgen, Component, Linker, Resource};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};
use crate::wasi::logging::logging;
use crate::logging::Level;
const INSTRUCTIONS: &str = "Fetches the current weather \n        for a given location. \n        Call the get_weather tool and pass a json {'location'='input your location here'}, \n        as input. Location can be in different formats:\n        * Latitude and Longitude (Decimal degree) e.g: location=48.8567,2.3508\n        * city name e.g.: location=Paris\n        * US zip e.g.: location=10001\n        * UK postcode e.g: location=SW1\n        * Canada postal code e.g: location=G2J\n        * metar:<metar code> e.g: location=metar:EGLL\n        * iata:<3 digit airport code> e.g: location=iata:DXB\n        * auto:ip IP lookup e.g: location=auto:ip\n        * IP address (IPv4 and IPv6 supported) e.g: location=100.0.0.1\n        * By ID returned from Search API. e.g: location=id:2801268";


bindgen!({
    world: "mcp-secrets",
});

#[derive(Debug, Clone, Copy)]
struct SecretsStore {
    weather_api_key: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct Logging;

struct MyState {
    secrets_store: SecretsStore,
    logging: Logging,
    table: ResourceTable,
    ctx: WasiCtx,
    http: WasiHttpCtx,
}

impl HostSecret for MyState{
    fn drop(&mut self,_rep:wasmtime::component::Resource<Secret>) -> wasmtime::Result<()> {
        self.secrets_store.weather_api_key = "";
        Ok(())
    }
}

impl WasiHttpView for MyState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
}

impl mcp::secrets_store::Host for MyState{
    #[doc = " Gets a single opaque secrets value set at the given key if it exists"]
    fn get(&mut self,_key:wasmtime::component::__internal::String,) -> Result<Resource<Secret>,SecretsError> {
            Ok(Resource::<Secret>::new_own(1))
    }

    fn reveal(&mut self,_s:wasmtime::component::Resource<Secret>,) -> SecretValue {
        SecretValue { secret: self.secrets_store.weather_api_key.to_string().clone() }
    }
}

impl logging::Host for MyState {
    fn log(&mut self,level:logging::Level,context:wasmtime::component::__internal::String,message:wasmtime::component::__internal::String,) -> () {
        match level {
            Level::Trace => tracing::trace!(context, message),
            Level::Debug => tracing::debug!(context, message),
            Level::Info => tracing::info!(context, message),
            Level::Warn => tracing::warn!(context, message),
            Level::Error => tracing::error!(context, message),
            Level::Critical => tracing::error!(context, message),
        }
    }
}

impl IoView for MyState {
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}
impl WasiView for MyState {
    fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
}


#[test]
fn test_weather_api_router() {
    dotenv().ok();
    // Load the wasm file (ensure it's built first)
    let file = "target/wasm32-wasip2/debug/mcp_weather_api.wasm";
    let mut config = Config::default();
    config.async_support(false);

    let weather_api_key = env::var("WEATHER_API_KEY").expect("WEATHER_API_KEY not set in .env");
    let weather_api_key = Box::leak(weather_api_key.into_boxed_str());
    let secrets_store = SecretsStore { weather_api_key };
    let logging = Logging{};
    // Create a Wasmtime engine and store
    let engine = Engine::new(&config).unwrap();
    let wasi = WasiCtxBuilder::new().build();
    let state = MyState {
        secrets_store,
        logging,
        ctx: wasi,
        http: WasiHttpCtx::new(),
        table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, state);
    let component = Component::from_file(&engine, file).unwrap();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker).expect("wasi linker not added");
    wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker).expect("Could not add http to linker");
    mcp::secrets_store::add_to_linker(&mut linker,  |state: &mut MyState| state).expect("Could not link secrets store");
    wasi::logging::logging::add_to_linker(&mut linker, |state: &mut MyState| state).expect("Could not link logging");

    let router = McpSecrets::instantiate(&mut store, &component, &linker);//.unwrap();
    let router = match router {
        Ok(mcp) => mcp,
        Err(err) =>  {eprint!("Error: {:?}",err); Err(err).expect("error")}
    };

    let mcp = router.wasix_mcp_router();
    let name = mcp.call_name(&mut store).unwrap();
    assert_eq!(name, "Weather API Router".to_string());
    let instructions = mcp.call_instructions(&mut store).unwrap();
    assert_eq!(instructions, INSTRUCTIONS.to_string());
    let tools = mcp.call_list_tools(&mut store).unwrap();
    assert_eq!(tools.len(), 1);  // Assuming only 1 tool is added in the implementation
    assert_eq!(tools[0].name, "get_weather");
    let left: serde_json::Value = serde_json::from_str(&tools[0].input_schema.json)
    .expect("failed to parse left JSON");
    let right: serde_json::Value = serde_json::from_str(r#"{
        "type": "object",
        "properties": {
            "location": {
                "type": "string"
            }
        },
        "required": [
            "location"
        ]
    }"#)
        .expect("failed to parse right JSON");
    assert_eq!(left, right);

    // Test the 'call-tool' function
    let location_json = json!({
        "location": "New York"
    });

    let value = Value {
        json: location_json.to_string(),
    };

    let tool_result = mcp.call_call_tool(&mut store, "get_weather", &value);
    let call_tool_result = tool_result.expect("expected a CallToolResult").expect("within another result");
    let contents = call_tool_result.content.clone();
    let content = contents[0].clone();
    let result = match content {
        Content::Text(text_content) => {
            assert!(text_content.text.contains("New York"));
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
