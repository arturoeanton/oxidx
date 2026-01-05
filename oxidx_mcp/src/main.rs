//! # OxidX MCP Server
//!
//! Model Context Protocol (MCP) server that exposes OxidX code generation
//! tools to AI assistants like Claude Desktop.
//!
//! ## Usage
//!
//! Add to claude_desktop_config.json:
//! ```json
//! {
//!   "mcpServers": {
//!     "oxidx": {
//!       "command": "/path/to/oxidx-mcp"
//!     }
//!   }
//! }
//! ```

use anyhow::{anyhow, Result};
use oxidx_codegen::generate_view;
use oxidx_core::schema::ComponentNode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Command;

/// MCP Protocol Version
const PROTOCOL_VERSION: &str = "2024-11-05";

/// JSON-RPC Request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

/// JSON-RPC Response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC Error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl JsonRpcResponse {
    fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    eprintln!("[oxidx-mcp] Server starting...");

    for line in stdin.lock().lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        eprintln!("[oxidx-mcp] Received: {}", &line[..line.len().min(100)]);

        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => {
                if let Some(response) = handle_request(request) {
                    let response_json = serde_json::to_string(&response)?;
                    eprintln!(
                        "[oxidx-mcp] Sending: {}",
                        &response_json[..response_json.len().min(100)]
                    );
                    writeln!(stdout, "{}", response_json)?;
                    stdout.flush()?;
                }
                // If None, it's a notification - no response needed
            }
            Err(e) => {
                eprintln!("[oxidx-mcp] Parse error: {}", e);
                let response =
                    JsonRpcResponse::error(Value::Null, -32700, format!("Parse error: {}", e));
                let response_json = serde_json::to_string(&response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
        }
    }

    eprintln!("[oxidx-mcp] Server shutting down.");
    Ok(())
}

/// Handle a single JSON-RPC request
/// Returns None for notifications (no response needed)
fn handle_request(request: JsonRpcRequest) -> Option<JsonRpcResponse> {
    let id = request.id.clone();
    let is_notification = id.is_none() || request.method.starts_with("notifications/");

    match request.method.as_str() {
        "initialize" => Some(handle_initialize(
            id.unwrap_or(Value::Null),
            &request.params,
        )),
        "initialized" | "notifications/initialized" => {
            // Notification, no response needed
            eprintln!("[oxidx-mcp] Received initialized notification");
            None
        }
        "tools/list" => Some(handle_tools_list(id.unwrap_or(Value::Null))),
        "tools/call" => Some(handle_tools_call(
            id.unwrap_or(Value::Null),
            &request.params,
        )),
        "ping" => Some(JsonRpcResponse::success(
            id.unwrap_or(Value::Null),
            json!({}),
        )),
        _ => {
            eprintln!("[oxidx-mcp] Unknown method: {}", request.method);
            if is_notification {
                // Don't respond to unknown notifications
                None
            } else {
                Some(JsonRpcResponse::error(
                    id.unwrap_or(Value::Null),
                    -32601,
                    format!("Method not found: {}", request.method),
                ))
            }
        }
    }
}

/// Handle initialize request
fn handle_initialize(id: Value, _params: &Value) -> JsonRpcResponse {
    JsonRpcResponse::success(
        id,
        json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "oxidx-mcp",
                "version": "0.1.0"
            }
        }),
    )
}

/// Handle tools/list request
fn handle_tools_list(id: Value) -> JsonRpcResponse {
    // Define all supported components in one place for dynamic discovery
    let supported_components = vec![
        "VStack",
        "HStack",
        "ZStack",
        "Button",
        "Label",
        "Input",
        "Image",
        "Chart",
        "PieChart",
        "BarChart",
        "LineChart",
        "Checkbox",
        "Radio",
        "Slider",
        "Toggle",
        "ScrollView",
        "SplitView",
        "TreeView",
        "Grid",
        "ListBox",
        "ComboBox",
        "GroupBox",
        "RadioGroup",
        "ProgressBar",
        "TextArea",
        "CodeEditor",
        "Calendar",
        "Modal",
        "Alert",
        "Confirm",
    ];

    JsonRpcResponse::success(
        id,
        json!({
            "tools": [
                {
                    "name": "generate_oxid_ui",
                    "description": "Generate Rust UI code from a component schema. Takes a JSON object representing the UI layout (ComponentNode structure) and returns compilable Rust code. Use the type_name enum to see all available components.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "view_name": {
                                "type": "string",
                                "description": "Name for the generated view struct (e.g., 'LoginView', 'DashboardView')"
                            },
                            "schema": {
                                "type": "object",
                                "description": "The ComponentNode schema representing the UI tree",
                                "properties": {
                                    "type_name": {
                                        "type": "string",
                                        "enum": supported_components,
                                        "description": "Component type. Use containers (VStack, HStack, ZStack) to organize layout, and widgets (Button, Label, Input, etc.) for interactive elements."
                                    },
                                    "id": {
                                        "type": "string",
                                        "description": "Optional component ID (becomes struct field name, used for event handling)"
                                    },
                                    "props": {
                                        "type": "object",
                                        "description": "Component properties. Common props: label (Button/Label), placeholder (Input), spacing/padding (containers), data/chart_type (Chart), width/height (Image/Chart)."
                                    },
                                    "children": {
                                        "type": "array",
                                        "description": "Child components for container types (VStack, HStack, ZStack, ScrollView, etc.)"
                                    }
                                },
                                "required": ["type_name"]
                            }
                        },
                        "required": ["view_name", "schema"]
                    }
                }
            ]
        }),
    )
}

