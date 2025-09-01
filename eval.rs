use crate::source::Expr;
use std::collections::{HashMap, HashSet, BTreeSet, BTreeMap};
use serde::{Serialize, Deserialize};

/// Maximum number of variables allowed
const MAX_VARIABLES: usize = 20;  // 2^20 = ~1M rows max
const MAX_VARIABLE_NAME_LENGTH: usize = 50;

/// Errors that can occur during evaluation
#[derive(Debug, Clone)]
pub enum EvaluationError {
    TooManyVariables { count: usize, max: usize },
    InvalidVariableName(String),
}

/// A sorted set of variable names for consistent ordering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variables {
    names: BTreeSet<String>,
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
    
    pub fn contains(&self, name: &String) -> bool {
        self.names.contains(name)
    }
}

/// Result of a truth table evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruthTable {
    pub variables: Variables,
    pub rows: Vec<TruthTableRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruthTableRow {
    pub assignments: HashMap<String, bool>,
    pub result: bool,
}

/// Output of an equivalence check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceCheck {
    pub equivalent: bool,
    pub differences: Vec<EquivalenceDifference>,
    pub variables: Variables,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceDifference {
    pub assignment: HashMap<String, bool>,
    pub left_value: bool,
    pub right_value: bool,
}

/// Output of expression reduction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reduction {
    pub original: Expr,
    pub reduced: Expr,
    pub simplified: bool,
}

/// Represents a minterm or implicant in the Quine-McCluskey algorithm
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Minterm {
    /// Binary representation where Some(true/false) is a literal, None is don't-care
    bits: Vec<Option<bool>>,
    /// Original minterms this implicant covers
    covered_minterms: BTreeSet<usize>,
}

impl Minterm {
    fn new(minterm_index: usize, num_vars: usize) -> Self {
        let mut bits = Vec::new();
        for i in 0..num_vars {
            let bit = (minterm_index >> (num_vars - 1 - i)) & 1 == 1;
            bits.push(Some(bit));
        }
        
        let mut covered_minterms = BTreeSet::new();
        covered_minterms.insert(minterm_index);
        
        Self { bits, covered_minterms }
    }
    
    /// Count the number of 1s in the minterm (for grouping)
    fn count_ones(&self) -> usize {
        self.bits.iter().filter(|&&bit| bit == Some(true)).count()
    }
    
    /// Try to combine two minterms if they differ by exactly one bit
    fn combine(&self, other: &Self) -> Option<Self> {
        if self.bits.len() != other.bits.len() {
            return None;
        }
        
        let mut diff_count = 0;
        let mut combined_bits = Vec::new();
        
        for i in 0..self.bits.len() {
            match (self.bits[i], other.bits[i]) {
                (Some(a), Some(b)) if a == b => combined_bits.push(Some(a)),
                (Some(_), Some(_)) => {
                    diff_count += 1;
                    if diff_count > 1 {
                        return None;
                    }
                    combined_bits.push(None); // Don't care
                }
                (None, None) => combined_bits.push(None),
                _ => return None, // Can't combine if one has don't-care and other has value
            }
        }
        
        if diff_count == 1 {
            let mut covered_minterms = self.covered_minterms.clone();
            covered_minterms.extend(&other.covered_minterms);
            Some(Minterm { bits: combined_bits, covered_minterms })
        } else {
            None
        }
    }
    
    /// Convert minterm back to an expression
    fn to_expression(&self, variables: &Variables) -> Option<Expr> {
        let var_vec = variables.to_vec();
        let mut terms = Vec::new();
        
        for (i, &bit) in self.bits.iter().enumerate() {
            match bit {
                Some(true) => terms.push(Expr::Identifier(var_vec[i].clone())),
                Some(false) => terms.push(Expr::Not(Box::new(Expr::Identifier(var_vec[i].clone())))),
                None => {} // Don't care, skip
            }
        }
        
        if terms.is_empty() {
            return None; // Should not happen in normal cases
        }
        
        // Combine terms with AND
        let mut result = terms[0].clone();
        for term in terms.into_iter().skip(1) {
            result = Expr::And(Box::new(result), Box::new(term));
        }
        
        Some(result)
    }
}

/// Quine-McCluskey algorithm implementation
pub struct QuineMcCluskey {
    variables: Variables,
    minterms: BTreeSet<usize>,
}

