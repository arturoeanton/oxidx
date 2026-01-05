import ollama
import subprocess
import json
import sys
import os
import time
import random

# --- CONFIGURACIÓN ---
BASE_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(BASE_DIR, "..")) 

MCP_BINARY = os.path.join(PROJECT_ROOT, "target/release/oxidx-mcp")
VIEWER_BINARY = os.path.join(PROJECT_ROOT, "target/debug/oxidx-viewer")
MODEL_NAME = "llama3.2" 

# --- CLIENTE MCP ---
class MCPClient:
    def __init__(self):
        print(f"🚀 Conectando al Cerebro (MCP): {MCP_BINARY}")
        try:
            self.process = subprocess.Popen(
                [MCP_BINARY],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=sys.stderr,
                text=True,
                bufsize=1
            )
            self._handshake()
        except FileNotFoundError:
            print(f"❌ Error: No encuentro el MCP en: {MCP_BINARY}")
            sys.exit(1)

    def _handshake(self):
        init_req = {
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "magic-ide", "version": "1.0"}}
        }
        self._send(init_req)
        self._recv() 
        self._send({"jsonrpc": "2.0", "method": "notifications/initialized"})
        tools_req = {"jsonrpc": "2.0", "id": 2, "method": "tools/list"}
        self._send(tools_req)
        res = self._recv()
        # Guardamos las herramientas recibidas
        self.tools = res.get("result", {}).get("tools", [])
        print(f"✅ Sistema OxidX Online. Herramientas disponibles: {len(self.tools)}")

    def get_tools(self):
        """Devuelve la lista de herramientas obtenidas del handshake"""
        return self.tools

    def _send(self, data):
        self.process.stdin.write(json.dumps(data) + "\n")
        self.process.stdin.flush()

    def _recv(self):
        line = self.process.stdout.readline()
        if not line: return {}
        return json.loads(line)

    def call_tool(self, name, args):
        req = {
            "jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": {"name": name, "arguments": args}
        }
        self._send(req)
        res = self._recv()
        if "error" in res:
            return f"❌ Error: {res['error']['message']}"
        try:
            return res["result"]["content"][0]["text"]
        except (KeyError, IndexError):
            return str(res)

# --- UTILIDADES ---

def extract_json_from_text(text):
    text = text.strip()
    if "```json" in text:
        text = text.split("```json")[1].split("```")[0].strip()
    start = text.find('{')
    end = text.rfind('}')
    if start != -1 and end != -1:
        try: return json.loads(text[start:end+1])
        except: return None
    return None

def recursive_sanitize(data):
    if isinstance(data, str):
        data = data.strip()
        if (data.startswith('{') and data.endswith('}')) or (data.startswith('[') and data.endswith(']')):
            try: return recursive_sanitize(json.loads(data))
            except: pass 
        return data

    if isinstance(data, list):
        cleaned = []
        for item in data:
            sanitized = recursive_sanitize(item)
            if isinstance(sanitized, dict): cleaned.append(sanitized)
        return cleaned

    if isinstance(data, dict):
        new_data = {}
        for k, v in data.items():
            new_data[k] = recursive_sanitize(v)
        
        if ('children' in new_data or 'props' in new_data) and 'type_name' not in new_data:
            new_data['type_name'] = 'VStack' 
        if 'children' in new_data and not isinstance(new_data['children'], list):
            new_data['children'] = []
        if 'props' in new_data and not isinstance(new_data['props'], dict):
            new_data['props'] = {}
        return new_data

    return data

