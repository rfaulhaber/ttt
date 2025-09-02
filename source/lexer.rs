use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
    
    pub fn single(pos: usize) -> Self {
        Self { start: pos, end: pos + 1 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Unary operators
    Not,
    
    // Binary operators
    And,
    Or,
    Xor,
    Implication,
    
    // Identifiers
    Identifier(String),
    
    // Delimiters
    LeftParen,
    RightParen,
    
    // End of input
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Not => write!(f, "NOT"),
            Token::And => write!(f, "AND"),
            Token::Or => write!(f, "OR"),
            Token::Xor => write!(f, "XOR"),
            Token::Implication => write!(f, "IMPL"),
            Token::Identifier(name) => write!(f, "{}", name),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug)]
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.first().copied();
        
        Self {
            input: chars,
            position: 0,
            current_char,
        }
    }
    
    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }
    
    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn read_identifier(&mut self) -> (String, Span) {
        let start = self.position;
        let mut result = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_alphabetic() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        (result, Span::new(start, self.position))
    }
    
    fn read_symbol(&mut self) -> Option<(Token, Span)> {
        let start = self.position;
        match self.current_char? {
            '-' if self.peek() == Some('>') => {
                self.advance(); // consume '-'
                self.advance(); // consume '>'
                Some((Token::Implication, Span::new(start, self.position)))
            }
            // Unicode arrow: →
            '\u{2192}' => {
                self.advance();
                Some((Token::Implication, Span::new(start, self.position)))
            }
            '&' if self.peek() == Some('&') => {
                self.advance(); // consume first '&'
                self.advance(); // consume second '&'
                Some((Token::And, Span::new(start, self.position)))
            }
            // Unicode and: ∧
            '\u{2227}' => {
                self.advance();
                Some((Token::And, Span::new(start, self.position)))
            }
            '|' if self.peek() == Some('|') => {
                self.advance(); // consume first '|'
                self.advance(); // consume second '|'
                Some((Token::Or, Span::new(start, self.position)))
            }
            // Unicode or: ∨
            '\u{2228}' => {
                self.advance();
                Some((Token::Or, Span::new(start, self.position)))
            }
            '!' => {
                self.advance();
                Some((Token::Not, Span::new(start, self.position)))
            }
            // Unicode not: ¬
            '\u{00AC}' => {
                self.advance();
                Some((Token::Not, Span::new(start, self.position)))
            }
            // Unicode xor: ⊻ or ⊕
            c if c == '\u{22BB}' || c == '\u{2295}' => {
                self.advance();
                Some((Token::Xor, Span::new(start, self.position)))
            }
            '(' => {
                self.advance();
                Some((Token::LeftParen, Span::new(start, self.position)))
            }
            ')' => {
                self.advance();
                Some((Token::RightParen, Span::new(start, self.position)))
            }
            _ => None,
        }
    }
    
    pub fn next_spanned_token(&mut self) -> SpannedToken {
        loop {
            self.skip_whitespace();
            
            match self.current_char {
                None => return SpannedToken {
                    token: Token::Eof,
                    span: Span::single(self.position),
                },
                Some(ch) if ch.is_alphabetic() => {
                    let (identifier, span) = self.read_identifier();
                    let token = match identifier.as_str() {
                        "and" => Token::And,
                        "or" => Token::Or,
                        "not" => Token::Not,
                        "xor" => Token::Xor,
                        _ => Token::Identifier(identifier),
                    };
                    return SpannedToken { token, span };
                }
                Some(_) => {
                    if let Some((token, span)) = self.read_symbol() {
                        return SpannedToken { token, span };
                    } else {
                        // Skip unknown character and continue
                        self.advance();
                        continue;
                    }
                }
            }
        }
    }
    
    pub fn next_token(&mut self) -> Token {
        self.next_spanned_token().token
    }
    
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token();
            let is_eof = matches!(token, Token::Eof);
            tokens.push(token);
            
            if is_eof {
                break;
            }
        }
        
        tokens
    }
    
    pub fn tokenize_spanned(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();
        
        loop {
            let spanned_token = self.next_spanned_token();
            let is_eof = matches!(spanned_token.token, Token::Eof);
            tokens.push(spanned_token);
            
            if is_eof {
                break;
            }
        }
        
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_word_operators() {
        let test_cases = [
            ("and", vec![Token::And, Token::Eof]),
            ("or", vec![Token::Or, Token::Eof]),
            ("not", vec![Token::Not, Token::Eof]),
            ("xor", vec![Token::Xor, Token::Eof]),
        ];
        
        for (input, expected) in test_cases {
            let mut lexer = Lexer::new(input);
            let tokens = lexer.tokenize();
            assert_eq!(tokens, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_symbol_operators() {
        let test_cases = [
            ("&&", vec![Token::And, Token::Eof]),
            ("||", vec![Token::Or, Token::Eof]),
            ("!", vec![Token::Not, Token::Eof]),
            ("->", vec![Token::Implication, Token::Eof]),
        ];
        
        for (input, expected) in test_cases {
            let mut lexer = Lexer::new(input);
            let tokens = lexer.tokenize();
            assert_eq!(tokens, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_unicode_operators() {
        let test_cases = [
            ("∧", vec![Token::And, Token::Eof]),
            ("∨", vec![Token::Or, Token::Eof]),
            ("¬", vec![Token::Not, Token::Eof]),
            ("→", vec![Token::Implication, Token::Eof]),
            ("⊻", vec![Token::Xor, Token::Eof]),
            ("⊕", vec![Token::Xor, Token::Eof]),
        ];
        
        for (input, expected) in test_cases {
            let mut lexer = Lexer::new(input);
            let tokens = lexer.tokenize();
            assert_eq!(tokens, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_identifiers() {
        let test_cases = [
            ("a", vec![Token::Identifier("a".to_string()), Token::Eof]),
            ("variable", vec![Token::Identifier("variable".to_string()), Token::Eof]),
            ("var_name", vec![Token::Identifier("var_name".to_string()), Token::Eof]),
        ];
        
        for (input, expected) in test_cases {
            let mut lexer = Lexer::new(input);
            let tokens = lexer.tokenize();
            assert_eq!(tokens, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_complex_expression() {
        let mut lexer = Lexer::new("a and b or not c");
        let tokens = lexer.tokenize();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("a".to_string()),
                Token::And,
                Token::Identifier("b".to_string()),
                Token::Or,
                Token::Not,
                Token::Identifier("c".to_string()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_whitespace_handling() {
        let inputs = [
            "a and b",
            "  a   and   b  ",
            "\ta\nand\r\nb\t",
        ];
        
        let expected = vec![
            Token::Identifier("a".to_string()),
            Token::And,
            Token::Identifier("b".to_string()),
            Token::Eof,
        ];
        
        for input in inputs {
            let mut lexer = Lexer::new(input);
            let tokens = lexer.tokenize();
            assert_eq!(tokens, expected, "Failed for input: {:?}", input);
        }
    }
}