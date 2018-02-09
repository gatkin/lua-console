extern crate libc;

mod lua;

use std::io::{BufRead, Write};


fn main() {
    let mut lua_state = lua::LuaState::new();

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
