use wasmtime::{Config, Engine, Store};
use wasmtime::component::{Component, Linker};
use wasmtime_wasi::{DirPerms, FilePerms, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi::bindings::Command;
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

struct HostState {
    wasi_ctx: WasiCtx,
    wasi_http_ctx: WasiHttpCtx,
    table: ResourceTable,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl WasiHttpView for HostState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.wasi_http_ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wasm = std::fs::read("target/wasm32-wasip2/release/ai-ark.wasm")?;
    println!("📦 Loaded {} bytes", wasm.len());

    // Read test input
    let test_input = std::fs::read("test.json")?;
    println!("📝 Test input: {}", String::from_utf8_lossy(&test_input));

    // Configure engine
    let mut config = Config::new();
    config.consume_fuel(true);
    config.wasm_component_model(true);
    config.async_support(true);
    let engine = Engine::new(&config)?;

    let component = Component::from_binary(&engine, &wasm)?;
    println!("✅ Component loaded!");

    // Setup linker with WASI + HTTP
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_async(&mut linker)?;
    wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;

    // Setup stdin/stdout pipes
    let stdin_pipe = wasmtime_wasi::pipe::MemoryInputPipe::new(test_input);
    let stdout_pipe = wasmtime_wasi::pipe::MemoryOutputPipe::new(1024 * 1024);

    // Build WASI context with env vars
    let mut wasi_builder = WasiCtxBuilder::new();
    wasi_builder.stdin(stdin_pipe);
    wasi_builder.stdout(stdout_pipe.clone());
    wasi_builder.stderr(wasmtime_wasi::pipe::MemoryOutputPipe::new(1024 * 1024));

    // Add preopened directory (required for WASI P2 components with filesystem)
    wasi_builder.preopened_dir("/tmp", ".", DirPerms::all(), FilePerms::all())?;

    wasi_builder.env("OPENAI_API_KEY", "test-key-123");

    let host_state = HostState {
        wasi_ctx: wasi_builder.build(),
        wasi_http_ctx: WasiHttpCtx::new(),
        table: ResourceTable::new(),
    };

    // Create store with fuel limit
    let mut store = Store::new(&engine, host_state);
    store.set_fuel(100_000_000)?;
    println!("✅ Fuel set to 100M");

    // Instantiate as Command component
    println!("🔧 Instantiating component...");
    let command = Command::instantiate_async(&mut store, &component, &linker).await?;
    println!("✅ Component instantiated!");

    // Call wasi:cli/run
    println!("🚀 Calling wasi:cli/run...");

    let fuel_before = store.get_fuel().unwrap_or(0);

    let result = command.wasi_cli_run().call_run(&mut store).await?;

    let fuel_after = store.get_fuel().unwrap_or(0);
    let fuel_consumed = fuel_before - fuel_after;

    println!("⛽ Fuel consumed: {}", fuel_consumed);
    println!("✅ Result: {:?}", result);

    let output = stdout_pipe.contents();
    println!("📤 Output ({} bytes): {}", output.len(), String::from_utf8_lossy(&output));

    Ok(())
}