impl QuineMcCluskey {
    /// Create a new Quine-McCluskey instance from an expression
    pub fn from_expression(expr: &Expr) -> Result<Self, EvaluationError> {
        let variables = Variables::from_expr(expr)?;
        let num_vars = variables.len();
        let mut minterms = BTreeSet::new();
        
        // Generate all possible truth assignments and check which ones make the expression true
        for i in 0..(1 << num_vars) {
            let mut assignment = HashMap::new();
            
            for (j, var) in variables.iter().enumerate() {
                let value = (i >> (num_vars - 1 - j)) & 1 == 1;
                assignment.insert(var.clone(), value);
            }
            
            if evaluate_expression(expr, &assignment) {
                minterms.insert(i);
            }
        }
        
        Ok(Self { variables, minterms })
    }
    
    /// Run the Quine-McCluskey algorithm to find minimal sum-of-products
    pub fn minimize(&self) -> Option<Expr> {
        if self.minterms.is_empty() {
            // Expression is always false
            return Some(Expr::And(
                Box::new(Expr::Identifier("false".to_string())),
                Box::new(Expr::Not(Box::new(Expr::Identifier("false".to_string()))))
            ));
        }
        
        let num_vars = self.variables.len();
        if num_vars == 0 {
            return None;
        }
        
        // Step 1: Generate initial minterms
        let current_implicants: Vec<Minterm> = self.minterms
            .iter()
            .map(|&idx| Minterm::new(idx, num_vars))
            .collect();
        
        // Step 2: Find all prime implicants
        let prime_implicants = self.find_prime_implicants(current_implicants);
        
        // Step 3: Find essential prime implicants and minimal cover
        let minimal_cover = self.find_minimal_cover(&prime_implicants);
        
        // Step 4: Convert back to expression
        self.implicants_to_expression(&minimal_cover)
    }
    
    /// Find all prime implicants using iterative combining
    fn find_prime_implicants(&self, mut current_implicants: Vec<Minterm>) -> Vec<Minterm> {
        let mut prime_implicants = Vec::new();
        
        while !current_implicants.is_empty() {
            let mut next_implicants = Vec::new();
            let mut used = vec![false; current_implicants.len()];
            
            // Group by number of 1s
            let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
            for (i, implicant) in current_implicants.iter().enumerate() {
                let ones_count = implicant.count_ones();
                groups.entry(ones_count).or_insert_with(Vec::new).push(i);
            }
            
            // Try to combine adjacent groups
            for (&ones_count, indices) in &groups {
                if let Some(next_indices) = groups.get(&(ones_count + 1)) {
                    for &i in indices {
                        for &j in next_indices {
                            if let Some(combined) = current_implicants[i].combine(&current_implicants[j]) {
                                next_implicants.push(combined);
                                used[i] = true;
                                used[j] = true;
                            }
                        }
                    }
                }
            }
            
            // Add unused implicants as prime implicants
            for (i, &is_used) in used.iter().enumerate() {
                if !is_used {
                    prime_implicants.push(current_implicants[i].clone());
                }
            }
            
            // Remove duplicates from next_implicants
            next_implicants.sort_by(|a, b| a.bits.cmp(&b.bits));
            next_implicants.dedup();
            
            current_implicants = next_implicants;
        }
        
        prime_implicants
    }
    
