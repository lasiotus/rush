#[derive(PartialEq, Eq, Debug)]
enum State {
    Normal,
    Quoted(char),
    Escape, // Last char was '\'
    QuotedEscape(char),
}

impl Default for State {
    fn default() -> Self {
        State::Normal
    }
}

#[derive(Default)]
pub struct LineParser {
    result: Vec<Vec<String>>,
    current_command: Vec<String>,
    current_token: String,

    state: State,
}

impl LineParser {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn process_char(&mut self, c: char) {
        match self.state {
            State::Normal => {
                if c.is_whitespace() {
                    self.finish_token();
                } else if c == '|' {
                    self.finish_command();
                } else if c == '\'' {
                    self.state = State::Quoted(c);
                } else if c == '\"' {
                    self.state = State::Quoted(c);
                } else if c == '\\' {
                    self.state = State::Escape;
                } else {
                    self.current_token.push(c);
                }
            }
            State::Quoted(q) => {
                if c == q {
                    // Consume the quote.
                    self.state = State::Normal;
                } else if c == '\\' {
                    self.state = State::QuotedEscape(q);
                } else {
                    self.current_token.push(c);
                }
            }
            State::Escape => {
                self.current_token.push(c);
                self.state = State::Normal;
            }
            State::QuotedEscape(q) => {
                self.current_token.push(c);
                self.state = State::Quoted(q);
            }
        }
    }

    fn finish_token(&mut self) {
        let token = std::mem::take(&mut self.current_token);
        let trimmed = token.trim();
        if trimmed.is_empty() {
            return;
        }
        self.current_command.push(trimmed.to_owned());
    }

    fn finish_command(&mut self) {
        assert_eq!(self.state, State::Normal);

        self.finish_token();

        if !self.current_command.is_empty() {
            self.result.push(std::mem::take(&mut self.current_command));
        }
    }

    // Parse a line; return a vector of pipelined commands to run, each
    // command represented by a vector of strings, with wildcards resolved.
    pub fn parse_line(&mut self, line: &str) -> Option<Vec<Vec<String>>> {
        for c in line.chars() {
            self.process_char(c);
        }

        match self.state {
            State::Normal => {
                self.finish_command();
                if self.result.is_empty() {
                    None
                } else {
                    Some(std::mem::take(&mut self.result))
                }
            }
            _ => None,
        }
    }
}