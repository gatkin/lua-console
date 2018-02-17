#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
/// Contains all FFI function declarations from the Lua API that are used by the rest of the library.
/// Many common Lua API functions are actually implemented as macros which are not available through the
/// FFI mechanism. Those macros are implemented here as normal Rust functions using the FFI Lua functions
/// exactly how they are implemented in the Lua header files.

use std::os::raw::{c_char, c_int, c_void, c_longlong};
use std::ptr;

use libc;

const LUAI_MAXSTACK: c_int = 1000000;
pub const LUA_MULTRET: c_int = -1;
pub const LUA_OK: c_int = 0;
pub const LUA_ERRUN: c_int = 2;
pub const LUA_REGISTRYINDEX: c_int = (-LUAI_MAXSTACK) - 1000;
pub const LUA_RIDX_GLOBALS: c_int = 2;

pub type lua_CFunction = unsafe extern "C" fn(L: *mut lua_State) -> c_int;
pub type lua_Integer = c_longlong;
pub type lua_KContext = *mut c_void;
pub type lua_KFunction = *mut c_void;
pub type lua_State = *mut c_void;

#[link(name = "lua5.3")]
extern "C" {
    pub fn lua_callk
        (
        L: *mut lua_State,
        n_args: c_int,
        nresults: c_int,
        ctx: lua_KContext,
        k: lua_KFunction
        );

    pub fn lua_close(L: *mut lua_State);

    pub fn lua_getglobal(L: *mut lua_State, name: *const c_char) -> c_int;

    pub fn lua_gettop(L: *mut lua_State) -> c_int;

    pub fn lua_pcallk
        (
        L: *mut lua_State,
        n_args: c_int,
        nresults: c_int,
        errfunc: c_int,
        ctx: lua_KContext,
        k: lua_KFunction
        ) -> c_int;

    pub fn lua_pushcclosure(L: *mut lua_State, f: lua_CFunction, n: c_int);

    pub fn lua_pushlightuserdata(L: *mut lua_State, p: *mut c_void);

    pub fn lua_pushnil(L: *mut lua_State);

    pub fn lua_pushvalue(L: *mut lua_State, idx: c_int);

    pub fn lua_rawgeti(L: *mut lua_State, idx: c_int, n: lua_Integer) -> c_int;

    pub fn lua_rotate(L: *mut lua_State, idx: c_int, n: c_int);

    pub fn lua_setfield(L: *mut lua_State, idx: c_int, k: *const c_char);

    pub fn lua_settop(L: *mut lua_State, idx: c_int);

    pub fn lua_tolstring(L: *mut lua_State, idx: c_int, len: *mut usize) -> *const c_char;

    pub fn lua_touserdata(L: *mut lua_State, idx: c_int) -> *mut c_void;

    pub fn luaL_loadbufferx
        (
        L: *mut lua_State,
        buff: *const c_char,
        size: libc::size_t,
        name: *const c_char,
        mode: *const c_char
        ) -> c_int;

    pub fn luaL_newstate() -> *mut lua_State;

    pub fn luaL_openlibs(L: *mut lua_State);

    pub fn luaL_traceback(L: *mut lua_State, L1: *mut lua_State, msg: *const c_char, level: c_int);
}

pub unsafe fn lua_call(L: *mut lua_State, n: c_int, r: c_int) {
    lua_callk(L, n, r, ptr::null_mut(), ptr::null_mut());
}

pub unsafe fn lua_insert(L: *mut lua_State, idx: c_int) {
    lua_rotate(L, idx, 1);
}

pub unsafe fn lua_pcall(L: *mut lua_State, nargs: c_int, nresults: c_int, errfunc: c_int) -> c_int {
    lua_pcallk(L, nargs, nresults, errfunc, ptr::null_mut(), ptr::null_mut())
}

pub unsafe fn lua_pop(L: *mut lua_State, n: c_int) {
    lua_settop(L, (-n) - 1);
}

pub unsafe fn lua_pushcfunction(L: *mut lua_State, f: lua_CFunction) {
    lua_pushcclosure(L, f, 0);
}

pub unsafe fn lua_pushglobaltable(L: *mut lua_State) {
    lua_rawgeti(L, LUA_REGISTRYINDEX, LUA_RIDX_GLOBALS as lua_Integer);
}

pub unsafe fn lua_remove(L: *mut lua_State, idx: c_int) {
    lua_rotate(L, idx, -1);
    lua_pop(L, 1);
}

pub fn lua_upvalueindex(i: c_int) -> c_int {
    LUA_REGISTRYINDEX - i
}

pub unsafe fn luaL_loadbuffer
    (
    L: *mut lua_State,
    buff: *const c_char,
    size: libc::size_t,
    name: *const c_char
    ) -> c_int
{
    luaL_loadbufferx(
        L,
        buff,
        size,
        name,
        ptr::null()
    )
}