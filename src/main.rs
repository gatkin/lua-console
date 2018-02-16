extern crate lua_console;

use std::io::{BufRead, Write};

use lua_console::lua;

struct LuaConsole;

impl lua::LuaIO for LuaConsole {
    fn on_print(&mut self, values: Vec<String>) {
        for value in &values {
            print!("{}", value);
        }

        println!("");
    }
}

fn main() {
    let mut io = LuaConsole{};
    let lua_state = lua::LuaState::new();

    let stdin = std::io::stdin();

    show_prompt();
    for line in stdin.lock().lines() {
        match lua_state.execute_chunk(&line.unwrap(), &mut io) {
            Err(rcode) => println!("Error {:?}", rcode),
            Ok(ref values) => print_return_values(values),
        };
        
        show_prompt();
    }
}

fn print_return_values(values: &Vec<String>) {
    for value in values {
        print!("{}  ", value);
    }

    if values.len() > 0 {
        println!("");
    }
}

fn show_prompt() {
    print!("/> ");
    std::io::stdout().flush().unwrap();
}
