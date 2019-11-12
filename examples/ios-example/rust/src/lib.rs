#[path = "../../../../examples/todos.rs"]
mod todos;

#[no_mangle]
pub extern "C" fn run_app() {
    color_backtrace::install();
    todos::main();
}
