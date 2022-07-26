#![feature(stmt_expr_attributes)]
#![feature(local_key_cell_methods)]

mod gui;

fn main() {
    let mut gui = gui::Gui::new();
    gui.run();
}
