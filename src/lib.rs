
const WEATHER_API_KEY: &str = "WEATHER_API_KEY";
mod bindings {
    use crate::WeatherAPIRouter;
    wit_bindgen::generate!({ 
        generate_all,
     });


    export!(WeatherAPIRouter);
}

use bindings::exports::wasix::mcp::{router::{self, Annotations, CallToolResult, Content::{self, Text}, GetPromptResult, Guest, McpResource, Prompt, PromptError, PromptMessage, PromptMessageContent, PromptMessageRole, ReadResourceResult, ResourceContents, ResourceError, Role, ServerCapabilities, TextContent, TextResourceContents, Tool, ToolError, Value}, secrets_list::{self, SecretsDescription}};
use serde_json::{from_str,Value as JsonValue};
use bindings::wasix::mcp::secrets_store::{get, reveal};
use bindings::wasi::http::{outgoing_handler::handle,types::{Scheme,Fields,OutgoingRequest}};

#[derive(serde::Deserialize)]
struct WeatherResponse {
    message: String,
}

struct WeatherAPIRouter;

impl secrets_list::Guest for WeatherAPIRouter {
    fn list_secrets() -> Vec::<SecretsDescription> {
       vec![SecretsDescription{ 
        name: WEATHER_API_KEY.to_string(), 
        description: "the api key for weatherapi.com".to_string(), 
        required: true }]
    }
}


impl Guest for WeatherAPIRouter{
    // Implement the methods required by the Router trait
    fn name() -> String {
        "Weather API Router".to_string()
    }

    fn instructions() -> String {
        "Fetches the current weather 
        for a given location. 
        Call the get_weather tool and pass a json {'location'='input your location here'}, 
        as input. Location can be in different formats:
        * Latitude and Longitude (Decimal degree) e.g: location=48.8567,2.3508
        * city name e.g.: location=Paris
        * US zip e.g.: location=10001
        * UK postcode e.g: location=SW1
        * Canada postal code e.g: location=G2J
        * metar:<metar code> e.g: location=metar:EGLL
        * iata:<3 digit airport code> e.g: location=iata:DXB
        * auto:ip IP lookup e.g: location=auto:ip
        * IP address (IPv4 and IPv6 supported) e.g: location=100.0.0.1
        * By ID returned from Search API. e.g: location=id:2801268".to_string()
    }

    fn capabilities() -> ServerCapabilities {
        ServerCapabilities {
            prompts: None,
            resources: None,
            tools: Some(router::ToolsCapability {
                list_changed: Some(true),
            }),
        }
    }

