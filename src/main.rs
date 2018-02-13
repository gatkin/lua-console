extern crate lua_console;

use std::cell::RefCell;
use std::io::{BufRead, Write};
use std::rc::Rc;

use lua_console::lua;

struct LuaConsole;

impl lua::LuaIO for LuaConsole {
    fn on_print(&mut self, values: Vec<String>) {
        println!("Received {:?}", values);
    }
}

fn main() {
    let io = Rc::new(RefCell::new(LuaConsole));
    let lua_state = lua::LuaState::new(io.clone());

    let stdin = std::io::stdin();

    show_prompt();
    for line in stdin.lock().lines() {
        lua_state.execute_chunk(&line.unwrap());
        show_prompt();
    }
}

fn show_prompt() {
    print!("/> ");
    std::io::stdout().flush().unwrap();
}
