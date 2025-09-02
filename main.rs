use ttt::source::{Parser, Expr};
use ttt::eval::Evaluator;
use ttt::io::output::{OutputFormat, format_truth_table, format_equivalence_result, format_reduction_result};
use ttt::io::input::InputHandler;
use miette::{Result, NamedSource};
use clap::{Parser as ClapParser, Subcommand};


#[derive(ClapParser)]
#[command(name = ttt::config::APP_NAME)]
#[command(about = ttt::config::APP_DESCRIPTION)]
#[command(version = ttt::config::VERSION)]
struct Cli {
    /// Output format
    #[arg(short = 'o', long = "output", value_enum, default_value_t = OutputFormat::Table)]
    output: OutputFormat,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a truth table from a boolean expression
    #[command(name = "table")]
    Table {
        /// Boolean expression (if not provided, reads from stdin)
        expression: Vec<String>,
    },
    /// Check expression equivalency
    #[command(name = "eq")]
    Equivalence {
        /// Two boolean expressions to compare (if not provided, reads from stdin)
        expressions: Vec<String>,
    },
    /// Reduce/simplify an expression
    #[command(name = "reduce")]
    Reduce {
        /// Boolean expression to reduce (if not provided, reads from stdin)
        expression: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Table { expression } => {
            let expr_str = InputHandler::get_single_expression(expression)?;
            let expr = parse_expression_with_error_handling(&expr_str)?;
            let table = Evaluator::generate_truth_table(&expr)
                .map_err(|e| miette::miette!("Truth table generation failed: {}", e))?;
            print!("{}", format_truth_table(&table, &cli.output));
        }
        Commands::Equivalence { expressions } => {
            let (left_expr, right_expr) = InputHandler::get_expression_pair(expressions)?;
            let left_parsed = parse_expression_with_error_handling(&left_expr)?;
            let right_parsed = parse_expression_with_error_handling(&right_expr)?;
            let result = Evaluator::check_equivalence(&left_parsed, &right_parsed)
                .map_err(|e| miette::miette!("Equivalence check failed: {}", e))?;
            print!("{}", format_equivalence_result(&result, &left_expr, &right_expr, &cli.output));
        }
        Commands::Reduce { expression } => {
            let expr_str = InputHandler::get_single_expression(expression)?;
            let expr = parse_expression_with_error_handling(&expr_str)?;
            let result = Evaluator::reduce_expression(&expr)
                .map_err(|e| miette::miette!("Expression reduction failed: {}", e))?;
            print!("{}", format_reduction_result(&result, &cli.output));
        }
    }
    
    Ok(())
}


fn parse_expression_with_error_handling(input: &str) -> Result<Expr> {
    let mut parser = Parser::new(input);
    parser.parse().map_err(|e| {
        let named_source = NamedSource::new("expression", input.to_string());
        miette::Report::new(e).with_source_code(named_source)
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use ttt::eval::{EquivalenceCheck, Reduction, TruthTable};
    
    #[test]
    fn test_input_handler_single_expression() {
        let args = vec!["a".to_string(), "and".to_string(), "b".to_string()];
        let result = InputHandler::get_single_expression(args).unwrap();
        assert_eq!(result, "a and b");
    }

    #[test]
    fn test_input_handler_expression_pair() {
        let result = InputHandler::get_expression_pair(
            vec!["a and b".to_string(), "b and a".to_string()]
        ).unwrap();
        assert_eq!(result.0, "a and b");
        assert_eq!(result.1, "b and a");
    }
    
    #[test]
    fn test_input_handler_expression_pair_different_args() {
        let result = InputHandler::get_expression_pair(
            vec!["a or b".to_string(), "b or a".to_string()]
        ).unwrap();
        assert_eq!(result.0, "a or b");
        assert_eq!(result.1, "b or a");
    }
    
    #[test]
    fn test_input_handler_expression_pair_error_cases() {
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
    fn test_parse_expression_with_error_handling() {
        // Valid expression
        let result = parse_expression_with_error_handling("a and b");
        assert!(result.is_ok());
        
        // Invalid expression should return a miette error
        let result = parse_expression_with_error_handling("a and");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_display_functions_dont_panic() {
        // These tests verify that display functions don't panic
        // We can't easily test output without capturing stdout
        
        // Test truth table display
        use ttt::eval::Variables;
        let variables = Variables::from_expr(&Expr::And(
            Box::new(Expr::Identifier("a".to_string())),
            Box::new(Expr::Identifier("b".to_string()))
        )).unwrap();
        
        let table = TruthTable {
            variables,
            rows: vec![
                TruthTableRow {
                    assignments: {
                        let mut map = HashMap::new();
                        map.insert("a".to_string(), false);
                        map.insert("b".to_string(), false);
                        map
                    },
                    result: false,
                }
            ],
        };
        let _result = format_truth_table(&table, &OutputFormat::Table); // Should not panic
        
        // Test equivalence display
        let variables = Variables::from_expr(&Expr::Identifier("a".to_string())).unwrap();
        let check = EquivalenceCheck {
            equivalent: false,
            variables,
            differences: vec![],
        };
        let _result = format_equivalence_result(&check, "a", "not a", &OutputFormat::Table); // Should not panic
        
        // Test reduction display
        use ttt::source::Expr;
        use ttt::eval::TruthTableRow;
        let reduction = Reduction {
            original: Expr::Identifier("a".to_string()),
            reduced: Expr::Identifier("a".to_string()),
            simplified: false,
        };
        let _result = format_reduction_result(&reduction, &OutputFormat::Table); // Should not panic
    }
}