    // Implement the rest of the required methods...
    fn list_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "get_weather".to_string(),
                description: "Fetches, retrieves or gets the weather prediction for a 
                specific location. 
                Use the location parameter. Location can be in different formats:
                * Latitude and Longitude (Decimal degree) e.g: location=48.8567,2.3508
                * city name e.g.: location=Paris
                * US zip e.g.: location=10001
                * UK postcode e.g: location=SW1
                * Canada postal code e.g: location=G2J
                * metar:<metar code> e.g: location=metar:EGLL
                * iata:<3 digit airport code> e.g: location=iata:DXB
                * auto:ip IP lookup e.g: location=auto:ip
                * IP address (IPv4 and IPv6 supported) e.g: location=100.0.0.1
                * By ID returned from Search API. e.g: location=id:2801268".to_string(),
                input_schema: Value {
                    json: "{'type': 'object',\n\t'properties': {\n\t\t'location': { 'type': 'string' }\n\t},\n\t'required': ['location']\n}\n".to_string(),
                },
            },
        ]
    }

    fn call_tool(tool_name: String, arguments: Value) -> Result<CallToolResult, ToolError> {
        // Handle calling the tool, returning the appropriate result
        if tool_name == "get_weather" {
            let args: JsonValue = from_str(&arguments.json).expect(format!("Could not read the json arguments: {}",&arguments.json).as_str());

            let location = args
                .get("location")
                .and_then(|v| v.as_str())
                .unwrap(); // Default location

            if location.is_empty() {
                return Ok(CallToolResult{ content: vec![Content::Text(TextContent{text:"you need to provide a location".to_string(),annotations:None})], is_error: Some(true) });
            }

            let weather_key = get(WEATHER_API_KEY);
            let secret = reveal(&weather_key.expect("Could not read WEATHER_API_KEY value"));

            let req = OutgoingRequest::new(Fields::new());
            req.set_scheme(Some(&Scheme::Https)).unwrap();
            req.set_authority(Some("api.weatherapi.com")).unwrap();
            req.set_path_with_query(Some(format!(
                "/v1/current.json?key={}&q={}",
                secret.secret, 
                location).as_str()))
                .unwrap();

            // Perform the API call to the weather api, expecting a URL to come back as the response body
            let weather_req = match handle(req, None) {
                Ok(resp) => {
                    resp.subscribe().block();
                    let response = resp
                        .get()
                        .expect("HTTP request response missing")
                        .expect("HTTP request response requested more than once")
                        .expect("HTTP request failed");
                    if response.status() == 200 {
                        let response_body = response
                            .consume()
                            .expect("failed to get incoming request body");
                        let mut body = Vec::new();
                        let stream = response_body
                            .stream()
                            .expect("failed to get HTTP request response stream");
                        let chunk_size = 1024;
                        loop {
                            let chunk: Vec<u8> = stream.read(chunk_size).unwrap();
                            if chunk.is_empty() {
                                break;
                            }
                            body.extend_from_slice(&chunk);
                        }
                        //let _trailers = wasi::http::types::IncomingBody::finish(response_body);
                        let weather_response: WeatherResponse = serde_json::from_slice(&body).unwrap();
                        weather_response.message
                    } else {
                        format!("HTTP request failed with status code {}", response.status())
                    }
                }
                Err(e) => {
                    format!("Got error when trying to fetch dog: {}", e)
                }
            };

            // Example logic for calling the "WeatherFetcher" tool
            Ok(CallToolResult {
                content: vec![Text(TextContent {
                    text: weather_req,
                    annotations: None,
                })],
                is_error: Some(false),
            })
        } else {
            Err(ToolError::NotFound(format!("Tool {} not found", tool_name)))
        }
    }

    fn list_resources() -> Vec<McpResource> {
        vec![
            McpResource {
                uri: "weather-data-uri".to_string(),
                name: "WeatherDataResource".to_string(),
                description: Some("This router provides weather predictions. Call the WeatherFetcher tool and pass a location, e.g. London, as input.".to_string()),
                mime_type: "application/json".to_string(),
                annotations: None,
            },
        ]
        
    }

    fn read_resource(uri: String) -> Result<ReadResourceResult, ResourceError> {
        if uri == "weather-data-uri" {
            Ok(ReadResourceResult {
                contents: vec![ResourceContents::Text(TextResourceContents {
                    uri: uri.clone(),
                    mime_type: Some("application/json".to_string()),
                    text: "{\"weather\": \"sunny\", \"temperature\":\"15 degrees\"}".to_string(),
                })],
            })
        } else {
            Err(ResourceError::NotFound(format!("Resource at {} not found", uri)))
        }
    }

    fn list_prompts() -> Vec<Prompt> {
        vec![
            Prompt {
                name: "GetWeather".to_string(),
                description: Some("Prompt to get weather information".to_string()),
                arguments: Some(vec![
                    router::PromptArgument {
                        name: "location".to_string(),
                        description: Some("Location to get weather for".to_string()),
                        required: Some(true),
                    },
                ]),
            },
        ]
    }

    fn get_prompt(prompt_name: String) -> Result<GetPromptResult, PromptError> {
        if prompt_name == "GetWeather" {
            Ok(GetPromptResult {
                description: Some("Prompt to fetch weather data".to_string()),
                messages: vec![
                    PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::Text(TextContent{
                            text:"Please provide a location to get the weather.".to_string(), 
                            annotations:  Some(Annotations{ 
                                audience: vec![Role::User].into(), 
                                priority: Some(1.0), 
                                timestamp: Some("now".to_string()) 
                            })
                        }),
                    },
                ],
            })
        } else {
            Err(PromptError::NotFound(format!("Prompt {} not found", prompt_name)))
        }
    }
}


