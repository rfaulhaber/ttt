use crate::source::Expr;
use crate::eval::{Variables, EvaluationError};
use crate::eval::truth_table::evaluate_expression;
use std::collections::{HashMap, BTreeSet, BTreeMap};
use serde::{Serialize, Deserialize};

/// Result of expression reduction
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
                groups.entry(ones_count).or_default().push(i);
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
            
            // Remove used implicants (sort indices in descending order to avoid index invalidation)
            to_remove.sort_by(|a, b| b.cmp(a));
            for &idx in &to_remove {
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

/// Reduce/simplify a boolean expression using Quine-McCluskey algorithm
pub fn reduce_expression(expr: &Expr) -> Result<Reduction, EvaluationError> {
    // Handle special cases first
    if is_tautology(expr) {
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
    
    if is_contradiction(expr) {
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
                let simplified = !expressions_equivalent_structure(expr, &reduced_expr);
                
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

/// Compare two expressions for structural equivalence (not logical equivalence)
fn expressions_equivalent_structure(left: &Expr, right: &Expr) -> bool {
    match (left, right) {
        (Expr::Identifier(a), Expr::Identifier(b)) => a == b,
        (Expr::Not(a), Expr::Not(b)) => expressions_equivalent_structure(a, b),
        (Expr::And(a1, a2), Expr::And(b1, b2)) |
        (Expr::Or(a1, a2), Expr::Or(b1, b2)) |
        (Expr::Xor(a1, a2), Expr::Xor(b1, b2)) |
        (Expr::Implication(a1, a2), Expr::Implication(b1, b2)) => {
            expressions_equivalent_structure(a1, b1) && expressions_equivalent_structure(a2, b2)
        }
        _ => false,
    }
}