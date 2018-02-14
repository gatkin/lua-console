extern crate lua_console;

use std::io::{BufRead, Write};

use lua_console::lua;

struct LuaConsole;

impl lua::LuaIO for LuaConsole {
    fn on_print(&mut self, values: Vec<String>) {
        println!("Received {:?}", values);
    }
}

fn main() {
    let mut io = LuaConsole{};
    let lua_state = lua::LuaState::new();

    let stdin = std::io::stdin();

    show_prompt();
    for line in stdin.lock().lines() {
        lua_state.execute_chunk(&line.unwrap(), &mut io);
        show_prompt();
    }
}

fn show_prompt() {
    print!("/> ");
    std::io::stdout().flush().unwrap();
}
