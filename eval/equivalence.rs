use crate::source::Expr;
use crate::eval::{Variables, EvaluationError};
use crate::eval::truth_table::evaluate_expression;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Result of an equivalence check between two expressions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceCheck {
    pub equivalent: bool,
    pub variables: Variables,
    pub differences: Vec<EquivalenceDifference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceDifference {
    pub assignment: HashMap<String, bool>,
    pub left_value: bool,
    pub right_value: bool,
}

/// Check if two boolean expressions are equivalent
pub fn check_equivalence(left: &Expr, right: &Expr) -> Result<EquivalenceCheck, EvaluationError> {
    let left_vars = Variables::from_expr(left)?;
    let right_vars = Variables::from_expr(right)?;
    let all_vars = left_vars.union(&right_vars);
    
    let mut differences = Vec::new();
    let num_vars = all_vars.len();
    
    if num_vars == 0 {
        // Handle expressions with no variables
        let left_result = evaluate_expression(left, &HashMap::new());
        let right_result = evaluate_expression(right, &HashMap::new());
        
        return Ok(EquivalenceCheck {
            equivalent: left_result == right_result,
            variables: all_vars,
            differences: if left_result != right_result {
                vec![EquivalenceDifference {
                    assignment: HashMap::new(),
                    left_value: left_result,
                    right_value: right_result,
                }]
            } else {
                vec![]
            },
        });
    }
    
    let num_combinations = 1 << num_vars;
    
    for i in 0..num_combinations {
        let mut assignments = HashMap::new();
        
        // Create assignment from bit pattern
        for (var_idx, var_name) in all_vars.iter().enumerate() {
            let bit_value = (i >> var_idx) & 1 == 1;
            assignments.insert(var_name.clone(), bit_value);
        }
        
        let left_result = evaluate_expression(left, &assignments);
        let right_result = evaluate_expression(right, &assignments);
        
        if left_result != right_result {
            differences.push(EquivalenceDifference {
                assignment: assignments,
                left_value: left_result,
                right_value: right_result,
            });
        }
    }
    
    Ok(EquivalenceCheck {
        equivalent: differences.is_empty(),
        variables: all_vars,
        differences,
    })
}