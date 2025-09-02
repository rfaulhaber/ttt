use crate::source::Expr;
use crate::eval::{Variables, EvaluationError};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Result of a truth table evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruthTable {
    pub variables: Variables,
    pub rows: Vec<TruthTableRow>,
}

impl TruthTable {
    /// Create a new empty truth table
    pub fn new(variables: Variables) -> Self {
        Self {
            variables,
            rows: Vec::new(),
        }
    }
    
    /// Get a builder for constructing truth tables
    pub fn builder() -> TruthTableBuilder {
        TruthTableBuilder::new()
    }
}

/// Builder for constructing truth tables incrementally
pub struct TruthTableBuilder {
    variables: Option<Variables>,
    rows: Vec<TruthTableRow>,
}

impl TruthTableBuilder {
    /// Create a new truth table builder
    pub fn new() -> Self {
        Self {
            variables: None,
            rows: Vec::new(),
        }
    }
    
    /// Set the variables for the truth table
    pub fn variables(mut self, variables: Variables) -> Self {
        self.variables = Some(variables);
        self
    }
    
    /// Add a row to the truth table
    pub fn add_row(mut self, row: TruthTableRow) -> Self {
        self.rows.push(row);
        self
    }
    
    /// Add multiple rows to the truth table
    pub fn add_rows(mut self, rows: Vec<TruthTableRow>) -> Self {
        self.rows.extend(rows);
        self
    }
    
    /// Build the truth table
    pub fn build(self) -> Result<TruthTable, EvaluationError> {
        let variables = self.variables.ok_or_else(|| {
            EvaluationError::ExpressionTooComplex {
                reason: "Variables must be set when building a truth table".to_string(),
            }
        })?;
        
        Ok(TruthTable {
            variables,
            rows: self.rows,
        })
    }
}

impl Default for TruthTableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruthTableRow {
    pub assignments: HashMap<String, bool>,
    pub result: bool,
}

/// Generate a truth table from a boolean expression
pub fn generate_truth_table(expr: &Expr) -> Result<TruthTable, EvaluationError> {
    let variables = Variables::from_expr(expr)?;
    let num_vars = variables.len();
    
    if num_vars == 0 {
        // Handle expressions with no variables (like constants)
        return Ok(TruthTable {
            variables,
            rows: vec![TruthTableRow {
                assignments: HashMap::new(),
                result: evaluate_expression(expr, &HashMap::new()),
            }],
        });
    }
    
    let mut rows = Vec::new();
    let num_combinations = 1 << num_vars; // 2^num_vars
    
    for i in 0..num_combinations {
        let mut assignments = HashMap::new();
        
        // Create assignment from bit pattern
        for (var_idx, var_name) in variables.iter().enumerate() {
            let bit_value = (i >> var_idx) & 1 == 1;
            assignments.insert(var_name.clone(), bit_value);
        }
        
        let result = evaluate_expression(expr, &assignments);
        
        rows.push(TruthTableRow {
            assignments,
            result,
        });
    }
    
    Ok(TruthTable {
        variables,
        rows,
    })
}

/// Evaluate a boolean expression with given variable assignments
pub fn evaluate_expression(expr: &Expr, assignments: &HashMap<String, bool>) -> bool {
    match expr {
        Expr::Identifier(name) => {
            assignments.get(name).copied().unwrap_or(false)
        }
        Expr::Not(inner) => {
            !evaluate_expression(inner, assignments)
        }
        Expr::And(left, right) => {
            evaluate_expression(left, assignments) && evaluate_expression(right, assignments)
        }
        Expr::Or(left, right) => {
            evaluate_expression(left, assignments) || evaluate_expression(right, assignments)
        }
        Expr::Xor(left, right) => {
            evaluate_expression(left, assignments) ^ evaluate_expression(right, assignments)
        }
        Expr::Implication(left, right) => {
            !evaluate_expression(left, assignments) || evaluate_expression(right, assignments)
        }
    }
}