extern crate lua_console;

use lua_console::repl::ConsoleRepl;


fn main() {
    let mut repl = ConsoleRepl::new();
    repl.run_repl();
}

