extern crate lua_console;

use std::cell::RefCell;
use std::rc::Rc;

use lua_console::lua;


struct IOReceiver {
    values: Vec<String>
}


impl IOReceiver {
    fn new() -> IOReceiver {
        IOReceiver{
            values: Vec::new(),
        }
    }
}


impl lua::LuaIO for IOReceiver {
    fn on_print(&mut self, mut values: Vec<String>) {
        self.values.append(&mut values);
    }
}


struct TestCase {
    chunks: Vec<&'static str>,
    expected_print_values: Vec<&'static str>
}


impl TestCase {
    fn run(&self) {
        let io_receiver = Rc::new(RefCell::new(IOReceiver::new()));
        let lua_state = lua::LuaState::new(io_receiver.clone());

        for chunk in &self.chunks {
            let rcode = lua_state.execute_chunk(chunk);
            assert_eq!(lua::LuaRcode::Ok, rcode);
        }

        assert_eq!(self.expected_print_values, io_receiver.borrow().values);
    }
}


#[test]
fn leftover_stack_values_printed() {
    let test_case = TestCase {
        chunks: vec![
            "give_two = function() return 5, true end",
            "give_two()",
        ],
        expected_print_values: vec![
            "5",
            "true",
        ],
    };
    
    test_case.run();
}


#[test]
fn print_local_vars() {
    let test_case = TestCase {
        chunks: vec![
            "x = 5",
            "print(x)",
        ],
        expected_print_values: vec![
            "5",
        ],
    };
    
    test_case.run();
}


#[test]
fn print_many_values() {
    let test_case = TestCase {
        chunks: vec![
            "print('a', 5, false, nil)",
        ],
        expected_print_values: vec![
            "a",
            "5",
            "false",
            "nil",
        ],
    };
    
    test_case.run();
}


#[test]
fn print_single_value() {
    let test_case = TestCase {
        chunks: vec![
            "print('Hello, World!')",
        ],
        expected_print_values: vec![
            "Hello, World!",
        ],
    };
    
    test_case.run();
}


#[test]
fn use_standard_module() {
    let test_case = TestCase {
        chunks: vec![
            "t = {}",
            "table.insert(t, 17)",
            "print(t[1])",
        ],
        expected_print_values: vec![
            "17",
        ],
    };
    
    test_case.run();
}
