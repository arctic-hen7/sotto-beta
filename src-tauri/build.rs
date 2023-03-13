fn main() {
    // TODO Test if this is still necessary with static linking on macOS
    pyo3_build_config::add_extension_module_link_args();

    tauri_build::build()
}
