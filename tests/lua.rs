extern crate lua_console;

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


struct TestChunk {
    chunk: &'static str,
    expected_return_values: Vec<&'static str>
}


struct TestCase {
    chunks: Vec<TestChunk>,
    expected_print_values: Vec<&'static str>
}


impl TestCase {
    fn run(&self) {
        let mut io_receiver = IOReceiver::new();
        let lua_state = lua::LuaState::new();

        for chunk in &self.chunks {
            let result = lua_state.execute_chunk(chunk.chunk, &mut io_receiver);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), chunk.expected_return_values);
        }

        assert_eq!(self.expected_print_values, io_receiver.values);
    }
}


#[test]
fn explicit_return() {
    let test_case = TestCase{
        chunks: vec![
            TestChunk{
                chunk: "return 17, false",
                expected_return_values: vec!["17", "false"],
            }
        ],
        expected_print_values: vec![],
    };

    test_case.run();
}


#[test]
fn leftover_stack_values_returned() {
    let test_case = TestCase {
        chunks: vec![
            TestChunk{
                chunk: "give_two = function() return 5, true end",
                expected_return_values: vec![],
            },
            TestChunk{
                chunk: "give_two()",
                expected_return_values: vec!["5", "true"],
            },
            TestChunk{
                chunk: "x = 5",
                expected_return_values: vec![],
            },
            TestChunk{
                chunk: "x",
                expected_return_values: vec!["5"],
            },
            TestChunk{
                chunk: "5, nil, false, 'Hello'",
                expected_return_values: vec!["5", "nil", "false", "Hello"],
            }
        ],
        expected_print_values: vec![],
    };
    
    test_case.run();
}


#[test]
fn print_local_vars() {
    let test_case = TestCase {
        chunks: vec![
            TestChunk{
                chunk: "x = 5",
                expected_return_values: vec![],
            },
            TestChunk{
                chunk: "print(x)",
                expected_return_values: vec![],
            },
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
            TestChunk{
                chunk: "print('a', 5, false, nil)",
                expected_return_values: vec![],
            },
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
            TestChunk{
                chunk: "print('Hello, World!')",
                expected_return_values: vec![],
            },
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
            TestChunk{
                chunk: "t = {}",
                expected_return_values: vec![],
            },
            TestChunk{
                chunk: "table.insert(t, 17)",
                expected_return_values: vec![],
            },
            TestChunk{
                chunk: "print(t[1])",
                expected_return_values: vec![],
            },
        ],
        expected_print_values: vec![
            "17",
        ],
    };
    
    test_case.run();
}
