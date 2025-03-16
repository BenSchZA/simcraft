use console_error_panic_hook;
use console_log;
use log::Level;

pub fn init_logging() {
    console_error_panic_hook::set_once(); // Show panics in browser console
    console_log::init_with_level(Level::Debug)
        .map_err(|err| println!("Failed to initialize WASM logger: {:?}", err))
        .ok();
}