/// Handle tools/call request
fn handle_tools_call(id: Value, params: &Value) -> JsonRpcResponse {
    let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    match tool_name {
        "generate_oxid_ui" => match execute_generate_oxid_ui(&arguments) {
            Ok(code) => JsonRpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": code
                        }
                    ]
                }),
            ),
            Err(e) => JsonRpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Error: {}", e)
                        }
                    ],
                    "isError": true
                }),
            ),
        },
        _ => JsonRpcResponse::error(id, -32602, format!("Unknown tool: {}", tool_name)),
    }
}

/// Execute the generate_oxid_ui tool
fn execute_generate_oxid_ui(arguments: &Value) -> Result<String> {
    // Extract view_name
    let view_name = arguments
        .get("view_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'view_name' argument"))?;

    // Extract schema
    let schema_value = arguments
        .get("schema")
        .ok_or_else(|| anyhow!("Missing 'schema' argument"))?;

    // Parse schema into ComponentNode
    let schema: ComponentNode = serde_json::from_value(schema_value.clone())
        .map_err(|e| anyhow!("Failed to parse schema: {}", e))?;

    // Generate the code
    let code = generate_view(&schema, view_name);

    // ===== PREVIEW WINDOW LAUNCH =====
    let preview_launched = spawn_preview_window(schema_value);

    // Append preview note if launched
    let final_code = if preview_launched {
        format!("{}\n// (Preview window launched)", code)
    } else {
        code
    };

    Ok(final_code)
}

/// Spawns the oxidx-viewer process with the schema
///
/// Saves schema to temp file and launches viewer in background.
/// Returns true if viewer was successfully spawned.
fn spawn_preview_window(schema_value: &Value) -> bool {
    // 1. Serialize schema to JSON
    let schema_json = match serde_json::to_string_pretty(schema_value) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("[oxidx-mcp] Failed to serialize schema: {}", e);
            return false;
        }
    };

    // 2. Save to temp file
    let temp_path = "/tmp/oxidx_preview.json";
    if let Err(e) = fs::write(temp_path, &schema_json) {
        eprintln!("[oxidx-mcp] Failed to write temp file: {}", e);
        return false;
    }
    eprintln!("[oxidx-mcp] Saved preview schema to: {}", temp_path);

    // 3. Locate the viewer binary
    let viewer_path = find_viewer_binary();
    let viewer_path = match viewer_path {
        Some(p) => p,
        None => {
            eprintln!("[oxidx-mcp] Could not locate oxidx-viewer binary");
            return false;
        }
    };
    eprintln!("[oxidx-mcp] Found viewer at: {:?}", viewer_path);

    // 4. Spawn the viewer process (non-blocking)
    match Command::new(&viewer_path).arg(temp_path).spawn() {
        Ok(child) => {
            eprintln!("[oxidx-mcp] Launched viewer with PID: {}", child.id());
            true
        }
        Err(e) => {
            eprintln!("[oxidx-mcp] Failed to spawn viewer: {}", e);
            false
        }
    }
}

/// Finds the oxidx-viewer binary relative to the MCP executable
fn find_viewer_binary() -> Option<PathBuf> {
    // Try to find viewer relative to current executable
    if let Ok(exe_path) = std::env::current_exe() {
        let exe_dir = exe_path.parent()?;

        // Same directory as MCP binary
        let viewer_same = exe_dir.join("oxidx-viewer");
        if viewer_same.exists() {
            return Some(viewer_same);
        }

        // Try target/debug or target/release patterns
        if let Some(target_dir) = exe_dir.parent() {
            // If we're in target/debug, try target/debug/oxidx-viewer
            let viewer_target = target_dir.join("debug").join("oxidx-viewer");
            if viewer_target.exists() {
                return Some(viewer_target);
            }

            let viewer_release = target_dir.join("release").join("oxidx-viewer");
            if viewer_release.exists() {
                return Some(viewer_release);
            }
        }
    }

    // Fallback: try PATH
    if let Ok(output) = Command::new("which").arg("oxidx-viewer").output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
    }

    None
}