    /// Find minimal cover using essential prime implicants and heuristics
    fn find_minimal_cover(&self, prime_implicants: &[Minterm]) -> Vec<Minterm> {
        if prime_implicants.is_empty() {
            return Vec::new();
        }
        
        let mut uncovered_minterms: BTreeSet<usize> = self.minterms.clone();
        let mut selected_implicants = Vec::new();
        let mut available_implicants = prime_implicants.to_vec();
        
        // First, select essential prime implicants
        loop {
            let mut essential_found = false;
            let mut to_remove = Vec::new();
            let mut covered_by_essential = BTreeSet::new();
            
            // Find essential prime implicants for uncovered minterms
            let uncovered_vec: Vec<_> = uncovered_minterms.iter().collect();
            for &minterm in uncovered_vec {
                let covering_implicants: Vec<_> = available_implicants
                    .iter()
                    .enumerate()
                    .filter(|(_, impl_)| impl_.covered_minterms.contains(&minterm))
                    .collect();
                
                if covering_implicants.len() == 1 {
                    // Essential prime implicant found
                    let (idx, implicant) = covering_implicants[0];
                    
                    // Check if we already selected this implicant in this iteration
                    if !to_remove.contains(&idx) {
                        selected_implicants.push(implicant.clone());
                        
                        // Mark covered minterms for removal
                        covered_by_essential.extend(&implicant.covered_minterms);
                        
                        to_remove.push(idx);
                        essential_found = true;
                    }
                }
            }
            
            // Remove covered minterms
            for &covered in &covered_by_essential {
                uncovered_minterms.remove(&covered);
            }
            
            // Remove used implicants
            for &idx in to_remove.iter().rev() {
                available_implicants.remove(idx);
            }
            
            if !essential_found {
                break;
            }
        }
        
        // If all minterms are covered, we're done
        if uncovered_minterms.is_empty() {
            return selected_implicants;
        }
        
        // Use greedy heuristic for remaining minterms
        while !uncovered_minterms.is_empty() && !available_implicants.is_empty() {
            // Find implicant that covers the most uncovered minterms
            let best_implicant = available_implicants
                .iter()
                .enumerate()
                .max_by_key(|(_, impl_)| {
                    impl_.covered_minterms.intersection(&uncovered_minterms).count()
                });
            
            if let Some((idx, implicant)) = best_implicant {
                selected_implicants.push(implicant.clone());
                
                // Remove covered minterms
                for &covered in &implicant.covered_minterms {
                    uncovered_minterms.remove(&covered);
                }
                
                available_implicants.remove(idx);
            } else {
                break;
            }
        }
        
        selected_implicants
    }
    
    /// Convert selected implicants back to a boolean expression
    fn implicants_to_expression(&self, implicants: &[Minterm]) -> Option<Expr> {
        if implicants.is_empty() {
            return None;
        }
        
        let terms: Vec<_> = implicants
            .iter()
            .filter_map(|impl_| impl_.to_expression(&self.variables))
            .collect();
        
        if terms.is_empty() {
            return None;
        }
        
        if terms.len() == 1 {
            return Some(terms[0].clone());
        }
        
        // Combine terms with OR
        let mut result = terms[0].clone();
        for term in terms.into_iter().skip(1) {
            result = Expr::Or(Box::new(result), Box::new(term));
        }
        
        Some(result)
    }
}

/// Main evaluator for boolean expressions
pub struct Evaluator;

