use std::collections::HashMap;
use std::collections::VecDeque;



#[derive(PartialEq, Debug)]
pub enum ExecutionStoreError {
    SessionNotFound,
    SessionAlreadyExists,
}

#[derive(PartialEq, Debug)]
struct ExecutionSession {
    id: String,
    pending_inputs: VecDeque<String>,
    executed_inputs: Vec<String>,
    outputs: Vec<String>,
}

#[derive(PartialEq, Debug)]
pub struct ExecutionSessionStore {
    sessions: HashMap<String, ExecutionSession>,
}

impl ExecutionSessionStore {
    pub fn new() -> ExecutionSessionStore {
        ExecutionSessionStore{
            sessions: HashMap::new(),
        }
    }

    pub fn add_session(&mut self, id: &str) -> Result<(), ExecutionStoreError> {
        if self.sessions.contains_key(id) {
            return Err(ExecutionStoreError::SessionAlreadyExists);
        }

        let session = ExecutionSession::new(id);
        self.sessions.insert(String::from(id), session);

        Ok(())
    }

    pub fn add_session_input(&mut self, id: &str, input: &str) -> Result<(), ExecutionStoreError> {
        match self.sessions.get_mut(id) {
            None => Err(ExecutionStoreError::SessionNotFound),
            Some(session) => {
                session.add_input(input);
                Ok(())
            }
        }
    }

    pub fn add_session_outout(&mut self, id: &str, output: &str) -> Result<(), ExecutionStoreError> {
        match self.sessions.get_mut(id) {
            None => Err(ExecutionStoreError::SessionNotFound),
            Some(session) => {
                session.add_output(output);
                Ok(())
            }
        }
    }

    fn lookup_session(&self, id: &str) -> Option<&ExecutionSession> {
        self.sessions.get(id)
    }
}

impl ExecutionSession {
    fn new(id: &str) -> ExecutionSession {
        ExecutionSession {
            id: String::from(id),
            pending_inputs: VecDeque::new(),
            executed_inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    fn add_input(&mut self, input: &str) {
        self.pending_inputs.push_back(String::from(input));
    }

    fn add_output(&mut self, input: &str) {
        self.outputs.push(String::from(input));
    }

    fn get_next_input(&mut self) -> Option<String> {
        let next_input = self.pending_inputs.pop_front();
        if let Some(ref input_line) = next_input {
            self.executed_inputs.push(input_line.clone());
        }

        next_input
    }
}

#[cfg(test)]
mod session_tests {
    use super::*;

    #[test]
    fn test_get_next_input() {
        let id = "abc";
        let input = "print('Hello, World!')";
        let mut session = ExecutionSession::new(id);

        session.add_input(input);

        let actual_input = session.get_next_input();
        assert_eq!(Some(String::from(input)), actual_input);

        // Should be empty now
        assert_eq!(None, session.get_next_input());
    }

    #[test]
    fn test_get_next_input_empty() {
        let id = "abc";
        let mut session = ExecutionSession::new(id);

        let actual_input = session.get_next_input();
        assert_eq!(None, actual_input);
    }
}

#[cfg(test)]
mod store_tests {
    use super::*;

    #[test]
    fn test_add_session() {
        let mut store = ExecutionSessionStore::new();
        let id = "abcd";

        let actual_result = store.add_session(id);
        assert_eq!(Ok(()), actual_result);

        let expected_session = ExecutionSession::new(id);
        let actual_session = store.lookup_session(id);
        assert_eq!(Some(&expected_session), actual_session);
    }

    #[test]
    fn test_add_session_already_exists() {
        let mut store = ExecutionSessionStore::new();
        let id = "efgh";

        store.add_session(id);

        let actual_result = store.add_session(id);
        assert_eq!(Err(ExecutionStoreError::SessionAlreadyExists), actual_result);
    }
}