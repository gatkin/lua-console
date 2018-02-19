use std::io::{Stdout, Write, stdin, stdout};

use termion;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use lua::{LuaError, LuaIO, LuaState};


/// External events to update the state of the REPL and perform effects.
#[derive(PartialEq, Debug)]
enum Msg {
    AddChar(char),
    Backspace,
    ClearScreen,
    ExecutionCompleted(Result<Vec<String>, LuaError>),
    GoBackInHistory,
    GoForwardInHistory,
    Quit,
    ResetInput,
    Submit,
}


/// Descriptions of side effects to be performed.
#[derive(PartialEq, Debug)]
enum Cmd {
    ClearScreen,
    DisplayErrorMessage(String),
    DisplayOutput(String),
    ExecuteChunk(String),
    None,
    Quit,
}


/// Contains the state of the REPL.
struct Repl {
    input_buffer: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
    input_history_index: Option<usize>,
}


/// A REPL executor that can read and write from the console.
pub struct ConsoleRepl {
    lua_state: LuaState,
    repl: Repl,
    stdout: RawTerminal<Stdout>,
}


/// Receives output generated by executing Lua chunks.
struct ConsoleIOReceiver<'a> {
    stdout: &'a mut RawTerminal<Stdout>,
}


impl Repl {
    fn new() -> Repl {
        Repl{
            input_buffer: String::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            input_history_index: None,
        }
    }

    /// Updates the REPL's state in response to the give message by mutating the
    /// REPL in palce. Returns a command describing an effect to be performed.
    fn update(&mut self, msg: Msg) -> Cmd {
        match msg {
            Msg::AddChar(c) => self.on_add_char(c),
            Msg::Backspace => self.on_backspace(),
            Msg::ClearScreen => self.on_clear_screen(),
            Msg::ExecutionCompleted(Ok(return_values)) => self.on_values_returned(return_values),
            Msg::ExecutionCompleted(Err(error)) => self.on_execution_error(error),
            Msg::GoBackInHistory => self.on_go_back_in_history(),
            Msg::GoForwardInHistory => self.on_go_forward_in_history(),
            Msg::ResetInput => self.on_reset_input(),
            Msg::Quit => self.on_quit(),
            Msg::Submit => self.on_submit(),
        }
    }

    fn on_add_char(&mut self, c: char) -> Cmd {
        self.input_buffer.push(c);
        Cmd::None
    }

    fn on_clear_screen(&mut self) -> Cmd {
        Cmd::ClearScreen
    }

    fn on_backspace(&mut self) -> Cmd {
        self.input_buffer.pop();
        Cmd::None
    }

    fn on_execution_error(&mut self, error: LuaError) -> Cmd {
        Cmd::DisplayErrorMessage(error.message)
    }

    fn on_go_back_in_history(&mut self) -> Cmd {
        // TODO
        Cmd::None
    }

    fn on_go_forward_in_history(&mut self) -> Cmd {
        // TODO
        Cmd::None
    }

    fn on_reset_input(&mut self) -> Cmd {
        self.input_buffer.clear();
        Cmd::None
    }

    fn on_quit(&mut self) -> Cmd {
        Cmd::Quit
    }

    fn on_submit(&mut self) -> Cmd {
        let chunk = self.input_buffer.clone();
        self.input_buffer.clear();
        Cmd::ExecuteChunk(chunk)
    }

    fn on_values_returned(&mut self, mut values: Vec<String>) -> Cmd {
        let mut output_display = String::new();
        for value in &values {
            output_display.push_str(value);
            output_display.push_str("   ");
        }
        
        self.outputs.append(&mut values);
        Cmd::DisplayOutput(output_display)
    }
}


impl ConsoleRepl {
    pub fn new() -> ConsoleRepl {
        ConsoleRepl{
            lua_state: LuaState::new(),
            repl: Repl::new(),
            stdout: stdout().into_raw_mode().unwrap(),
        }
    }

    /// Runs the REPL reading and writing from standard in and standard out.
    pub fn run_repl(&mut self) {
        write!(self.stdout, "\r/> ").unwrap();
        self.stdout.flush().unwrap();

        let key_messages = stdin().keys()
            .into_iter()
            .filter_map(|key| key_to_message(&key.unwrap()));

        for msg in key_messages {
            let cmd = self.repl.update(msg);
            match cmd {
                Cmd::ClearScreen => self.on_clear_screen(),
                Cmd::DisplayErrorMessage(error) => self.on_display_error_message(error),
                Cmd::DisplayOutput(output) => self.on_display_output(output),
                Cmd::ExecuteChunk(chunk) => self.on_execute_chunk(chunk),
                Cmd::None => self.render_input_buffer(),
                Cmd::Quit => break,
            }
        }

        write!(self.stdout, "\r\nGoodbye!\r\n").unwrap();
        self.stdout.flush().unwrap();
    }

    fn on_clear_screen(&mut self) {
        write!(self.stdout, "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)).unwrap();
        self.render_input_buffer();
    }

    fn on_display_error_message(&mut self, error: String) {
        write!(self.stdout, "\r\n").unwrap();
        if error.len() > 0 {
            write!(self.stdout, "\r\n{}\r\n", error).unwrap();
        }

        self.render_input_buffer();
    }

    fn on_display_output(&mut self, output: String) {
        write!(self.stdout, "\r\n").unwrap();
        if output.len() > 0 {
            write!(self.stdout, "{}\r\n", output).unwrap();
        }

        self.render_input_buffer();
    }

    fn on_execute_chunk(&mut self, chunk: String) {
        let result = {
            let mut io_receiver = ConsoleIOReceiver{ stdout: &mut self.stdout };
            self.lua_state.execute_chunk(&chunk, &mut io_receiver)
        };

       let cmd = self.repl.update(Msg::ExecutionCompleted(result));
       if let Cmd::DisplayErrorMessage(error) = cmd {
           self.on_display_error_message(error);
       } else if let Cmd::DisplayOutput(output) = cmd {
           self.on_display_output(output);
       }
    }

    fn render_input_buffer(&mut self) {
        write!(self.stdout, "{}\r/> {}",
            termion::clear::CurrentLine,
            self.repl.input_buffer).unwrap();
        self.stdout.flush().unwrap();
    }
}


impl<'a> LuaIO for ConsoleIOReceiver<'a> {
    fn on_print (&mut self, values: Vec<String>) {
        for value in &values {
            write!(self.stdout, "\r\n{}", value).unwrap();
        }
        self.stdout.flush().unwrap();
    }
}


/// Converts the given console key event to the corresponding message. Returns None
/// if the key event is not supported.
fn key_to_message(key: &Key) -> Option<Msg> {
    match key {
        &Key::Ctrl('c') => Some(Msg::ResetInput),
        &Key::Ctrl('l') => Some(Msg::ClearScreen),
        &Key::Ctrl('z') => Some(Msg::Quit),
        &Key::Backspace => Some(Msg::Backspace),
        &Key::Down => Some(Msg::GoForwardInHistory),
        &Key::Up => Some(Msg::GoBackInHistory),
        &Key::Char('\n') => Some(Msg::Submit),
        &Key::Char(c) => Some(Msg::AddChar(c)),
        _ => None,
    }
}
