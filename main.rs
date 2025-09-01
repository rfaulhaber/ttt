use ttt::source::{Parser, Expr};
use ttt::eval::{Evaluator, TruthTable, EquivalenceCheck, Reduction, EquivalenceDifference};
use miette::{IntoDiagnostic, Result, NamedSource};
use clap::{Parser as ClapParser, Subcommand, ValueEnum};
use std::io::{self, Read};
use serde_json;

#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    /// Human-readable table format (default)
    Table,
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// Nuon format
    Nuon,
}

#[derive(ClapParser)]
#[command(name = "ttt")]
#[command(about = "A command line utility for checking truth tables and optimizing boolean functions")]
#[command(version = "0.1.0")]
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
            let expr_str = get_expression_input(expression)?;
            let expr = parse_expression_with_error_handling(&expr_str)?;
            let table = Evaluator::generate_truth_table(&expr)
                .map_err(|e| miette::miette!("Evaluation error: {:?}", e))?;
            display_truth_table(&table, &cli.output);
        }
        Commands::Equivalence { expressions } => {
            let (left_expr, right_expr) = get_equivalence_expressions(expressions)?;
            let left_parsed = parse_expression_with_error_handling(&left_expr)?;
            let right_parsed = parse_expression_with_error_handling(&right_expr)?;
            let result = Evaluator::check_equivalence(&left_parsed, &right_parsed)
                .map_err(|e| miette::miette!("Evaluation error: {:?}", e))?;
            display_equivalence_result(&result, &left_expr, &right_expr, &cli.output);
        }
        Commands::Reduce { expression } => {
            let expr_str = get_expression_input(expression)?;
            let expr = parse_expression_with_error_handling(&expr_str)?;
            let result = Evaluator::reduce_expression(&expr)
                .map_err(|e| miette::miette!("Reduction error: {:?}", e))?;
            display_reduction_result(&result, &cli.output);
        }
    }
    
    Ok(())
}

fn get_expression_input(args: Vec<String>) -> Result<String> {
    if args.is_empty() {
        // Read from stdin
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).into_diagnostic()?;
        Ok(input.trim().to_string())
    } else {
        // Join arguments with spaces
        Ok(args.join(" "))
    }
}

fn get_equivalence_expressions(expressions: Vec<String>) -> Result<(String, String)> {
    if expressions.len() == 2 {
        Ok((expressions[0].clone(), expressions[1].clone()))
    } else if expressions.is_empty() {
        // Read from stdin - expect two lines
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).into_diagnostic()?;
        let lines: Vec<&str> = input.trim().lines().collect();
        if lines.len() != 2 {
            return Err(miette::miette!("Expected exactly two expressions for equivalence check"));
        }
        Ok((lines[0].to_string(), lines[1].to_string()))
    } else {
        Err(miette::miette!(
            "Equivalence check requires exactly two expressions as arguments"
        ))
    }
}

fn parse_expression_with_error_handling(input: &str) -> Result<Expr> {
    let mut parser = Parser::from_str(input);
    parser.parse().map_err(|e| {
        let named_source = NamedSource::new("expression", input.to_string());
        miette::Report::new(e).with_source_code(named_source)
    })
}

/// Display a truth table in the specified format
fn display_truth_table(table: &TruthTable, format: &OutputFormat) {
    match format {
        OutputFormat::Table => display_truth_table_table(table),
        OutputFormat::Json => display_truth_table_json(table),
        OutputFormat::Csv => display_truth_table_csv(table),
        OutputFormat::Nuon => display_truth_table_nuon(table),
    }
}

/// Display a truth table in human-readable table format
fn display_truth_table_table(table: &TruthTable) {
    // Print header
    for var in table.variables.iter() {
        print!("{:>4}", var);
    }
    println!("{:>8}", "Result");
    
    // Print separator
    for _ in 0..table.variables.len() {
        print!("----");
    }
    println!("--------");
    
    // Print each row
    for row in &table.rows {
        for var in table.variables.iter() {
            let value = row.assignments.get(var).copied().unwrap_or(false);
            print!("{:>4}", if value { "T" } else { "F" });
        }
        println!("{:>8}", if row.result { "T" } else { "F" });
    }
}

/// Display a truth table in JSON format
fn display_truth_table_json(table: &TruthTable) {
    match serde_json::to_string_pretty(table) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}

