# AI Ark - WASI HTTP Example

> **[Full documentation](https://outlayer.fastnear.com/docs/examples#ai-example)** on the OutLayer dashboard.

WASM component that makes HTTP requests to OpenAI-compatible APIs using WASI Preview 2 HTTP support.

## Description

Sends HTTP POST request to OpenAI-compatible API and returns the response. Demonstrates WASI P2 component with HTTP capabilities running on NEAR OutLayer platform.

**Key Features:**
- Supports custom system prompts via environment variables (secrets)
- Works with any OpenAI-compatible API endpoint
- Configurable model, max tokens, and conversation history
- Secure API key management through encrypted secrets

## Input Format

```json
{
  "prompt": "What is NEAR Protocol?",
  "history": [
    {"role": "user", "content": "Previous question"},
    {"role": "assistant", "content": "Previous answer"}
  ],
  "openai_endpoint": "https://api.openai.com/v1/chat/completions",
  "model_name": "gpt-3.5-turbo",
  "max_tokens": 150
}
```

**Required fields:**
- `prompt` - User's question/prompt (string)

**Optional fields:**
- `history` - Conversation history (array of messages, default: empty)
- `openai_endpoint` - API endpoint URL (default: OpenAI)
- `model_name` - Model name (default: "gpt-3.5-turbo")
- `max_tokens` - Maximum tokens in response (default: 150)

## Output Format

Plain text response from the AI model (not JSON-wrapped):

```
NEAR Protocol is a layer-1 blockchain designed for building decentralized applications...
```

## Building

```bash
# Add WASI P2 target
rustup target add wasm32-wasip2

# Build WASM component
cargo build --target wasm32-wasip2 --release

# Output: target/wasm32-wasip2/release/ai-example.wasm
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

## Environment Variables (Secrets)

This WASM component reads sensitive data from environment variables, which should be stored as encrypted secrets:

### Required:
- `OPENAI_API_KEY` - Your OpenAI API key (or compatible API key)

### Optional:
- `SYSTEM_PROMPT` - Custom system prompt to control AI behavior

**Example system prompts:**
```
"You are a helpful blockchain expert. Answer questions about NEAR Protocol concisely."
"You are a code assistant. Provide code examples in Rust when relevant."
"You are a friendly teacher. Explain complex topics in simple terms."
```

## Usage with NEAR OutLayer

### Step 1: Store Secrets

First, store your API key and system prompt as encrypted secrets using the dashboard or CLI:

**Option A: Using Dashboard** (Recommended)
1. Open http://localhost:3000/secrets
2. Connect your NEAR wallet
3. Fill in the form:
   - **Repo**: `github.com/your-username/ai-example`
   - **Branch**: `main`
   - **Profile**: `production`
   - **Secrets** (JSON):
     ```json
     {
       "OPENAI_API_KEY": "sk-...",
       "SYSTEM_PROMPT": "You are a helpful blockchain expert."
     }
     ```
   - **Access Condition**: Select who can use these secrets (e.g., AllowAll, Whitelist, NEAR balance check)
4. Click "Encrypt & Store Secrets"

**Option B: Using Python Script**
```bash
cd keystore-worker

# Create secrets JSON file
cat > secrets.json << 'EOF'
{
  "OPENAI_API_KEY": "sk-proj-...",
  "SYSTEM_PROMPT": "You are a helpful blockchain expert. Answer questions about NEAR Protocol concisely."
}
EOF

# Encrypt secrets
python3 encrypt_secrets.py secrets.json \
  --repo "github.com/your-username/ai-example" \
  --branch "main" \
  --owner "your-account.testnet" \
  --keystore-url "http://localhost:8080/secrets/pubkey"

# This outputs an array like: [42,15,67,...]
# Copy this array for the next step
```

**Option C: Store via NEAR CLI**
```bash
# Store encrypted secrets on contract
near call outlayer.testnet store_secrets '{
  "repo": "github.com/your-username/ai-example",
  "branch": "main",
  "profile": "production",
  "encrypted_data": [42,15,67,...],
  "access_condition": {"AllowAll": {}}
}' \
  --accountId your-account.testnet \
  --deposit 0.1
