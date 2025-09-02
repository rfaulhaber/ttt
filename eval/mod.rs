pub mod truth_table;
pub mod equivalence;
pub mod reduction;

use crate::source::Expr;
use std::collections::BTreeSet;
use std::fmt;
use serde::{Serialize, Deserialize};

use crate::config::{MAX_VARIABLES, MAX_VARIABLE_NAME_LENGTH};

/// Errors that can occur during evaluation
#[derive(Debug, Clone)]
pub enum EvaluationError {
    TooManyVariables { count: usize, max: usize },
    InvalidVariableName(String),
    ExpressionTooComplex { reason: String },
    ReductionTimeout { max_iterations: usize },
    UnsupportedOperation { operation: String },
    EmptyExpression,
    InvalidTruthAssignment { variable: String, context: String },
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvaluationError::TooManyVariables { count, max } => {
                write!(f, "Expression has too many variables ({} > {}). Consider simplifying the expression.", count, max)
            }
            EvaluationError::InvalidVariableName(name) => {
                write!(f, "Invalid variable name '{}'. Variable names must be non-empty, alphanumeric (with underscores), and at most {} characters long.", name, MAX_VARIABLE_NAME_LENGTH)
            }
            EvaluationError::ExpressionTooComplex { reason } => {
                write!(f, "Expression is too complex to process: {}", reason)
            }
            EvaluationError::ReductionTimeout { max_iterations } => {
                write!(f, "Expression reduction timed out after {} iterations. The expression may be too complex to simplify.", max_iterations)
            }
            EvaluationError::UnsupportedOperation { operation } => {
                write!(f, "Unsupported operation: {}", operation)
            }
            EvaluationError::EmptyExpression => {
                write!(f, "Cannot evaluate an empty expression")
            }
            EvaluationError::InvalidTruthAssignment { variable, context } => {
                write!(f, "Invalid truth assignment for variable '{}' in context: {}", variable, context)
            }
        }
    }
}

impl std::error::Error for EvaluationError {}

/// A sorted set of variable names for consistent ordering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variables {
    names: BTreeSet<String>,
}

impl Default for Variables {
    fn default() -> Self {
        Self::new()
    }
}

impl Variables {
    pub fn new() -> Self {
        Self { names: BTreeSet::new() }
    }
    
    pub fn from_expr(expr: &Expr) -> Result<Self, EvaluationError> {
        let mut vars = Self::new();
        vars.collect_from_expr(expr)?;
        Ok(vars)
    }
    
    fn collect_from_expr(&mut self, expr: &Expr) -> Result<(), EvaluationError> {
        match expr {
            Expr::Identifier(name) => {
                // Validate variable name
                if name.is_empty() || name.len() > MAX_VARIABLE_NAME_LENGTH || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Err(EvaluationError::InvalidVariableName(name.clone()));
                }
                
                self.names.insert(name.clone());
                
                // Check variable count limit
                if self.names.len() > MAX_VARIABLES {
                    return Err(EvaluationError::TooManyVariables {
                        count: self.names.len(),
                        max: MAX_VARIABLES,
                    });
                }
                
                Ok(())
            }
            Expr::Not(e) => self.collect_from_expr(e),
            Expr::And(left, right) 
            | Expr::Or(left, right) 
            | Expr::Xor(left, right) 
            | Expr::Implication(left, right) => {
                self.collect_from_expr(left)?;
                self.collect_from_expr(right)?;
                Ok(())
            }
        }
    }
    
    pub fn len(&self) -> usize {
        self.names.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.names.iter()
    }
    
    pub fn to_vec(&self) -> Vec<String> {
        self.names.iter().cloned().collect()
    }
    
    pub fn union(&self, other: &Variables) -> Variables {
        Variables {
            names: self.names.union(&other.names).cloned().collect()
        }
    }
    
    pub fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }
}

/// Main evaluator interface
pub struct Evaluator;

impl Evaluator {
    /// Generate a truth table from a boolean expression
    pub fn generate_truth_table(expr: &Expr) -> Result<truth_table::TruthTable, EvaluationError> {
        truth_table::generate_truth_table(expr)
    }

    /// Check if two boolean expressions are equivalent
    pub fn check_equivalence(left: &Expr, right: &Expr) -> Result<equivalence::EquivalenceCheck, EvaluationError> {
        equivalence::check_equivalence(left, right)
    }

    /// Reduce/simplify a boolean expression using Quine-McCluskey algorithm
    pub fn reduce_expression(expr: &Expr) -> Result<reduction::Reduction, EvaluationError> {
        reduction::reduce_expression(expr)
    }
    
    /// Evaluate an expression with a given variable assignment (for testing)
    pub fn evaluate_with_assignment(expr: &Expr, assignment: &std::collections::HashMap<String, bool>) -> bool {
        truth_table::evaluate_expression(expr, assignment)
    }
    
    /// Collect all variables from an expression (for testing)
    pub fn collect_expression_variables(expr: &Expr) -> Result<Variables, EvaluationError> {
        Variables::from_expr(expr)
    }
}

// Re-export public types for backward compatibility
pub use truth_table::{TruthTable, TruthTableRow};
pub use equivalence::{EquivalenceCheck, EquivalenceDifference};
pub use reduction::Reduction;