/// Display a truth table in CSV format
fn display_truth_table_csv(table: &TruthTable) {
    // Print header
    for var in table.variables.iter() {
        print!("{},", var);
    }
    println!("result");
    
    // Print each row
    for row in &table.rows {
        for var in table.variables.iter() {
            let value = row.assignments.get(var).copied().unwrap_or(false);
            print!("{},", if value { "true" } else { "false" });
        }
        println!("{}", if row.result { "true" } else { "false" });
    }
}

/// Display a truth table in nuon format
fn display_truth_table_nuon(table: &TruthTable) {
    println!("[");
    for (i, row) in table.rows.iter().enumerate() {
        print!("  {{");
        
        // Print variable assignments
        for (j, var) in table.variables.iter().enumerate() {
            let value = row.assignments.get(var).copied().unwrap_or(false);
            print!("{}: {}", var, if value { "true" } else { "false" });
            if j < table.variables.len() - 1 {
                print!(", ");
            }
        }
        
        // Print result
        print!(", result: {}", if row.result { "true" } else { "false" });
        print!("}}");
        
        if i < table.rows.len() - 1 {
            println!(",");
        } else {
            println!("");
        }
    }
    println!("]");
}

/// Display the result of an equivalence check
fn display_equivalence_result(check: &EquivalenceCheck, left_str: &str, right_str: &str, format: &OutputFormat) {
    match format {
        OutputFormat::Table => display_equivalence_result_table(check, left_str, right_str),
        OutputFormat::Json => display_equivalence_result_json(check, left_str, right_str),
        OutputFormat::Csv => display_equivalence_result_csv(check, left_str, right_str),
        OutputFormat::Nuon => display_equivalence_result_nuon(check, left_str, right_str),
    }
}

/// Display equivalence result in human-readable format
fn display_equivalence_result_table(check: &EquivalenceCheck, left_str: &str, right_str: &str) {
    if check.equivalent {
        println!("✓ Expressions are equivalent");
        println!("  Left:  {}", left_str);
        println!("  Right: {}", right_str);
    } else {
        println!("✗ Expressions are not equivalent");
        println!("  Left:  {}", left_str);
        println!("  Right: {}", right_str);
        println!("\nDifferences:");
        
        for diff in check.differences.iter().take(5) {
            print!("  ");
            for var in check.variables.iter() {
                let value = diff.assignment.get(var).copied().unwrap_or(false);
                print!("{}={} ", var, if value { "T" } else { "F" });
            }
            println!("→ Left={}, Right={}", 
                    if diff.left_value { "T" } else { "F" },
                    if diff.right_value { "T" } else { "F" });
        }
        
        if check.differences.len() > 5 {
            println!("  ... and {} more differences", check.differences.len() - 5);
        }
    }
}

/// Display equivalence result in JSON format
fn display_equivalence_result_json(check: &EquivalenceCheck, left_str: &str, right_str: &str) {
    #[derive(serde::Serialize)]
    struct EquivalenceOutput {
        equivalent: bool,
        left_expression: String,
        right_expression: String,
        differences: Vec<EquivalenceDifference>,
    }
    
    let output = EquivalenceOutput {
        equivalent: check.equivalent,
        left_expression: left_str.to_string(),
        right_expression: right_str.to_string(),
        differences: check.differences.clone(),
    };
    
    match serde_json::to_string_pretty(&output) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}

/// Display equivalence result in CSV format
fn display_equivalence_result_csv(check: &EquivalenceCheck, left_str: &str, right_str: &str) {
    println!("equivalent,left_expression,right_expression");
    println!("{},{},{}", check.equivalent, left_str, right_str);
    
    if !check.differences.is_empty() {
        println!("\nDifferences:");
        // Print header for differences
        for var in check.variables.iter() {
            print!("{},", var);
        }
        println!("left_value,right_value");
        
        // Print each difference
        for diff in &check.differences {
            for var in check.variables.iter() {
                let value = diff.assignment.get(var).copied().unwrap_or(false);
                print!("{},", if value { "true" } else { "false" });
            }
            println!("{},{}", 
                if diff.left_value { "true" } else { "false" },
                if diff.right_value { "true" } else { "false" });
        }
    }
}

