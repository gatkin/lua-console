extern crate libc;

mod lua;


fn main() {
    let mut lua_state = lua::LuaState::new();

    lua_state.execute_chunk("print('Hello, World from Lua!')");
    lua_state.execute_chunk("for i in pairs(_G) do print(i) end");
}
