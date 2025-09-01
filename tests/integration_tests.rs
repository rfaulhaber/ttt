use ttt::source::{Parser, Expr};

#[test]
fn test_parse_workflow() {
    let test_cases = [
        ("a", "a"),
        ("not a", "¬a"),
        ("a and b", "(a ∧ b)"),
        ("a or b", "(a ∨ b)"),
        ("a xor b", "(a ⊕ b)"),
        ("a -> b", "(a → b)"),
        ("(a or b) and c", "((a ∨ b) ∧ c)"),
        ("a or b and c", "(a ∨ (b ∧ c))"), // Test precedence
        ("not a or b", "(¬a ∨ b)"),
    ];
    
    for (input, expected_display) in test_cases {
        let mut parser = Parser::from_str(input);
        let result = parser.parse().expect(&format!("Failed to parse: {}", input));
        assert_eq!(result.to_string(), expected_display, "Input: {}", input);
    }
}

#[test]
fn test_mixed_operator_formats() {
    let equivalent_expressions = [
        ("a and b", "a && b"),
        ("a or b", "a || b"),  
        ("not a", "!a"),
        ("a and b", "a ∧ b"),
        ("a or b", "a ∨ b"),
        ("not a", "¬a"),
        ("a -> b", "a → b"),
    ];
    
    for (expr1, expr2) in equivalent_expressions {
        let mut parser1 = Parser::from_str(expr1);
        let mut parser2 = Parser::from_str(expr2);
        
        let result1 = parser1.parse().expect(&format!("Failed to parse: {}", expr1));
        let result2 = parser2.parse().expect(&format!("Failed to parse: {}", expr2));
        
        // Both should produce the same AST structure
        assert_eq!(result1, result2, "Expressions should be equivalent: {} vs {}", expr1, expr2);
    }
}

#[test]
fn test_operator_precedence() {
    let precedence_tests = [
        // AND has higher precedence than OR
        ("a or b and c", Expr::Or(
            Box::new(Expr::Identifier("a".to_string())),
            Box::new(Expr::And(
                Box::new(Expr::Identifier("b".to_string())),
                Box::new(Expr::Identifier("c".to_string()))
            ))
        )),
        // NOT has highest precedence
        ("not a and b", Expr::And(
            Box::new(Expr::Not(Box::new(Expr::Identifier("a".to_string())))),
            Box::new(Expr::Identifier("b".to_string()))
        )),
        // Implication has lowest precedence
        ("a and b -> c or d", Expr::Implication(
            Box::new(Expr::And(
                Box::new(Expr::Identifier("a".to_string())),
                Box::new(Expr::Identifier("b".to_string()))
            )),
            Box::new(Expr::Or(
                Box::new(Expr::Identifier("c".to_string())),
                Box::new(Expr::Identifier("d".to_string()))
            ))
        )),
    ];
    
    for (input, expected) in precedence_tests {
        let mut parser = Parser::from_str(input);
        let result = parser.parse().expect(&format!("Failed to parse: {}", input));
        assert_eq!(result, expected, "Input: {}", input);
    }
}

#[test]
fn test_parentheses_override_precedence() {
    let parentheses_tests = [
        ("(a or b) and c", Expr::And(
            Box::new(Expr::Or(
                Box::new(Expr::Identifier("a".to_string())),
                Box::new(Expr::Identifier("b".to_string()))
            )),
            Box::new(Expr::Identifier("c".to_string()))
        )),
        ("a and (b or c)", Expr::And(
            Box::new(Expr::Identifier("a".to_string())),
            Box::new(Expr::Or(
                Box::new(Expr::Identifier("b".to_string())),
                Box::new(Expr::Identifier("c".to_string()))
            ))
        )),
    ];
    
    for (input, expected) in parentheses_tests {
        let mut parser = Parser::from_str(input);
        let result = parser.parse().expect(&format!("Failed to parse: {}", input));
        assert_eq!(result, expected, "Input: {}", input);
    }
}

#[test]
fn test_complex_nested_expressions() {
    let complex_cases = [
        "((a and b) or (c and d))",
        "(not a or b) and (c -> d)",
        "a -> b -> c", // Right associative
        "not not a",
        "(a xor b) and (c or not d)",
    ];
    
    for input in complex_cases {
        let mut parser = Parser::from_str(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse complex expression: {}", input);
        
        // Verify we can display the parsed expression
        let expr = result.unwrap();
        let display = expr.to_string();
        assert!(!display.is_empty(), "Display should not be empty for: {}", input);
    }
}