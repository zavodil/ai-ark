# AI Ark - WASI HTTP Example

WASM component that makes HTTP requests using WASI Preview 2 HTTP support.

## Description

Sends HTTP POST request to OpenAI-compatible API and returns the response. Demonstrates WASI P2 component with HTTP capabilities running on NEAR Offshore platform.

## Input Format

```json
{
  "prompt": "What is NEAR Protocol?"
}
```

## Output Format

```json
{
  "response": "NEAR Protocol is a layer-1 blockchain..."
}
```

## Building

```bash
# Add WASI P2 target
rustup target add wasm32-wasip2

# Build WASM component
cargo build --target wasm32-wasip2 --release

# Output: target/wasm32-wasip2/release/ai-ark.wasm
```

## Local Testing

```bash
# Build test runner (native binary)
cargo build --bin test_run --features test-runner

# Run test
./target/debug/test_run

# This will:
# - Load the WASM component
# - Execute it with wasmtime
# - Show output and fuel consumed
```

## Usage with NEAR Offshore

1. Push this code to a GitHub repository

2. Call `request_execution` on the OffchainVM contract:
```bash
near call offchainvm.testnet request_execution '{
  "code_source": {
    "repo": "https://github.com/username/ai-ark",
    "commit": "main",
    "build_target": "wasm32-wasip2"
  },
  "resource_limits": {
    "max_instructions": 100000000,
    "max_memory_mb": 128,
    "max_execution_seconds": 60
  },
  "input_data": "{\"prompt\":\"What is NEAR Protocol?\"}"
}' --accountId your-account.testnet --deposit 0.1
```

3. Worker will:
   - Compile the WASM component in Docker
   - Execute with wasmtime (WASI P2 + HTTP support)
   - Return the AI response as readable text
   - Show result in NEAR explorer

## Features

- ✅ WASI Preview 2 component
- ✅ HTTP/HTTPS requests support
- ✅ Fuel metering (instruction counting)
- ✅ JSON input/output via stdin/stdout
- ✅ Works with OpenAI-compatible APIs

## Notes

- Requires wasmtime 28+ for execution
- HTTP requests need network access (disabled in sandboxed compilation)
- API endpoint is hardcoded in source (can be made configurable via env vars)