impl Evaluator {
    /// Generate a complete truth table for an expression
    pub fn generate_truth_table(expr: &Expr) -> Result<TruthTable, EvaluationError> {
        let variables = Variables::from_expr(expr)?;
        let num_vars = variables.len();
        let mut rows = Vec::new();
        
        // Generate all possible truth assignments
        for i in 0..(1 << num_vars) {
            let mut assignments = HashMap::new();
            
            // Create truth assignment for this combination
            for (j, var) in variables.iter().enumerate() {
                let value = (i >> (num_vars - 1 - j)) & 1 == 1;
                assignments.insert(var.clone(), value);
            }
            
            // Evaluate expression with this assignment
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
    
    /// Check if two expressions are logically equivalent
    pub fn check_equivalence(left: &Expr, right: &Expr) -> Result<EquivalenceCheck, EvaluationError> {
        let left_vars = Variables::from_expr(left)?;
        let right_vars = Variables::from_expr(right)?;
        let variables = left_vars.union(&right_vars);
        
        let num_vars = variables.len();
        let mut equivalent = true;
        let mut differences = Vec::new();
        
        // Check all possible truth assignments
        for i in 0..(1 << num_vars) {
            let mut assignment = HashMap::new();
            
            for (j, var) in variables.iter().enumerate() {
                let value = (i >> (num_vars - 1 - j)) & 1 == 1;
                assignment.insert(var.clone(), value);
            }
            
            let left_result = evaluate_expression(left, &assignment);
            let right_result = evaluate_expression(right, &assignment);
            
            if left_result != right_result {
                equivalent = false;
                differences.push(EquivalenceDifference {
                    assignment,
                    left_value: left_result,
                    right_value: right_result,
                });
            }
        }
        
        Ok(EquivalenceCheck {
            equivalent,
            differences,
            variables,
        })
    }
    
    /// Attempt to reduce/simplify a boolean expression using Quine-McCluskey
    pub fn reduce_expression(expr: &Expr) -> Result<Reduction, EvaluationError> {
        // Handle special cases first
        if Self::is_tautology(expr) {
            // Expression is always true
            let true_expr = Expr::Or(
                Box::new(Expr::Identifier("true".to_string())),
                Box::new(Expr::Not(Box::new(Expr::Identifier("true".to_string()))))
            );
            return Ok(Reduction {
                original: expr.clone(),
                reduced: true_expr,
                simplified: true,
            });
        }
        
        if Self::is_contradiction(expr) {
            // Expression is always false
            let false_expr = Expr::And(
                Box::new(Expr::Identifier("false".to_string())),
                Box::new(Expr::Not(Box::new(Expr::Identifier("false".to_string()))))
            );
            return Ok(Reduction {
                original: expr.clone(),
                reduced: false_expr,
                simplified: true,
            });
        }
        
        // Use Quine-McCluskey for general reduction
        match QuineMcCluskey::from_expression(expr) {
            Ok(qm) => {
                if let Some(reduced_expr) = qm.minimize() {
                    // Check if the reduction actually simplified the expression
                    let simplified = !Self::expressions_equivalent_structure(expr, &reduced_expr);
                    
                    Ok(Reduction {
                        original: expr.clone(),
                        reduced: reduced_expr,
                        simplified,
                    })
                } else {
                    // Could not minimize (e.g., no variables)
                    Ok(Reduction {
                        original: expr.clone(),
                        reduced: expr.clone(),
                        simplified: false,
                    })
                }
            }
            Err(e) => Err(e),
        }
    }
    
    /// Check if an expression is a tautology (always true)
    fn is_tautology(expr: &Expr) -> bool {
        match Variables::from_expr(expr) {
            Ok(variables) => {
                let num_vars = variables.len();
                if num_vars == 0 {
                    return false; // No variables, evaluate directly
                }
                
                // Check all possible truth assignments
                for i in 0..(1 << num_vars) {
                    let mut assignment = HashMap::new();
                    
                    for (j, var) in variables.iter().enumerate() {
                        let value = (i >> (num_vars - 1 - j)) & 1 == 1;
                        assignment.insert(var.clone(), value);
                    }
                    
                    if !evaluate_expression(expr, &assignment) {
                        return false; // Found an assignment that makes it false
                    }
                }
                
                true // All assignments make it true
            }
            Err(_) => false, // Error in expression, not a tautology
        }
    }
    
    /// Check if an expression is a contradiction (always false)
    fn is_contradiction(expr: &Expr) -> bool {
        match Variables::from_expr(expr) {
            Ok(variables) => {
                let num_vars = variables.len();
                if num_vars == 0 {
                    return false; // No variables, evaluate directly
                }
                
                // Check all possible truth assignments
                for i in 0..(1 << num_vars) {
                    let mut assignment = HashMap::new();
                    
                    for (j, var) in variables.iter().enumerate() {
                        let value = (i >> (num_vars - 1 - j)) & 1 == 1;
                        assignment.insert(var.clone(), value);
                    }
                    
                    if evaluate_expression(expr, &assignment) {
                        return false; // Found an assignment that makes it true
                    }
                }
                
                true // All assignments make it false
            }
            Err(_) => false, // Error in expression, not a contradiction
        }
    }
    
    /// Check if two expressions have equivalent structure (for simplification detection)
    fn expressions_equivalent_structure(a: &Expr, b: &Expr) -> bool {
        match (a, b) {
            (Expr::Identifier(name1), Expr::Identifier(name2)) => name1 == name2,
            (Expr::Not(e1), Expr::Not(e2)) => Self::expressions_equivalent_structure(e1, e2),
            (Expr::And(l1, r1), Expr::And(l2, r2)) 
            | (Expr::Or(l1, r1), Expr::Or(l2, r2))
            | (Expr::Xor(l1, r1), Expr::Xor(l2, r2))
            | (Expr::Implication(l1, r1), Expr::Implication(l2, r2)) => {
                (Self::expressions_equivalent_structure(l1, l2) && Self::expressions_equivalent_structure(r1, r2)) ||
                (Self::expressions_equivalent_structure(l1, r2) && Self::expressions_equivalent_structure(r1, l2))
            }
            _ => false,
        }
    }
    
    /// Evaluate a single expression with given variable assignments
    pub fn evaluate_with_assignment(expr: &Expr, assignment: &HashMap<String, bool>) -> bool {
        evaluate_expression(expr, assignment)
    }
    
    /// Get all variables used in an expression
    pub fn collect_expression_variables(expr: &Expr) -> HashSet<String> {
        collect_variables(expr)
    }
}

/// Collect all variable names used in an expression
fn collect_variables(expr: &Expr) -> HashSet<String> {
    let mut variables = HashSet::new();
    collect_variables_recursive(expr, &mut variables);
    variables
}

/// Recursively collect variables from an expression tree
fn collect_variables_recursive(expr: &Expr, variables: &mut HashSet<String>) {
    match expr {
        Expr::Identifier(name) => {
            variables.insert(name.clone());
        }
        Expr::Not(e) => collect_variables_recursive(e, variables),
        Expr::And(left, right) 
        | Expr::Or(left, right) 
        | Expr::Xor(left, right) 
        | Expr::Implication(left, right) => {
            collect_variables_recursive(left, variables);
            collect_variables_recursive(right, variables);
        }
    }
}

/// Evaluate a boolean expression with given variable assignments
fn evaluate_expression(expr: &Expr, assignment: &HashMap<String, bool>) -> bool {
    match expr {
        Expr::Identifier(name) => {
            assignment.get(name).copied().unwrap_or(false)
        }
        Expr::Not(e) => {
            !evaluate_expression(e, assignment)
        }
        Expr::And(left, right) => {
            evaluate_expression(left, assignment) && evaluate_expression(right, assignment)
        }
        Expr::Or(left, right) => {
            evaluate_expression(left, assignment) || evaluate_expression(right, assignment)
        }
        Expr::Xor(left, right) => {
            evaluate_expression(left, assignment) ^ evaluate_expression(right, assignment)
        }
        Expr::Implication(left, right) => {
            let left_val = evaluate_expression(left, assignment);
            let right_val = evaluate_expression(right, assignment);
            // A -> B is equivalent to !A || B
            !left_val || right_val
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::Parser;
    
    #[test]
    fn test_truth_table_generation() {
        let mut parser = Parser::from_str("a and b");
        let expr = parser.parse().unwrap();
        
        let table = Evaluator::generate_truth_table(&expr).unwrap();
        
        assert_eq!(table.variables.to_vec(), vec!["a", "b"]);
        assert_eq!(table.rows.len(), 4);
        
        // Check specific rows
        let row_ff = table.rows.iter().find(|r| !r.assignments["a"] && !r.assignments["b"]).unwrap();
        assert_eq!(row_ff.result, false);
        
        let row_tt = table.rows.iter().find(|r| r.assignments["a"] && r.assignments["b"]).unwrap();
        assert_eq!(row_tt.result, true);
    }
    
    #[test]
    fn test_equivalence_check_true() {
        let mut parser1 = Parser::from_str("a and b");
        let mut parser2 = Parser::from_str("b and a");
        let expr1 = parser1.parse().unwrap();
        let expr2 = parser2.parse().unwrap();
        
        let check = Evaluator::check_equivalence(&expr1, &expr2).unwrap();
        
        assert!(check.equivalent);
        assert!(check.differences.is_empty());
    }
    
    #[test]
    fn test_equivalence_check_false() {
        let mut parser1 = Parser::from_str("a or b");
        let mut parser2 = Parser::from_str("a and b");
        let expr1 = parser1.parse().unwrap();
        let expr2 = parser2.parse().unwrap();
        
        let check = Evaluator::check_equivalence(&expr1, &expr2).unwrap();
        
        assert!(!check.equivalent);
        assert!(!check.differences.is_empty());
    }
    
    #[test]
    fn test_variable_collection() {
        let mut parser = Parser::from_str("(a and b) or (c -> d)");
        let expr = parser.parse().unwrap();
        
        let vars = Evaluator::collect_expression_variables(&expr);
        
        assert_eq!(vars.len(), 4);
        assert!(vars.contains("a"));
        assert!(vars.contains("b"));
        assert!(vars.contains("c"));
        assert!(vars.contains("d"));
    }
    
    #[test]
    fn test_evaluation_with_assignment() {
        let mut parser = Parser::from_str("a and (b or not c)");
        let expr = parser.parse().unwrap();
        
        let mut assignment = HashMap::new();
        assignment.insert("a".to_string(), true);
        assignment.insert("b".to_string(), false);
        assignment.insert("c".to_string(), true);
        
        let result = Evaluator::evaluate_with_assignment(&expr, &assignment);
        
        // a=T and (b=F or not c=T) = T and (F or F) = T and F = F
        assert_eq!(result, false);
    }
    
    #[test]
    fn test_invalid_variable_name_validation() {
        // Test empty variable name
        let result = Variables::from_expr(&Expr::Identifier("".to_string()));
        assert!(matches!(result, Err(EvaluationError::InvalidVariableName(_))));
        
        // Test variable name too long
        let long_name = "a".repeat(51);
        let result = Variables::from_expr(&Expr::Identifier(long_name));
        assert!(matches!(result, Err(EvaluationError::InvalidVariableName(_))));
        
        // Test invalid characters
        let result = Variables::from_expr(&Expr::Identifier("a-b".to_string()));
        assert!(matches!(result, Err(EvaluationError::InvalidVariableName(_))));
        
        // Test valid variable names work
        let result = Variables::from_expr(&Expr::Identifier("valid_var123".to_string()));
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_too_many_variables_validation() {
        // Create an expression with more than MAX_VARIABLES variables
        // This would be a very deep expression tree, but let's test the limit
        let mut expr = Expr::Identifier("a".to_string());
        
        // Create an OR chain with many variables: a or b or c or ... 
        for i in 1..25 { // More than MAX_VARIABLES (20)
            let var_name = format!("var{}", i);
            expr = Expr::Or(
                Box::new(expr),
                Box::new(Expr::Identifier(var_name))
            );
        }
        
        let result = Variables::from_expr(&expr);
        assert!(matches!(result, Err(EvaluationError::TooManyVariables { .. })));
    }
    
    #[test]
    fn test_quine_mccluskey_absorption() {
        // Test absorption law: a ∨ (a ∧ b) = a
        let mut parser = Parser::from_str("a or (a and b)");
        let expr = parser.parse().unwrap();
        
        let result = Evaluator::reduce_expression(&expr).unwrap();
        
        assert!(result.simplified);
        assert_eq!(result.reduced, Expr::Identifier("a".to_string()));
    }
    
    #[test]
    fn test_quine_mccluskey_consensus() {
        // Test consensus: (a ∧ b) ∨ (¬a ∧ b) = b
        let mut parser = Parser::from_str("(a and b) or (not a and b)");
        let expr = parser.parse().unwrap();
        
        let result = Evaluator::reduce_expression(&expr).unwrap();
        
        assert!(result.simplified);
        assert_eq!(result.reduced, Expr::Identifier("b".to_string()));
    }
    
    #[test]
    fn test_quine_mccluskey_tautology() {
        // Test tautology: a ∨ ¬a = true
        let mut parser = Parser::from_str("a or not a");
        let expr = parser.parse().unwrap();
        
        let result = Evaluator::reduce_expression(&expr).unwrap();
        
        assert!(result.simplified);
        // Should be reduced to a tautology form
        let expected_true = Expr::Or(
            Box::new(Expr::Identifier("true".to_string())),
            Box::new(Expr::Not(Box::new(Expr::Identifier("true".to_string()))))
        );
        assert_eq!(result.reduced, expected_true);
    }
    
    #[test]
    fn test_quine_mccluskey_contradiction() {
        // Test contradiction: a ∧ ¬a = false
        let mut parser = Parser::from_str("a and not a");
        let expr = parser.parse().unwrap();
        
        let result = Evaluator::reduce_expression(&expr).unwrap();
        
        assert!(result.simplified);
        // Should be reduced to a contradiction form
        let expected_false = Expr::And(
            Box::new(Expr::Identifier("false".to_string())),
            Box::new(Expr::Not(Box::new(Expr::Identifier("false".to_string()))))
        );
        assert_eq!(result.reduced, expected_false);
    }
    
    #[test]
    fn test_quine_mccluskey_complex_reduction() {
        // Test complex case: (a∧b∧c) ∨ (a∧b∧¬c) ∨ (a∧¬b∧c) should reduce
        let mut parser = Parser::from_str("(a and b and c) or (a and b and not c) or (a and not b and c)");
        let expr = parser.parse().unwrap();
        
        let result = Evaluator::reduce_expression(&expr).unwrap();
        
        assert!(result.simplified);
        // The reduction should have fewer terms or literals than original
        // Exact form may vary but should be equivalent and simpler
    }
    
    #[test]
    fn test_quine_mccluskey_already_minimal() {
        // Test case that's already minimal
        let mut parser = Parser::from_str("a and b");
        let expr = parser.parse().unwrap();
        
        let result = Evaluator::reduce_expression(&expr).unwrap();
        
        assert!(!result.simplified); // Should not be simplified
        // Should be structurally the same (though order might differ)
    }
}
