use exports::wasix::mcp::router::{self, Annotations, CallToolResult, Content::Text, GetPromptResult, Guest, McpResource, Prompt, PromptMessage, PromptMessageContent, PromptMessageRole, ReadResourceResult, ResourceContents, ResourceError, Role, ServerCapabilities, TextContent, TextResourceContents, Tool, ToolError, Value};



wit_bindgen::generate!({
    // with: {
    //     "wasix:mcp/router@0.0.1": generate,
    // }
    world: "mcp",
});



struct WeatherAPIRouter;



impl Guest for WeatherAPIRouter{
    // Implement the methods required by the Router trait
    fn name() -> String {
        "Weather API Router".to_string()
    }

    fn instructions() -> String {
        "This router provides weather data.".to_string()
    }

    fn capabilities() -> ServerCapabilities {
        ServerCapabilities {
            prompts: Some(router::PromptsCapability { list_changed: Some(true) }),
            resources: Some(router::ResourcesCapability {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            tools: Some(router::ToolsCapability {
                list_changed: Some(true),
            }),
        }
    }

    // Implement the rest of the required methods...
    fn list_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "WeatherFetcher".to_string(),
                description: "Fetches weather data".to_string(),
                input_schema: Value {
                    key: "location".to_string(),
                    data: "string".to_string(),
                },
            },
        ]
    }

    fn call_tool(tool_name: String, arguments: Value) -> Result<CallToolResult, ToolError> {
        // Handle calling the tool, returning the appropriate result
        if tool_name == "WeatherFetcher" {
            // Example logic for calling the "WeatherFetcher" tool
            Ok(CallToolResult {
                content: vec![Text(TextContent {
                    text: format!("Fetching weather data for: {}", arguments.data),
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
                description: Some("Resource containing weather data".to_string()),
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
                    text: "{\"weather\": \"sunny\"}".to_string(),
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

    fn get_prompt(prompt_name: String) -> Result<GetPromptResult, ResourceError> {
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
            Err(ResourceError::NotFound(format!("Prompt {} not found", prompt_name)))
        }
    }
}


export!(WeatherAPIRouter);