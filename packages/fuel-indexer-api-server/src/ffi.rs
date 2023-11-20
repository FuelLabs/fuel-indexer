use wasmer::{
    imports, AsStoreMut, Exports, Function, Instance, MemoryView, StoreMut, WasmPtr,
};

/// Extract the `TOOLCHAIN_VERSION` string from a WASM module. This function
/// creates a `wasmer::Instance` in order to do this.
pub(crate) fn check_wasm_toolchain_version(data: Vec<u8>) -> anyhow::Result<String> {
    let mut store = wasmer::Store::default();

    let module = wasmer::Module::new(&store, data.clone())?;

    let mut exports = Exports::new();
    exports.insert(
        "ff_put_object".to_string(),
        Function::new_typed(&mut store, |_: i64, _: i32, _: i32| {}),
    );
    exports.insert(
        "ff_get_object".to_string(),
        Function::new_typed(&mut store, |_: i64, _: i32, _: i32| 0i32),
    );
    exports.insert(
        "ff_single_select".to_string(),
        Function::new_typed(&mut store, |_: i64, _: i32, _: i32| 0i32),
    );
    exports.insert(
        "ff_early_exit".to_string(),
        Function::new_typed(&mut store, |_: i32| {}),
    );
    exports.insert(
        "ff_put_many_to_many_record".to_string(),
        Function::new_typed(&mut store, |_: i32, _: i32| {}),
    );
    exports.insert(
        "ff_log_data".to_string(),
        Function::new_typed(&mut store, |_: i32, _: i32, _: i32| {}),
    );

    let mut imports = imports! {};
    wasmer::Imports::register_namespace(&mut imports, "env", exports);

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
