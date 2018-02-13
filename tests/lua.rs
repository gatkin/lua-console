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


fn create_io_reciver() -> Rc<RefCell<IOReceiver>> {
    Rc::new(RefCell::new(IOReceiver::new()))
}


#[test]
fn leftover_stack_values_printed() {
    let io_receiver = create_io_reciver();
    let lua_state = lua::LuaState::new(io_receiver);

    let rcode = lua_state.execute_chunk("give_two = function() return 5, true end");
    assert_eq!(lua::LuaRcode::Ok, rcode);

    let rcode = lua_state.execute_chunk("give_two()");
    assert_eq!(lua::LuaRcode::Ok, rcode);

    let expected_values = vec!["5", "true"];
}


#[test]
fn print_local_vars() {
    let io_receiver = create_io_reciver();
    let lua_state = lua::LuaState::new(io_receiver.clone());
    
    let rcode = lua_state.execute_chunk("x = 5");
    assert_eq!(lua::LuaRcode::Ok, rcode);

    let rcode = lua_state.execute_chunk("print(x)");
    assert_eq!(lua::LuaRcode::Ok, rcode);

    let expected_values = vec!["5"];
    assert_eq!(expected_values, io_receiver.borrow().values);
}


#[test]
fn print_many_values() {
    let io_receiver = create_io_reciver();
    let lua_state = lua::LuaState::new(io_receiver.clone());

    let rcode = lua_state.execute_chunk("print('a', 5, false, nil)");
    assert_eq!(lua::LuaRcode::Ok, rcode);

    let expected_values = vec!["a", "5", "false", "nil"];
    assert_eq!(expected_values, io_receiver.borrow().values);
}


#[test]
fn print_single_value() {
    let io_receiver = create_io_reciver();
    let lua_state = lua::LuaState::new(io_receiver.clone());
    
    let rcode = lua_state.execute_chunk("print('Hello, World!')");
    assert_eq!(lua::LuaRcode::Ok, rcode);

    let expected_values = vec!["Hello, World!"];
    assert_eq!(expected_values, io_receiver.borrow().values);
}


#[test]
fn use_standard_module() {
    let io_receiver = create_io_reciver();
    let lua_state = lua::LuaState::new(io_receiver.clone());

    let rcode = lua_state.execute_chunk("t = {}");
    assert_eq!(lua::LuaRcode::Ok, rcode);

    let rcode = lua_state.execute_chunk("table.insert(t, 17)");
    assert_eq!(lua::LuaRcode::Ok, rcode);

    let rcode = lua_state.execute_chunk("print(t[1])");
    assert_eq!(lua::LuaRcode::Ok, rcode);

    let expected_values = vec!["17"];
    assert_eq!(expected_values, io_receiver.borrow().values);
}
