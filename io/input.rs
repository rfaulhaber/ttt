use std::io::{self, Read};
use miette::{IntoDiagnostic, Result};

/// Generic input handler for CLI arguments and stdin
pub struct InputHandler;

impl InputHandler {
    /// Get a single expression from command line args or stdin
    pub fn get_single_expression(args: Vec<String>) -> Result<String> {
        if args.is_empty() {
            Self::read_from_stdin()
        } else {
            Ok(args.join(" "))
        }
    }
    
    /// Get exactly two expressions for equivalence checking
    pub fn get_expression_pair(expressions: Vec<String>) -> Result<(String, String)> {
        match expressions.len() {
            2 => Ok((expressions[0].clone(), expressions[1].clone())),
            0 => {
                // Read from stdin - expect two lines
                let input = Self::read_from_stdin()?;
                let lines: Vec<&str> = input.trim().lines().collect();
                if lines.len() != 2 {
                    return Err(miette::miette!(
                        "Expected exactly two expressions for equivalence check (one per line), got {}.\nPlease provide two expressions separated by newlines or as separate arguments.", 
                        lines.len()
                    ));
                }
                Ok((lines[0].to_string(), lines[1].to_string()))
            }
            _ => Err(miette::miette!(
                "Equivalence check requires exactly two expressions as arguments, got {}.\nUsage: ttt eq \"expr1\" \"expr2\"", 
                expressions.len()
            )),
        }
    }
    
    /// Get N expressions from args or stdin
    pub fn get_multiple_expressions(args: Vec<String>, expected_count: Option<usize>) -> Result<Vec<String>> {
        if args.is_empty() {
            let input = Self::read_from_stdin()?;
            let expressions: Vec<String> = input
                .trim()
                .lines()
                .map(|line| line.to_string())
                .collect();
            
            if let Some(count) = expected_count {
                if expressions.len() != count {
                    return Err(miette::miette!(
                        "Expected exactly {} expressions, got {}", 
                        count, 
                        expressions.len()
                    ));
                }
            }
            
            Ok(expressions)
        } else {
            if let Some(count) = expected_count {
                if args.len() != count {
                    return Err(miette::miette!(
                        "Expected exactly {} expressions as arguments, got {}", 
                        count, 
                        args.len()
                    ));
                }
            }
            Ok(args)
        }
    }
    
    /// Read input from stdin
    fn read_from_stdin() -> Result<String> {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).into_diagnostic()?;
        Ok(input.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_expression_from_args() {
        let args = vec!["a".to_string(), "and".to_string(), "b".to_string()];
        let result = InputHandler::get_single_expression(args).unwrap();
        assert_eq!(result, "a and b");
    }

    #[test]
    fn test_expression_pair_with_two_args() {
        let result = InputHandler::get_expression_pair(
            vec!["a and b".to_string(), "b and a".to_string()]
        ).unwrap();
        assert_eq!(result.0, "a and b");
        assert_eq!(result.1, "b and a");
    }
    
    #[test]
    fn test_expression_pair_error_cases() {
        // Too few expressions
        let result = InputHandler::get_expression_pair(vec!["only one".to_string()]);
        assert!(result.is_err());
        
        // Too many expressions
        let result = InputHandler::get_expression_pair(
            vec!["one".to_string(), "two".to_string(), "three".to_string()]
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_expressions() {
        let args = vec!["expr1".to_string(), "expr2".to_string(), "expr3".to_string()];
        let result = InputHandler::get_multiple_expressions(args, Some(3)).unwrap();
        assert_eq!(result, vec!["expr1", "expr2", "expr3"]);
    }

    #[test]
    fn test_multiple_expressions_count_mismatch() {
        let args = vec!["expr1".to_string(), "expr2".to_string()];
        let result = InputHandler::get_multiple_expressions(args, Some(3));
        assert!(result.is_err());
    }
}