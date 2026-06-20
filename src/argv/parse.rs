use anyhow::Result;

pub fn parse_argv_text(raw: &str) -> Result<Vec<String>> {
    let parser = ArgvParser::new(raw);
    parser.parse()
}

struct ArgvParser<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    args: Vec<String>,
    current: String,
    quote: Option<char>,
    token_started: bool,
}

impl<'a> ArgvParser<'a> {
    fn new(raw: &'a str) -> Self {
        Self {
            chars: raw.chars().peekable(),
            args: Vec::new(),
            current: String::new(),
            quote: None,
            token_started: false,
        }
    }

    fn parse(mut self) -> Result<Vec<String>> {
        while let Some(character) = self.chars.next() {
            self.consume(character);
        }

        if self.quote.is_some() {
            anyhow::bail!("runtime args contain an unterminated quote");
        }
        if self.token_started {
            self.args.push(self.current);
        }
        Ok(self.args)
    }

    fn consume(&mut self, character: char) {
        match self.quote {
            Some(quote_char) => self.consume_quoted(character, quote_char),
            None if character.is_whitespace() => self.finish_token(),
            None if character == '"' || character == '\'' => self.begin_quote(character),
            None => self.push_character(character),
        }
    }

    fn consume_quoted(&mut self, character: char, quote_char: char) {
        if character == quote_char {
            self.quote = None;
        } else if quote_char == '"' && character == '\\' {
            self.consume_double_quote_escape(character);
        } else {
            self.current.push(character);
        }
        self.token_started = true;
    }

    fn consume_double_quote_escape(&mut self, character: char) {
        match self.chars.peek().copied() {
            Some(next) if next == '"' || next == '\\' => {
                self.current.push(next);
                self.chars.next();
            }
            _ => self.current.push(character),
        }
    }

    fn finish_token(&mut self) {
        if self.token_started {
            self.args.push(std::mem::take(&mut self.current));
            self.token_started = false;
        }
    }

    fn begin_quote(&mut self, quote: char) {
        self.quote = Some(quote);
        self.token_started = true;
    }

    fn push_character(&mut self, character: char) {
        self.current.push(character);
        self.token_started = true;
    }
}
