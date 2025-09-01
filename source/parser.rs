use crate::source::lexer::{Lexer, Token, SpannedToken, Span};
use std::fmt;
use thiserror::Error;
use miette::{Diagnostic, SourceSpan};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Identifier(String),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
    Implication(Box<Expr>, Box<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Identifier(name) => write!(f, "{}", name),
            Expr::Not(expr) => write!(f, "¬{}", expr),
            Expr::And(left, right) => write!(f, "({} ∧ {})", left, right),
            Expr::Or(left, right) => write!(f, "({} ∨ {})", left, right),
            Expr::Xor(left, right) => write!(f, "({} ⊕ {})", left, right),
            Expr::Implication(left, right) => write!(f, "({} → {})", left, right),
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, found {found}")]
    #[diagnostic(
        code(ttt::parser::unexpected_token),
        help("Try using one of: {expected}")
    )]
    UnexpectedToken {
        expected: String,
        found: String,
        #[label("unexpected token here")]
        span: SourceSpan,
    },
    
    #[error("Unexpected end of input")]
    #[diagnostic(
        code(ttt::parser::unexpected_eof),
        help("The expression appears to be incomplete")
    )]
    UnexpectedEof {
        #[label("expression ends here")]
        span: SourceSpan,
    },
    
    #[error("Invalid expression")]
    #[diagnostic(code(ttt::parser::invalid_expression))]
    InvalidExpression {
        #[label("invalid syntax")]
        span: SourceSpan,
    },
}

pub struct Parser {
    tokens: Vec<SpannedToken>,
    current: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize_spanned();
        Self { tokens, current: 0 }
    }
    
    // Keep from_str as an alias for consistency, but make it just call new
    pub fn from_str(input: &str) -> Self {
        Self::new(input)
    }
    
    fn current_token(&self) -> SpannedToken {
        self.tokens.get(self.current).cloned().unwrap_or_else(|| {
            // Create EOF token at the end of input
            let end_pos = self.tokens.last()
                .map(|t| t.span.end)
                .unwrap_or(0);
            SpannedToken {
                token: Token::Eof,
                span: Span::single(end_pos),
            }
        })
    }
    
    fn advance(&mut self) {
        if self.current < self.tokens.len().saturating_sub(1) {
            self.current += 1;
        }
    }
    
    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let current = self.current_token();
        if std::mem::discriminant(&current.token) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{}", expected),
                found: format!("{}", current.token),
                span: SourceSpan::from(current.span.start..current.span.end),
            })
        }
    }
    
    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_implication()?;
        
        let current = self.current_token();
        if !matches!(current.token, Token::Eof) {
            return Err(ParseError::UnexpectedToken {
                expected: "end of input".to_string(),
                found: format!("{}", current.token),
                span: SourceSpan::from(current.span.start..current.span.end),
            });
        }
        
        Ok(expr)
    }
    
    fn parse_implication(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_or()?;
        
        while matches!(self.current_token().token, Token::Implication) {
            self.advance();
            let right = self.parse_or()?;
            left = Expr::Implication(Box::new(left), Box::new(right));
        }
        
        Ok(left)
    }
    
    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_xor()?;
        
        while matches!(self.current_token().token, Token::Or) {
            self.advance();
            let right = self.parse_xor()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
        
        Ok(left)
    }
    
    fn parse_xor(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        
        while matches!(self.current_token().token, Token::Xor) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Xor(Box::new(left), Box::new(right));
        }
        
        Ok(left)
    }
    
    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        
        while matches!(self.current_token().token, Token::And) {
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }
        
        Ok(left)
    }
    
    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let current = self.current_token();
        match current.token {
            Token::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Not(Box::new(expr)))
            }
            _ => self.parse_primary(),
        }
    }
    
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let current = self.current_token();
        match &current.token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Identifier(name))
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_implication()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            Token::Eof => Err(ParseError::UnexpectedEof {
                span: SourceSpan::from(current.span.start..current.span.end),
            }),
            _ => Err(ParseError::UnexpectedToken {
                expected: "identifier or '('".to_string(),
                found: format!("{}", current.token),
                span: SourceSpan::from(current.span.start..current.span.end),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_identifier() {
        let mut parser = Parser::from_str("a");
        let result = parser.parse().unwrap();
        assert_eq!(result, Expr::Identifier("a".to_string()));
    }
    
    #[test]
    fn test_parse_not() {
        let mut parser = Parser::from_str("not a");
        let result = parser.parse().unwrap();
        assert_eq!(result, Expr::Not(Box::new(Expr::Identifier("a".to_string()))));
    }
    
    #[test]
    fn test_parse_and() {
        let mut parser = Parser::from_str("a and b");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            Expr::And(
                Box::new(Expr::Identifier("a".to_string())),
                Box::new(Expr::Identifier("b".to_string()))
            )
        );
    }
    
    #[test]
    fn test_parse_complex() {
        let mut parser = Parser::from_str("a or not b");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            Expr::Or(
                Box::new(Expr::Identifier("a".to_string())),
                Box::new(Expr::Not(Box::new(Expr::Identifier("b".to_string()))))
            )
        );
    }
    
    #[test]
    fn test_parse_with_parentheses() {
        let mut parser = Parser::from_str("(a or b) and c");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            Expr::And(
                Box::new(Expr::Or(
                    Box::new(Expr::Identifier("a".to_string())),
                    Box::new(Expr::Identifier("b".to_string()))
                )),
                Box::new(Expr::Identifier("c".to_string()))
            )
        );
    }
    
    #[test]
    fn test_operator_precedence() {
        let mut parser = Parser::from_str("a or b and c");
        let result = parser.parse().unwrap();
        // Should parse as: a or (b and c)
        assert_eq!(
            result,
            Expr::Or(
                Box::new(Expr::Identifier("a".to_string())),
                Box::new(Expr::And(
                    Box::new(Expr::Identifier("b".to_string())),
                    Box::new(Expr::Identifier("c".to_string()))
                ))
            )
        );
    }
    
    #[test]
    fn test_implication() {
        let mut parser = Parser::from_str("a -> b");
        let result = parser.parse().unwrap();
        assert_eq!(
            result,
            Expr::Implication(
                Box::new(Expr::Identifier("a".to_string())),
                Box::new(Expr::Identifier("b".to_string()))
            )
        );
    }
}