/// Display equivalence result in nuon format
fn display_equivalence_result_nuon(check: &EquivalenceCheck, left_str: &str, right_str: &str) {
    println!("{{");
    println!("  equivalent: {},", if check.equivalent { "true" } else { "false" });
    println!("  left_expression: \"{}\",", left_str);
    println!("  right_expression: \"{}\",", right_str);
    println!("  differences: [");
    
    for (i, diff) in check.differences.iter().enumerate() {
        print!("    {{");
        
        // Print variable assignments
        for (j, var) in check.variables.iter().enumerate() {
            let value = diff.assignment.get(var).copied().unwrap_or(false);
            print!("{}: {}", var, if value { "true" } else { "false" });
            if j < check.variables.len() - 1 {
                print!(", ");
            }
        }
        
        // Print left and right values
        print!(", left_value: {}, right_value: {}", 
            if diff.left_value { "true" } else { "false" },
            if diff.right_value { "true" } else { "false" });
        print!("}}");
        
        if i < check.differences.len() - 1 {
            println!(",");
        } else {
            println!("");
        }
    }
    
    println!("  ]");
    println!("}}");
}

/// Display the result of expression reduction
fn display_reduction_result(reduction: &Reduction, format: &OutputFormat) {
    match format {
        OutputFormat::Table => display_reduction_result_table(reduction),
        OutputFormat::Json => display_reduction_result_json(reduction),
        OutputFormat::Csv => display_reduction_result_csv(reduction),
        OutputFormat::Nuon => display_reduction_result_nuon(reduction),
    }
}

/// Display reduction result in human-readable format
fn display_reduction_result_table(reduction: &Reduction) {
    println!("Expression: {}", reduction.original);
    if reduction.simplified {
        println!("Reduced form: {}", reduction.reduced);
    } else {
        println!("Reduced form: {} (already minimal)", reduction.reduced);
    }
}

/// Display reduction result in JSON format
fn display_reduction_result_json(reduction: &Reduction) {
    match serde_json::to_string_pretty(reduction) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}

/// Display reduction result in CSV format
fn display_reduction_result_csv(reduction: &Reduction) {
    println!("original,reduced,simplified");
    println!("\"{}\",\"{}\",{}", reduction.original, reduction.reduced, reduction.simplified);
}

/// Display reduction result in nuon format
fn display_reduction_result_nuon(reduction: &Reduction) {
    println!("{{");
    println!("  original: \"{}\",", reduction.original);
    println!("  reduced: \"{}\",", reduction.reduced);
    println!("  simplified: {}", if reduction.simplified { "true" } else { "false" });
    println!("}}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_get_expression_input_from_args() {
        let args = vec!["a".to_string(), "and".to_string(), "b".to_string()];
        let result = get_expression_input(args).unwrap();
        assert_eq!(result, "a and b");
    }

    #[test]
    fn test_get_equivalence_expressions_with_two_args() {
        let result = get_equivalence_expressions(
            vec!["a and b".to_string(), "b and a".to_string()]
        ).unwrap();
        assert_eq!(result.0, "a and b");
        assert_eq!(result.1, "b and a");
    }
    
    #[test]
    fn test_get_equivalence_expressions_with_different_args() {
        let result = get_equivalence_expressions(
            vec!["a or b".to_string(), "b or a".to_string()]
        ).unwrap();
        assert_eq!(result.0, "a or b");
        assert_eq!(result.1, "b or a");
    }
    
    #[test]
    fn test_get_equivalence_expressions_error_cases() {
        // Too few expressions
        let result = get_equivalence_expressions(vec!["only one".to_string()]);
        assert!(result.is_err());
        
        // Too many expressions
        let result = get_equivalence_expressions(
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
        display_truth_table(&table, &OutputFormat::Table); // Should not panic
        
        // Test equivalence display
        let variables = Variables::from_expr(&Expr::Identifier("a".to_string())).unwrap();
        let check = EquivalenceCheck {
            equivalent: false,
            variables,
            differences: vec![],
        };
        display_equivalence_result(&check, "a", "not a", &OutputFormat::Table); // Should not panic
        
        // Test reduction display
        use ttt::source::Expr;
        use ttt::eval::TruthTableRow;
        let reduction = Reduction {
            original: Expr::Identifier("a".to_string()),
            reduced: Expr::Identifier("a".to_string()),
            simplified: false,
        };
        display_reduction_result(&reduction, &OutputFormat::Table); // Should not panic
    }
}
