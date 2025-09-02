use ttt::source::{Parser, Expr};

/// Tests based on examples from the README.md
#[test]
fn test_readme_table_example() {
    // From README: ttt table a or not b
    let mut parser = Parser::new("a or not b");
    let result = parser.parse().expect("Should parse README table example");
    
    let expected = Expr::Or(
        Box::new(Expr::Identifier("a".to_string())),
        Box::new(Expr::Not(Box::new(Expr::Identifier("b".to_string()))))
    );
    
    assert_eq!(result, expected);
    assert_eq!(result.to_string(), "(a ∨ ¬b)");
}

#[test] 
fn test_readme_equivalence_examples() {
    // From README: ttt eq --left a or not b --right not a or b
    let left_expr = "a or not b";
    let right_expr = "not a or b";
    
    let mut parser1 = Parser::new(left_expr);
    let left_result = parser1.parse().expect("Should parse left expression");
    
    let mut parser2 = Parser::new(right_expr);  
    let right_result = parser2.parse().expect("Should parse right expression");
    
    // Verify both parse successfully (equivalence logic would be separate)
    let expected_left = Expr::Or(
        Box::new(Expr::Identifier("a".to_string())),
        Box::new(Expr::Not(Box::new(Expr::Identifier("b".to_string()))))
    );
    
    let expected_right = Expr::Or(
        Box::new(Expr::Not(Box::new(Expr::Identifier("a".to_string())))),
        Box::new(Expr::Identifier("b".to_string()))
    );
    
    assert_eq!(left_result, expected_left);
    assert_eq!(right_result, expected_right);
}

#[test]
fn test_readme_grammar_examples() {
    // Test all operators mentioned in the README grammar table
    let operator_examples = [
        ("a && b", "and operator with &&"),
        ("a ∧ b", "and operator with ∧"), 
        ("a and b", "and operator with word"),
        ("a || b", "or operator with ||"),
        ("a ∨ b", "or operator with ∨"),
        ("a or b", "or operator with word"),
        ("!a", "not operator with !"),
        ("¬a", "not operator with ¬"),
        ("not a", "not operator with word"),
        ("a -> b", "implication with ->"),
        ("a → b", "implication with →"),
        ("a xor b", "xor operator with word"),
        ("a ⊻ b", "xor operator with ⊻"),
        ("a ⊕ b", "xor operator with ⊕"),
    ];
    
    for (expr, description) in operator_examples {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse {}: {}", description, expr);
    }
}

#[test]
fn test_readme_grammar_structure() {
    // Test the grammar structure: (unary operator)? identifier ((binary operator) expr)?
    
    // Just identifier
    let mut parser1 = Parser::new("x");
    assert!(parser1.parse().is_ok());
    
    // Unary operator + identifier  
    let mut parser2 = Parser::new("not x");
    assert!(parser2.parse().is_ok());
    
    // Identifier + binary operator + expression
    let mut parser3 = Parser::new("x and y");
    assert!(parser3.parse().is_ok());
    
    // Unary operator + identifier + binary operator + expression
    let mut parser4 = Parser::new("not x and y");
    assert!(parser4.parse().is_ok());
    
    // Complex nested case
    let mut parser5 = Parser::new("not x and y or z");
    assert!(parser5.parse().is_ok());
}

#[test]
fn test_identifier_constraints() {
    // From README: "alphabetic set of characters that is not a keyword"
    
    // Valid identifiers
    let valid_identifiers = ["a", "variable", "var_name", "P", "Q", "proposition"];
    for id in valid_identifiers {
        let mut parser = Parser::new(id);
        let result = parser.parse().expect(&format!("Should parse identifier: {}", id));
        assert_eq!(result, Expr::Identifier(id.to_string()));
    }
    
    // Keywords should be parsed as operators, not identifiers
    let keyword_expressions = [
        ("a and b", "and"),
        ("a or b", "or"), 
        ("not a", "not"),
        ("a xor b", "xor"),
    ];
    for (expr, keyword) in keyword_expressions {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok(), "Keywords should work as operators: {} in '{}'", keyword, expr);
        
        // The expression should not contain the keyword as an identifier
        if let Ok(parsed_expr) = result {
            match parsed_expr {
                Expr::Identifier(name) if name == keyword => {
                    panic!("Keyword '{}' was parsed as identifier instead of operator", keyword);
                }
                _ => {} // Good, the keyword is being used as an operator
            }
        }
    }
}