```

### Step 2: Request Execution

Call `request_execution` on the OutLayer contract:

**Simple execution (no secrets):**
```bash
near call outlayer.testnet request_execution '{
  "code_source": {
    "repo": "https://github.com/your-username/ai-example",
    "commit": "main",
    "build_target": "wasm32-wasip2"
  },
  "resource_limits": {
    "max_instructions": 500000000,
    "max_memory_mb": 128,
    "max_execution_seconds": 60
  },
  "input_data": "{\"prompt\":\"What is NEAR Protocol?\"}"
}' --accountId your-account.testnet --deposit 0.1
```

**With secrets (API key + system prompt):**
```bash
near call outlayer.testnet request_execution '{
  "code_source": {
    "repo": "https://github.com/your-username/ai-example",
    "commit": "main",
    "build_target": "wasm32-wasip2"
  },
  "secrets_ref": {
    "profile": "production",
    "account_id": "your-account.testnet"
  },
  "resource_limits": {
    "max_instructions": 500000000,
    "max_memory_mb": 128,
    "max_execution_seconds": 60
  },
  "input_data": "{\"prompt\":\"Explain smart contracts\",\"max_tokens\":300}"
}' --accountId your-account.testnet --deposit 0.1
```

**With conversation history:**
```bash
near call outlayer.testnet request_execution '{
  "code_source": {
    "repo": "https://github.com/your-username/ai-example",
    "commit": "main",
    "build_target": "wasm32-wasip2"
  },
  "secrets_ref": {
    "profile": "production",
    "account_id": "your-account.testnet"
  },
  "resource_limits": {
    "max_instructions": 500000000,
    "max_memory_mb": 128,
    "max_execution_seconds": 60
  },
  "input_data": "{\"prompt\":\"What about gas fees?\",\"history\":[{\"role\":\"user\",\"content\":\"Tell me about NEAR\"},{\"role\":\"assistant\",\"content\":\"NEAR is a blockchain...\"}]}"
}' --accountId your-account.testnet --deposit 0.1
```

### Step 3: Worker Execution Flow

The worker will automatically:
1. **Fetch secrets** from the contract (matching repo + branch + profile + owner)
2. **Validate access** conditions via keystore
3. **Decrypt secrets** via keystore (gets `OPENAI_API_KEY` and `SYSTEM_PROMPT`)
4. **Inject into WASI environment** as environment variables
5. **Compile WASM** (if not cached) in sandboxed Docker
6. **Execute WASM** with wasmtime (WASI P2 + HTTP support)
7. **Return AI response** as plain text
8. **Submit result** back to NEAR contract

Your WASM code automatically reads:
- `std::env::var("OPENAI_API_KEY")` → Gets decrypted API key
- `std::env::var("SYSTEM_PROMPT")` → Gets custom system prompt (if provided)

## Features

- ✅ WASI Preview 2 component
- ✅ HTTP/HTTPS requests support
- ✅ Fuel metering (instruction counting)
- ✅ JSON input/output via stdin/stdout
- ✅ Works with OpenAI-compatible APIs
- ✅ **Custom system prompts via `SYSTEM_PROMPT` environment variable**
- ✅ Encrypted secrets management via NEAR OutLayer
- ✅ Configurable API endpoint, model, and parameters
- ✅ Conversation history support

## Security Model

1. **API keys** are stored encrypted on NEAR blockchain
2. **Access control** - owner defines who can use secrets (whitelist, balance checks, etc.)
3. **Secrets decryption** happens in keystore worker (isolated from executor)
4. **Environment injection** - secrets passed as WASI env vars (secure, ephemeral)
5. **No plaintext storage** - API keys never stored in plaintext

## Use Cases

**Customer Support Bot:**
```json
{
  "SYSTEM_PROMPT": "You are a friendly customer support agent for a DeFi protocol. Be helpful and concise.",
  "OPENAI_API_KEY": "sk-..."
}
```

**Code Assistant:**
```json
{
  "SYSTEM_PROMPT": "You are an expert Rust developer. Provide code examples and best practices.",
  "OPENAI_API_KEY": "sk-..."
}
```

**Research Assistant:**
```json
{
  "SYSTEM_PROMPT": "You are a blockchain research assistant. Provide accurate technical information with sources when possible.",
  "OPENAI_API_KEY": "sk-..."
}
```

## Advanced Configuration

You can also use alternative OpenAI-compatible APIs:

```json
{
  "prompt": "Explain NEAR Protocol",
  "openai_endpoint": "https://api.anthropic.com/v1/messages",
  "model_name": "claude-3-sonnet",
  "max_tokens": 500
}
```

**Note:** Make sure your API key matches the endpoint you're using.

## Troubleshooting

**Error: OPENAI_API_KEY not found**
- Make sure you stored secrets on contract with correct repo/branch/profile
- Verify `secrets_ref` in your `request_execution` call matches stored secrets
- Check access conditions allow your account to decrypt

**Error: OpenAI API returned status 401**
- Your API key is invalid or expired
- Update secrets with new API key using dashboard or CLI

**Error: OpenAI API returned status 429**
- Rate limit exceeded on your API key
- Wait a few moments and try again
- Consider upgrading your OpenAI plan

**HTTP timeout**
- Increase `max_execution_seconds` in resource_limits
- Default is 60 seconds, which should be sufficient for most queries

## Notes

- Requires wasmtime 28+ for execution (worker uses wasmtime 27)
- HTTP requests require network access (allowed during execution, disabled during compilation)
- Compilation happens in sandboxed Docker with `--network=none`
- System prompt is **optional** - if not provided, OpenAI uses default behavior
- Maximum recommended `max_tokens`: 4000 (to stay within instruction limits)

## License

MIT OR Apache-2.0, at your option — see `LICENSE-MIT` and `LICENSE-APACHE`.
