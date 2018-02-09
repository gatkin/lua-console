use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use libc;

const LUA_MULTRET: c_int = -1;

type lua_State = *mut c_void;

/// Provides a safe handle to the lua_State structure used in the
/// Lua C API.
pub struct LuaState {
    state: *mut lua_State,
}

impl LuaState {
    /// Creates and configures a new Lua state that can be used to execute
    /// Lua chunks
    pub fn new() -> LuaState {
        unsafe {
            let state = lua_newstate(allocator, ptr::null_mut());
            luaL_openlibs(state);

            LuaState{
                state,
            }
        }
    }

    /// Executes the given Lua chunk. Returns all values pushed onto the stack
    /// by the chunk converted to strings.
    pub fn execute_chunk(&mut self, chunk: &str) -> i32 {
        unsafe {
            let mut rcode = self.load_string(chunk);
            if rcode == 0 {
                rcode = lua_pcallk(self.state, 0, LUA_MULTRET, 0, ptr::null_mut(), ptr::null_mut());
            }

            rcode
        }
    }

    fn load_string(&mut self, chunk: &str) -> i32 {
        unsafe {
            luaL_loadbufferx(
                self.state,
                chunk.as_ptr() as *const c_char,
                chunk.len() as libc::size_t,
                chunk.as_ptr() as *const c_char,
                ptr::null()
            )
        }
    }
}

impl Drop for LuaState {
    fn drop(&mut self) {
        unsafe {
            lua_close(self.state);
        }
    }
}

type lua_Alloc = unsafe extern "C" fn(ud: *mut c_void, ptr: *mut c_void, osize: usize, nsize: usize) -> *mut c_void;
type lua_KContext = *mut c_void;
type lua_KFunction = *mut c_void;

unsafe extern "C" fn allocator
    (
    _ud: *mut c_void,
    ptr: *mut c_void,
    _osize: usize,
    nsize: usize
    ) -> *mut c_void
{
    if nsize == 0 {
        libc::free(ptr as *mut libc::c_void);
        ptr::null_mut()
    } else {
        let p = libc::realloc(ptr as *mut libc::c_void, nsize);
        p as *mut c_void
    }
}

#[link(name = "lua5.3")]
extern "C" {
    fn lua_close(L: *mut lua_State);

    fn lua_gettop(L: *mut lua_State) -> c_int;

    fn lua_newstate(alloc: lua_Alloc, ud: *mut c_void) -> *mut lua_State;

    fn lua_pcallk
        (
        L: *mut lua_State,
        n_args: c_int,
        nresults: c_int,
        errfunc: c_int,
        ctx: lua_KContext,
        k: lua_KFunction
        ) -> c_int;

    fn luaL_loadbufferx
        (
        L: *mut lua_State,
        buff: *const c_char,
        size: libc::size_t,
        name: *const c_char,
        mode: *const c_char
        ) -> c_int;

    fn luaL_openlibs(L: *mut lua_State);
}