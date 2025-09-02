use ttt::source::Parser;
use ttt::eval::Evaluator;
use std::collections::HashMap;

/// Test the full workflow from parsing to evaluation
#[test]
fn test_truth_table_workflow() {
    // Parse expression
    let mut parser = Parser::new("a and b");
    let expr = parser.parse().expect("Should parse successfully");
    
    // Generate truth table
    let table = Evaluator::generate_truth_table(&expr).unwrap();
    
    // Verify structure
    assert_eq!(table.variables.to_vec(), vec!["a", "b"]);
    assert_eq!(table.rows.len(), 4);
    
    // Verify specific truth table values for AND
    let all_false_row = table.rows.iter()
        .find(|row| !row.assignments["a"] && !row.assignments["b"])
        .expect("Should have F,F row");
    assert!(!all_false_row.result);
    
    let all_true_row = table.rows.iter()
        .find(|row| row.assignments["a"] && row.assignments["b"])
        .expect("Should have T,T row");
    assert!(all_true_row.result);
}

#[test]
fn test_equivalence_check_workflow() {
    // Parse expressions
    let mut parser1 = Parser::new("a and b");
    let mut parser2 = Parser::new("b and a");
    let expr1 = parser1.parse().expect("Should parse left expression");
    let expr2 = parser2.parse().expect("Should parse right expression");
    
    // Check equivalence
    let check = Evaluator::check_equivalence(&expr1, &expr2).unwrap();
    
    // These should be equivalent (commutativity)
    assert!(check.equivalent);
    assert!(check.differences.is_empty());
    assert_eq!(check.variables.len(), 2);
    assert!(check.variables.contains(&"a".to_string()));
    assert!(check.variables.contains(&"b".to_string()));
}

#[test]
fn test_non_equivalent_expressions() {
    // Parse different expressions
    let mut parser1 = Parser::new("a and b");
    let mut parser2 = Parser::new("a or b");
    let expr1 = parser1.parse().expect("Should parse left expression");
    let expr2 = parser2.parse().expect("Should parse right expression");
    
    // Check equivalence
    let check = Evaluator::check_equivalence(&expr1, &expr2).unwrap();
    
    // These should not be equivalent
    assert!(!check.equivalent);
    assert!(!check.differences.is_empty());
    
    // Verify some differences exist
    for diff in &check.differences {
        let left_val = Evaluator::evaluate_with_assignment(&expr1, &diff.assignment);
        let right_val = Evaluator::evaluate_with_assignment(&expr2, &diff.assignment);
        assert_eq!(left_val, diff.left_value);
        assert_eq!(right_val, diff.right_value);
        assert_ne!(left_val, right_val);
    }
}

#[test]
fn test_reduction_workflow() {
    // Parse expression
    let mut parser = Parser::new("a or not a");
    let expr = parser.parse().expect("Should parse expression");
    
    // Attempt reduction
    let reduction = Evaluator::reduce_expression(&expr).unwrap();
    
    // Verify structure 
    assert_eq!(reduction.original, expr);
    // Note: reduced form may differ but should be equivalent
}

#[test]
fn test_complex_expression_workflow() {
    // Test complex expression with multiple operators and precedence
    let mut parser = Parser::new("(a or b) and (not c -> d)");
    let expr = parser.parse().expect("Should parse complex expression");
    
    // Generate truth table
    let table = Evaluator::generate_truth_table(&expr).unwrap();
    
    // Should have 4 variables
    let expected_vars = vec!["a", "b", "c", "d"];
    for var in &expected_vars {
        assert!(table.variables.contains(&var.to_string()));
    }
    
    // Should have 2^4 = 16 rows
    assert_eq!(table.rows.len(), 16);
    
    // Verify each row computes correctly
    for row in &table.rows {
        let computed_result = Evaluator::evaluate_with_assignment(&expr, &row.assignments);
        assert_eq!(computed_result, row.result, 
                  "Row result mismatch for assignment: {:?}", row.assignments);
    }
}

#[test]
fn test_all_operator_types() {
    let test_cases = [
        ("a and b", "conjunction"),
        ("a or b", "disjunction"), 
        ("not a", "negation"),
        ("a xor b", "exclusive or"),
        ("a -> b", "implication"),
        ("a ∧ b", "unicode conjunction"),
        ("a ∨ b", "unicode disjunction"),
        ("¬a", "unicode negation"),
        ("a ⊕ b", "unicode xor"),
        ("a → b", "unicode implication"),
    ];
    
    for (expr_str, description) in test_cases {
        let mut parser = Parser::new(expr_str);
        let expr = parser.parse().expect(&format!("Should parse {}", description));
        
        // Verify we can generate truth table
        let table = Evaluator::generate_truth_table(&expr).unwrap();
        assert!(!table.rows.is_empty(), "Should have truth table rows for {}", description);
        
        // Verify all rows evaluate consistently
        for row in &table.rows {
            let computed = Evaluator::evaluate_with_assignment(&expr, &row.assignments);
            assert_eq!(computed, row.result, "Evaluation mismatch for {} with {:?}", 
                      description, row.assignments);
        }
    }
}

#[test]
fn test_variable_collection() {
    let test_cases = [
        ("a", vec!["a"]),
        ("a and b", vec!["a", "b"]),
        ("not x", vec!["x"]),
        ("(p or q) and (r -> s)", vec!["p", "q", "r", "s"]),
        ("a and a", vec!["a"]), // Duplicates should be handled
    ];
    
    for (expr_str, expected_vars) in test_cases {
        let mut parser = Parser::new(expr_str);
        let expr = parser.parse().expect(&format!("Should parse {}", expr_str));
        
        let collected = Evaluator::collect_expression_variables(&expr).unwrap();
        
        assert_eq!(collected.len(), expected_vars.len(), 
                  "Variable count mismatch for {}", expr_str);
        
        for expected_var in expected_vars {
            assert!(collected.contains(expected_var), 
                   "Should contain variable '{}' for expression '{}'", expected_var, expr_str);
        }
    }
}

#[test]
fn test_expression_evaluation_with_assignments() {
    let test_cases = [
        ("a", vec![("a", true)], true),
        ("a", vec![("a", false)], false),
        ("a and b", vec![("a", true), ("b", true)], true),
        ("a and b", vec![("a", true), ("b", false)], false),
        ("a or b", vec![("a", false), ("b", true)], true),
        ("not a", vec![("a", true)], false),
        ("a -> b", vec![("a", false), ("b", false)], true), // false -> false is true
        ("a xor b", vec![("a", true), ("b", true)], false),
    ];
    
    for (expr_str, assignments, expected_result) in test_cases {
        let mut parser = Parser::new(expr_str);
        let expr = parser.parse().expect(&format!("Should parse {}", expr_str));
        
        let mut assignment_map = HashMap::new();
        for (var, value) in assignments {
            assignment_map.insert(var.to_string(), value);
        }
        
        let result = Evaluator::evaluate_with_assignment(&expr, &assignment_map);
        assert_eq!(result, expected_result, 
                  "Evaluation mismatch for '{}' with assignment {:?}", 
                  expr_str, assignment_map);
    }
}

#[test]
fn test_error_handling_in_workflow() {
    // Test that parsing errors are handled gracefully
    let invalid_expressions = [
        "a and", 
        "not",
        "a b", // missing operator
        "(a or", // unclosed paren
        "a +++ b", // invalid operator
    ];
    
    for invalid_expr in invalid_expressions {
        let mut parser = Parser::new(invalid_expr);
        let result = parser.parse();
        
        assert!(result.is_err(), 
               "Should fail to parse invalid expression: '{}'", invalid_expr);
    }
}