def launch_viewer(schema_json):
    temp_file = "temp_preview.json"
    
    # DEBUG: Ver qué JSON estamos intentando pintar
    print(f"🔍 DEBUG JSON: {json.dumps(schema_json)[:100]}... (len: {len(str(schema_json))})")
    
    with open(temp_file, "w") as f:
        json.dump(schema_json, f, indent=2)
    
    print(f"👁️  Abriendo Visor Nativo...")
    try:
        subprocess.Popen([VIEWER_BINARY, temp_file], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
    except:
        print(f"⚠️  Error lanzando viewer.")

def save_rust_code(view_name, code):
    filename = f"{view_name}.rs"
    with open(filename, "w") as f:
        f.write(code)
    print(f"💾 Código guardado en: \033[1m{filename}\033[0m")

def main():
    print("\n✨ \033[1;36mOxidX Magic Console (Dynamic + Chart)\033[0m ✨")
    mcp = MCPClient()
    
    # --- LÓGICA DINÁMICA DE COMPONENTES ---
    # 1. Lista por defecto (Fallback) por si el MCP Rust aún no está actualizado
    default_components = ['VStack', 'HStack', 'ZStack', 'Button', 'Label', 'Input', 'Image', 'Chart']
    
    allowed_components = default_components

    # 2. Intentamos leer la lista real del MCP
    try:
        tools = mcp.get_tools()
        if tools:
            # Buscamos dentro del esquema JSON que devuelve Rust
            # Nota: Esto funcionará cuando apliques el prompt a Claude para actualizar el MCP
            schema_enum = tools[0].get('inputSchema', {}).get('properties', {}).get('type_name', {}).get('enum')
            if schema_enum:
                print(f"📡 Componentes sincronizados con Rust: {len(schema_enum)}")
                allowed_components = schema_enum
    except Exception as e:
        print(f"⚠️ Usando lista de componentes por defecto ({len(allowed_components)})")

    # 3. CONSTRUIMOS LA HERRAMIENTA AQUÍ (En tiempo de ejecución)
    ollama_tools = [
      {
        'type': 'function',
        'function': {
          'name': 'generate_oxid_ui',
          'description': 'Generates Rust UI code.',
          'parameters': {
            'type': 'object',
            'properties': {
              'view_name': { 'type': 'string' },
              'type_name': { 'type': 'string', 'enum': allowed_components }, # <--- ¡DINÁMICO!
              'props': { 'type': 'object' },
              'children': { 'type': 'array', 'items': { 'type': 'object' } }
            },
            'required': ['view_name', 'type_name']
          }
        }
      }
    ]
    
    # --- PROMPT MAESTRO ---
    system_prompt = """You are an expert OxidX UI builder. Use the generate_oxid_ui tool.

EXAMPLE OF CORRECT JSON OUTPUT:
{
  "view_name": "LoginScreen",
  "type_name": "VStack",
  "props": { "spacing": 20, "padding": 20 },
  "children": [
    { "type_name": "Label", "props": { "text": "Welcome Back", "font_size": 24 } },
    { "type_name": "Input", "props": { "placeholder": "Username" } },
    { "type_name": "Chart", "props": { "title": "Sales Data" } },
    { "type_name": "Button", "props": { "label": "Sign In", "variant": "primary" } }
  ]
}

RULES:
1. Always include 'children' array with components.
2. Never output empty VStacks unless requested.
3. Use 'Input' for text fields.
"""
    
    messages = [{'role': 'system', 'content': system_prompt}]
    
    while True:
        try: user_input = input("\n🎨 \033[1;32mDescribe tu UI:\033[0m ")
        except: break
        if user_input.lower() in ['salir', 'exit']: break
        
        if len(messages) > 6: messages = [messages[0]]
        
        messages.append({'role': 'user', 'content': user_input})
        print("🤔 Diseñando...")

        response = ollama.chat(model=MODEL_NAME, messages=messages, tools=ollama_tools)
        tool_calls = response.get('message', {}).get('tool_calls')
        
        if not tool_calls:
            extracted = extract_json_from_text(response['message']['content'])
            if extracted:
                args = extracted.get("parameters", extracted.get("arguments", extracted))
                tool_calls = [{'function': {'name': 'generate_oxid_ui', 'arguments': args}}]

        if tool_calls:
            for tool in tool_calls:
                fn_name = tool['function']['name']
                fn_args = tool['function']['arguments']
                
                if fn_name == 'generate_oxid_ui':
                    print(f"🔨 Generando Arquitectura...")
                    
                    fn_args = recursive_sanitize(fn_args)

                    if 'type_name' not in fn_args: fn_args['type_name'] = 'VStack'
                    if 'view_name' not in fn_args: fn_args['view_name'] = f"AutoView_{random.randint(100,999)}"

                    view_name = fn_args.pop('view_name')
                    schema_for_viewer = fn_args.copy() 

                    mcp_args = {"view_name": view_name, "schema": fn_args}
                    rust_code = mcp.call_tool(fn_name, mcp_args)
                    
                    if "Error:" in rust_code:
                         print(f"\n❌ \033[1;31m{rust_code}\033[0m")
                    else:
                        save_rust_code(view_name, rust_code)
                        launch_viewer(schema_for_viewer)
                        print("\n" + "="*50)
                        print(rust_code[:200] + "...")
                        print("="*50)
                    
                    messages.append(response['message'])
                    messages.append({'role': 'tool', 'content': rust_code})
        else:
            print(f"🤖 {response['message']['content']}")
            messages.append(response['message'])

if __name__ == "__main__":
    main()