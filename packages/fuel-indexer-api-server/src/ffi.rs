use wasmer::{AsStoreMut, Instance, MemoryView, StoreMut, WasmPtr};

pub(crate) fn check_wasm_toolchain_version(data: Vec<u8>) -> anyhow::Result<String> {
    let mut store = wasmer::Store::default();

    let module = wasmer::Module::new(&store, data.clone())?;

    let imports = wasmer::imports! {};

    let instance = wasmer::Instance::new(&mut store, &module, &imports)?;

    let version = get_toolchain_version(&mut store.as_store_mut(), &instance)?;

    Ok(version)
}

/// Get the toolchain version stored in the WASM module.
pub fn get_toolchain_version(
    store: &mut StoreMut,
    instance: &Instance,
) -> anyhow::Result<String> {
    let exports = &instance.exports;

    let ptr = exports
        .get_function("get_toolchain_version_ptr")?
        .call(store, &[])?[0]
        .i32()
        .ok_or_else(|| anyhow::anyhow!("get_toolchain_version_ptr".to_string()))?
        as u32;

    let len = exports
        .get_function("get_toolchain_version_len")?
        .call(store, &[])?[0]
        .i32()
        .ok_or_else(|| anyhow::anyhow!("get_toolchain_version_len".to_string()))?
        as u32;

    let memory = exports.get_memory("memory")?.view(store);
    let version = get_string(&memory, ptr, len)?;

    Ok(version)
}

/// Fetch the string at the given pointer from memory.
fn get_string(mem: &MemoryView, ptr: u32, len: u32) -> anyhow::Result<String> {
    let result = WasmPtr::<u8>::new(ptr).read_utf8_string(mem, len)?;
    Ok(result)
}
