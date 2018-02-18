extern crate lua_console;

use lua_console::lua;


struct IOReceiver;


impl lua::LuaIO for IOReceiver {
    fn on_print(&mut self, _values: Vec<String>) {
    }
}


struct TestCase {
    chunk: &'static str,
    exptd_status: lua::LuaErrorStatus,
    exptd_msg_substr: &'static str,
}


impl TestCase {
    fn run(&self) {
        let mut io_reciver = IOReceiver{};
        let lua_state = lua::LuaState::new();

        let result = lua_state.execute_chunk(self.chunk, &mut io_reciver);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(self.exptd_status, error.status);
        assert!(error.message.contains(self.exptd_msg_substr));
    }
}


#[test]
fn assertion_failure() {
    let test_case = TestCase{
        chunk: "assert(false)",
        exptd_status: lua::LuaErrorStatus::RuntimeError,
        exptd_msg_substr: "assertion failed",
    };

    test_case.run();
}


#[test]
fn assertion_failure_with_msg() {
    let test_case = TestCase{
        chunk: "assert(false, 'Failure message')",
        exptd_status: lua::LuaErrorStatus::RuntimeError,
        exptd_msg_substr: "Failure message",
    };

    test_case.run();
}


#[test]
fn syntax_error() {
    let test_case = TestCase{
        chunk: "x = {]",
        exptd_status: lua::LuaErrorStatus::SyntaxError,
        exptd_msg_substr: "unexpected symbol",
    };

    test_case.run();
}
