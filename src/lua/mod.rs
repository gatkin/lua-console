#![allow(non_snake_case)]
mod ffi;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use libc;

use lua::ffi::*;


/// Status codes returned by the Lua virtual machine.
#[derive(PartialEq, Debug)]
pub enum LuaRcode {
    Ok,
    Yield,
    ErrRun,
    ErrSyntax,
    ErrMem,
    ErrGcmm,
    ErrErr,
    ErrInvalid,
}


/// Possible errors with compiling and executing Lua chunks
#[derive(PartialEq, Debug)]
pub enum LuaErrorStatus {
    Yield,
    RuntimeError,
    SyntaxError,
    InternalError,
}


/// Trait used to respond to output generated by an executing Lua chunk.
pub trait LuaIO {
    /// Invoked whenever "print" is called in Lua with all arguments converted to strings.
    fn on_print(&mut self, values: Vec<String>);
}


/// Provides a safe handle to the lua_State structure used in the
/// Lua C API.
pub struct LuaState {
    state: *mut lua_State,
}


/// Represents errors executing and compiling Lua chunks.
#[derive(PartialEq, Debug)]
pub struct LuaError {
    status: LuaErrorStatus,
    message: String,
}


/// Container to hold a reference to a LuaIO trait object. In order to redirect all output
/// written to standard out by an executing Lua script to an IO receiver, the standard Lua print
/// function is redefined in Rust. In order to send output values to the IO receiver, the Rust
/// print function must somehow have access to the particualr IO receiver for the Lua state that
/// is executing. This is done by storing a raw pointer to the reveiver as a light userdata object
/// as an up-value to the custom print function. Since trait objects are "fat" pointers, they cannot
/// be cast between raw C pointers. Therefore, the IO receiver is placed into a container which we
/// can then access in the print function as one of the function's up values.
struct LuaIOBox<'a> {
    io: &'a mut LuaIO,
}

/// Handle to provide RAII semantics for managing the registration and unregistration of IO
/// receivers with the Lua run time.
struct IORegistrationHandle<'a> {
    io: *mut LuaIOBox<'a>,
    L: *mut lua_State,
}


impl LuaState {
    /// Creates and configures a new Lua state that can be used to execute
    /// Lua chunks.
    pub fn new() -> LuaState {
        let state = unsafe{ luaL_newstate() };
        unsafe{ luaL_openlibs(state) };

        LuaState{
            state,
        }
    }

    /// Executes the given Lua chunk, and returns any values left on the stack converted to
    /// their string representations.
    pub fn execute_chunk(&self, chunk: &str, io: &mut LuaIO) -> Result<Vec<String>, LuaError> {
        let _io_handle = IORegistrationHandle::new(self.state, io);
        
        let initial_stack = unsafe{ lua_gettop(self.state) };
        let mut rcode = compile_chunk(self.state, chunk);

        if rcode == LuaRcode::Ok {
            rcode = unsafe{ execute_compiled_chunk(self.state) };
        }

        let num_stack_values = unsafe{ lua_gettop(self.state) } - initial_stack;

        let exctn_result = if rcode == LuaRcode::Ok {
            let stack_values = unsafe{ dump_stack(self.state, num_stack_values) };

            // Remove all of the returned values and the compiled chunk from the stack.
            unsafe{ lua_pop(self.state, num_stack_values + 1) };
            Ok(stack_values)
        } else {
            let error = unsafe{ get_execution_error(self.state, rcode) };
            unsafe{ lua_pop(self.state, num_stack_values) };
            Err(error)
        };

        exctn_result
    }
}


impl Drop for LuaState {
    fn drop(&mut self) {
        unsafe {
            lua_close(self.state);
        }
    }
}


impl LuaRcode {
    /// Converts a raw integer representing a Lua return code into a proper enum value.
    fn from_raw_rcode(rcode: c_int) -> LuaRcode {
        match rcode {
            0 => LuaRcode::Ok,
            1 => LuaRcode::Yield,
            2 => LuaRcode::ErrRun,
            3 => LuaRcode::ErrSyntax,
            4 => LuaRcode::ErrMem,
            5 => LuaRcode::ErrGcmm,
            6 => LuaRcode::ErrErr,
            _ => LuaRcode::ErrInvalid,
        }
    }
}


