use crate::eval::{TruthTable, EquivalenceCheck, Reduction, EquivalenceDifference};
use crate::config::MAX_DIFFERENCES_TO_SHOW;
use serde_json;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    /// Human-readable table format (default)
    Table,
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// Nuon format
    Nuon,
}

pub trait Formatter {
    fn format_truth_table(&self, table: &TruthTable) -> String;
    fn format_equivalence_result(&self, check: &EquivalenceCheck, left_str: &str, right_str: &str) -> String;
    fn format_reduction_result(&self, reduction: &Reduction) -> String;
}

pub struct TableFormatter;
pub struct JsonFormatter;
pub struct CsvFormatter;
pub struct NuonFormatter;

impl Formatter for TableFormatter {
    fn format_truth_table(&self, table: &TruthTable) -> String {
        let mut output = String::new();
        
        // Header
        for var in table.variables.iter() {
            output.push_str(&format!("{:>4}", var));
        }
        output.push_str(&format!("{:>8}\n", "Result"));
        
        // Separator
        for _ in 0..table.variables.len() {
            output.push_str("----");
        }
        output.push_str("--------\n");
        
        // Rows
        for row in &table.rows {
            for var in table.variables.iter() {
                let value = row.assignments.get(var).copied().unwrap_or(false);
                output.push_str(&format!("{:>4}", if value { "T" } else { "F" }));
            }
            output.push_str(&format!("{:>8}\n", if row.result { "T" } else { "F" }));
        }
        
        output
    }

    fn format_equivalence_result(&self, check: &EquivalenceCheck, left_str: &str, right_str: &str) -> String {
        let mut output = String::new();
        
        if check.equivalent {
            output.push_str("✓ Expressions are equivalent\n");
            output.push_str(&format!("  Left:  {}\n", left_str));
            output.push_str(&format!("  Right: {}\n", right_str));
        } else {
            output.push_str("✗ Expressions are not equivalent\n");
            output.push_str(&format!("  Left:  {}\n", left_str));
            output.push_str(&format!("  Right: {}\n", right_str));
            output.push_str("\nDifferences:\n");
            
            for diff in check.differences.iter().take(MAX_DIFFERENCES_TO_SHOW) {
                output.push_str("  ");
                for var in check.variables.iter() {
                    let value = diff.assignment.get(var).copied().unwrap_or(false);
                    output.push_str(&format!("{}={} ", var, if value { "T" } else { "F" }));
                }
                output.push_str(&format!("→ Left={}, Right={}\n", 
                    if diff.left_value { "T" } else { "F" },
                    if diff.right_value { "T" } else { "F" }));
            }
            
            if check.differences.len() > MAX_DIFFERENCES_TO_SHOW {
                output.push_str(&format!("  ... and {} more differences\n", check.differences.len() - MAX_DIFFERENCES_TO_SHOW));
            }
        }
        
        output
    }

    fn format_reduction_result(&self, reduction: &Reduction) -> String {
        let mut output = String::new();
        output.push_str(&format!("Expression: {}\n", reduction.original));
        if reduction.simplified {
            output.push_str(&format!("Reduced form: {}\n", reduction.reduced));
        } else {
            output.push_str(&format!("Reduced form: {} (already minimal)\n", reduction.reduced));
        }
        output
    }
}

impl Formatter for JsonFormatter {
    fn format_truth_table(&self, table: &TruthTable) -> String {
        serde_json::to_string_pretty(table).unwrap_or_else(|e| format!("Error serializing to JSON: {}", e))
    }

    fn format_equivalence_result(&self, check: &EquivalenceCheck, left_str: &str, right_str: &str) -> String {
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
        
        serde_json::to_string_pretty(&output).unwrap_or_else(|e| format!("Error serializing to JSON: {}", e))
    }

    fn format_reduction_result(&self, reduction: &Reduction) -> String {
        serde_json::to_string_pretty(reduction).unwrap_or_else(|e| format!("Error serializing to JSON: {}", e))
    }
}

impl Formatter for CsvFormatter {
    fn format_truth_table(&self, table: &TruthTable) -> String {
        let mut output = String::new();
        
        // Header
        for var in table.variables.iter() {
            output.push_str(&format!("{},", var));
        }
        output.push_str("result\n");
        
        // Rows
        for row in &table.rows {
            for var in table.variables.iter() {
                let value = row.assignments.get(var).copied().unwrap_or(false);
                output.push_str(&format!("{},", if value { "true" } else { "false" }));
            }
            output.push_str(&format!("{}\n", if row.result { "true" } else { "false" }));
        }
        
        output
    }

