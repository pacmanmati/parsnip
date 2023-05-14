use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum LexerError {
    MalformedHTML,
}

impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: match on self to determine error
        write!(f, "LexerError: {:?}", self)
    }
}

impl Error for LexerError {}

#[derive(Debug, Clone)]
pub enum Token {
    Open(String, String),
    Close(String),
    SelfClose(String, String),
    Inner(String),
}

fn sanitize(html: &str) -> String {
    html.trim().into()
}

pub fn lex(html: &str) -> Result<Vec<Token>, LexerError> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut inner = String::new();
    let mut start = false;
    for c in sanitize(html).chars() {
        if c == '<' {
            // beginning a new tag
            start = true;
            if !inner.is_empty() {
                tokens.push(Token::Inner(std::mem::take(&mut inner)));
            }
        } else if c == '>' {
            // ending a tag
            if token.ends_with('/') {
                let len = token.len();
                let take = std::mem::take(&mut token);
                let mut enclosed = take[0..len - 1].trim().split(' ');

                tokens.push(Token::SelfClose(
                    enclosed.next().unwrap().to_string(),
                    enclosed.collect::<Vec<&str>>().join(" "),
                ));
            } else if token.starts_with('/') {
                tokens.push(Token::Close(
                    std::mem::take(&mut token)[1..].trim().to_string(),
                ));
            } else {
                let take = std::mem::take(&mut token);
                let mut enclosed = take.trim().split(' ');

                tokens.push(Token::Open(
                    enclosed.next().unwrap().to_string(),
                    enclosed.collect::<Vec<&str>>().join(" "),
                ));
            }
            // if !inner.is_empty() {
            //     tokens.push(Token::Inner(std::mem::take(&mut inner)));
            // }
            start = false;
        } else if start {
            // stuff inside of '<' and '>'
            token.push(c);
        } else {
            // stuff before '<'
            inner.push(c);
        }
    }
    dbg!(&tokens);
    Ok(tokens)
}