impl<'a> IORegistrationHandle<'a> {
    fn new(L: *mut lua_State, io: &'a mut LuaIO) -> IORegistrationHandle<'a> {
        let io_ptr = Box::into_raw(Box::new(LuaIOBox{ io }));
        unsafe{ register_print(L, io_ptr as *mut c_void); }

        IORegistrationHandle{
            io: io_ptr,
            L,
        }
    }
}


impl<'a> Drop for IORegistrationHandle<'a> {
    fn drop(&mut self) {
        // For safety, unregister the print function which holds a raw reference to the
        // IO receiver trait object.
        unsafe{ unregister_print(self.L); }
        let _io_container = unsafe{ Box::from_raw(self.io) };
    }
}


/// Compiles the given chunk making it available to be executed as a no argument function
/// on top of the stack.
fn compile_chunk(L: *mut lua_State, chunk: &str) -> LuaRcode {
    let mut rcode = try_add_return(L, chunk);
    if rcode != LuaRcode::Ok {
        rcode = load_string(L, chunk);
    }

    rcode
}


/// Executes a chunk that has been compiled and is on the top of the stack.
unsafe fn execute_compiled_chunk(L: *mut lua_State) -> LuaRcode {
    let base = lua_gettop(L);
    lua_pushcfunction(L, message_handler);
    lua_insert(L, base); // Push our message handler under the function to call

    let rcode = lua_pcall(L, 0, LUA_MULTRET, base);
    lua_remove(L, base); // Remove the message handler from the stack

    LuaRcode::from_raw_rcode(rcode)
}


/// Compiles, but does not execute, the given chunk.
fn load_string(L: *mut lua_State, chunk: &str) -> LuaRcode {
    let rcode = unsafe {
        luaL_loadbuffer(
            L,
            chunk.as_ptr() as *const c_char,
            chunk.len() as libc::size_t,
            ptr::null(),
        )
    };

    LuaRcode::from_raw_rcode(rcode)
}


/// Extracts the specified number of values from the top of the stack
unsafe fn dump_stack(L: *mut lua_State, num_values: i32) -> Vec<String> {
    push_global(L, "tostring");

    let mut values = Vec::with_capacity(num_values as usize);
    for i in 1 .. num_values + 1 {
        lua_pushvalue(L, -1); // Push the tostring function to the top of the stack
        lua_pushvalue(L, i); // Push the ith argument passed to us to the top
        lua_call(L, 1, 1);

        let value = stack_top_to_string(L);
        values.push(value);

        lua_pop(L, 1); // Remove the value from the stack
    }

    values
}


/// Retrives the value at the top of the stack and converts it to a string representation.
unsafe fn dump_stack_top(L: *mut lua_State) -> String {
    push_global(L, "tostring");
    lua_pushvalue(L, -2);
    lua_call(L, 1, 1);
    let value = stack_top_to_string(L);

    lua_pop(L, 1);

    value
}


/// Retrieves all error information from the stack after an error is encountered in either the compiliation
/// or execution of a chunk.
unsafe fn get_execution_error(L: *mut lua_State, rcode: LuaRcode) -> LuaError {

    let error_status = match rcode {
        LuaRcode::Yield => LuaErrorStatus::Yield,
        LuaRcode::ErrSyntax => LuaErrorStatus::SyntaxError,
        LuaRcode::ErrRun => LuaErrorStatus::RuntimeError,
        _ => LuaErrorStatus::InternalError,
    };

    LuaError {
        status: error_status,
        message: dump_stack_top(L),
    }
}


/// Custom message handler invoked by the Lua runtime whenever an error is encountered
/// executing a chunk.
unsafe extern "C" fn message_handler(L: *mut lua_State) -> c_int {
    let msg = lua_tolstring(L, 1, ptr::null_mut());
    luaL_traceback(L, L, msg, 1); // Append a traceback to the message
    LUA_ERRUN
}


/// Pushes the given global identifier to the top of the stack.
unsafe fn push_global(L: *mut lua_State, name: &str) {
    let to_string_name = CString::new(name).unwrap();
    lua_getglobal(L, to_string_name.as_ptr());
}


/// Registers the custom print function as the default Lua print function. The given
/// pointer is made available in the print as a light userdata value as one of the
/// print function's up values.
unsafe fn register_print(L: *mut lua_State, io_userdata: *mut c_void) {
    lua_pushglobaltable(L);
    
    // Make a reference to the IO writer available as an up value in the
    // print function.
    lua_pushlightuserdata(L, io_userdata);
    lua_pushcclosure(L, print, 1);

    // Set the "print" value in the global table to our custom print function
    let name = CString::new("print").unwrap();
    lua_setfield(L, -2, name.as_ptr());

    lua_pop(L, 1); // Pop the global table from the stack
}


/// Custom print function that replaces the default Lua print function. This is invoked from the
/// Lua library C code whenver the Lua function print is called.
unsafe extern "C" fn print(L: *mut lua_State) -> c_int {
    // Load the printer we saved in this function's closure
    let io_idx = lua_upvalueindex(1);
    let raw_io_ptr = lua_touserdata(L, io_idx);
    let io_box = &mut *(raw_io_ptr as *mut LuaIOBox);
    
    let num_params = lua_gettop(L);
    let values = dump_stack(L, num_params);
    io_box.io.on_print(values);

    LUA_OK
}


/// Retrieves the string from the top of the stack.
unsafe fn stack_top_to_string(L: *mut lua_State) -> String {
    let raw_value = lua_tolstring(L, -1, ptr::null_mut());
    String::from(CStr::from_ptr(raw_value).to_str().unwrap())
}


/// Attempts to turn the given chunk into an expression by adding a "return" in
/// front of it. Returns the status code from compiling the chunk with a return
fn try_add_return(L: *mut lua_State, chunk: &str) -> LuaRcode {
    let mut with_return = String::from("return ");
    with_return.push_str(chunk);
    let rcode = load_string(L, &with_return);

    if LuaRcode::Ok != rcode {
        unsafe { lua_pop(L, 1); } // Pop the result from load buffer
    }

    rcode
}


/// Unregisters the print function.
unsafe fn unregister_print(L: *mut lua_State) {
    // Set the global print functino to nil
    lua_pushglobaltable(L);
    lua_pushnil(L);
    
    let name = CString::new("print").unwrap();
    lua_setfield(L, -2, name.as_ptr());

    lua_pop(L, 1); // Remove the global table from the stack
}
