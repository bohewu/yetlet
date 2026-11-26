fn main() -> std::io::Result<()> {
    yetlet::sfc::compile(
        &[
            ("todo-form", "src/TodoForm.yetlet"),
            ("todo-item", "src/TodoItem.yetlet"),
        ],
        std::env::var_os("OUT_DIR").expect("Cargo provides OUT_DIR"),
    )
}
