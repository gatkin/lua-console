extern crate libc;

mod executor;
mod lua;
mod session;

use std::io::{BufRead, Write};

struct LuaConsole;

impl lua::LuaIO for LuaConsole {
    fn on_print(&mut self, values: Vec<String>) {
        println!("Receved {:?}", values);
    }
}

fn main() {
    let io = Box::new(LuaConsole);
    let mut lua_state = lua::LuaState::new(io);

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