    fn format_equivalence_result(&self, check: &EquivalenceCheck, left_str: &str, right_str: &str) -> String {
        let mut output = String::new();
        output.push_str("equivalent,left_expression,right_expression\n");
        output.push_str(&format!("{},{},{}\n", check.equivalent, left_str, right_str));
        
        if !check.differences.is_empty() {
            output.push_str("\nDifferences:\n");
            // Header for differences
            for var in check.variables.iter() {
                output.push_str(&format!("{},", var));
            }
            output.push_str("left_value,right_value\n");
            
            // Each difference
            for diff in &check.differences {
                for var in check.variables.iter() {
                    let value = diff.assignment.get(var).copied().unwrap_or(false);
                    output.push_str(&format!("{},", if value { "true" } else { "false" }));
                }
                output.push_str(&format!("{},{}\n", 
                    if diff.left_value { "true" } else { "false" },
                    if diff.right_value { "true" } else { "false" }));
            }
        }
        
        output
    }

    fn format_reduction_result(&self, reduction: &Reduction) -> String {
        format!("original,reduced,simplified\n\"{}\",\"{}\",{}\n", 
            reduction.original, reduction.reduced, reduction.simplified)
    }
}

impl Formatter for NuonFormatter {
    fn format_truth_table(&self, table: &TruthTable) -> String {
        let mut output = String::new();
        output.push_str("[\n");
        for (i, row) in table.rows.iter().enumerate() {
            output.push_str("  {");
            
            // Variable assignments
            for (j, var) in table.variables.iter().enumerate() {
                let value = row.assignments.get(var).copied().unwrap_or(false);
                output.push_str(&format!("{}: {}", var, if value { "true" } else { "false" }));
                if j < table.variables.len() - 1 {
                    output.push_str(", ");
                }
            }
            
            // Result
            output.push_str(&format!(", result: {}", if row.result { "true" } else { "false" }));
            output.push('}');
            
            if i < table.rows.len() - 1 {
                output.push_str(",\n");
            } else {
                output.push('\n');
            }
        }
        output.push_str("]\n");
        output
    }

    fn format_equivalence_result(&self, check: &EquivalenceCheck, left_str: &str, right_str: &str) -> String {
        let mut output = String::new();
        output.push_str("{\n");
        output.push_str(&format!("  equivalent: {},\n", if check.equivalent { "true" } else { "false" }));
        output.push_str(&format!("  left_expression: \"{}\",\n", left_str));
        output.push_str(&format!("  right_expression: \"{}\",\n", right_str));
        output.push_str("  differences: [\n");
        
        for (i, diff) in check.differences.iter().enumerate() {
            output.push_str("    {");
            
            // Variable assignments
            for (j, var) in check.variables.iter().enumerate() {
                let value = diff.assignment.get(var).copied().unwrap_or(false);
                output.push_str(&format!("{}: {}", var, if value { "true" } else { "false" }));
                if j < check.variables.len() - 1 {
                    output.push_str(", ");
                }
            }
            
            // Left and right values
            output.push_str(&format!(", left_value: {}, right_value: {}", 
                if diff.left_value { "true" } else { "false" },
                if diff.right_value { "true" } else { "false" }));
            output.push('}');
            
            if i < check.differences.len() - 1 {
                output.push_str(",\n");
            } else {
                output.push('\n');
            }
        }
        
        output.push_str("  ]\n");
        output.push_str("}\n");
        output
    }

    fn format_reduction_result(&self, reduction: &Reduction) -> String {
        format!("{{\n  original: \"{}\",\n  reduced: \"{}\",\n  simplified: {}\n}}\n", 
            reduction.original, reduction.reduced, if reduction.simplified { "true" } else { "false" })
    }
}

pub fn get_formatter(format: &OutputFormat) -> Box<dyn Formatter> {
    match format {
        OutputFormat::Table => Box::new(TableFormatter),
        OutputFormat::Json => Box::new(JsonFormatter),
        OutputFormat::Csv => Box::new(CsvFormatter),
        OutputFormat::Nuon => Box::new(NuonFormatter),
    }
}

pub fn format_truth_table(table: &TruthTable, format: &OutputFormat) -> String {
    get_formatter(format).format_truth_table(table)
}

pub fn format_equivalence_result(check: &EquivalenceCheck, left_str: &str, right_str: &str, format: &OutputFormat) -> String {
    get_formatter(format).format_equivalence_result(check, left_str, right_str)
}

pub fn format_reduction_result(reduction: &Reduction, format: &OutputFormat) -> String {
    get_formatter(format).format_reduction_result(reduction)
}