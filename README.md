# ttt

`ttt` is a command-line utility for checking truth tables and optimizing boolean functions. It provides three main operations: generating truth tables, checking expression equivalence, and simplifying boolean expressions using the Quine-McCluskey algorithm.

## Usage

ttt supports three main commands, each with multiple output formats. You can provide expressions as command-line arguments or read them from standard input.

### Commands

#### `table` - Generate Truth Tables

Generate a complete truth table for a boolean expression:

```bash
ttt table "a and b"
```

```text
expression: (a ∧ b)
   a   b  Result
----------------
   F   F       F
   T   F       F
   F   T       F
   T   T       T
```

Complex expressions with multiple variables:

```bash
ttt table "a xor b and c"
```

```text
expression: (a ⊕ (b ∧ c))
   a   b   c  Result
--------------------
   F   F   F       F
   F   F   T       F
   F   T   F       F
   F   T   T       T
   T   F   F       T
   T   F   T       T
   T   T   F       T
   T   T   T       F
```

#### `eq` - Check Expression Equivalence

Check if two boolean expressions are logically equivalent:

```bash
ttt eq "a and b" "b and a"
```

```text
✓ Expressions are equivalent
  Left:  a and b
  Right: b and a
```

When expressions are not equivalent, ttt shows the differing truth assignments:

```bash
ttt eq "a or b" "a and b"
```

```text
✗ Expressions are not equivalent
  Left:  a or b
  Right: a and b

Differences:
  a=F b=T → Left=T, Right=F
  a=T b=F → Left=T, Right=F
```

#### `reduce` - Simplify Boolean Expressions

Simplify boolean expressions using the Quine-McCluskey algorithm:

```bash
ttt reduce "a and b or a and not b"
```

```text
Expression: ((a ∧ b) ∨ (a ∧ ¬b))
Reduced form: a
```

More complex example:

```bash
ttt reduce "(a and b and c) or (a and b and not c) or (a and not b and c)"
```

```text
Expression: (((a ∧ b) ∧ c) ∨ (((a ∧ b) ∧ ¬c) ∨ ((a ∧ ¬b) ∧ c)))
Reduced form: ((a ∧ b) ∨ (a ∧ c))
```

### Output Formats

All commands support multiple output formats using the `-o` or `--output` flag:

- `table` (default) - Human-readable format
- `json` - JSON format for programmatic use
- `csv` - Comma-separated values
- `nuon` - Nushell object notation

### Reading from Standard Input

All commands can read expressions from standard input when no arguments are provided:

```bash
echo "a and b" | ttt table
```

For equivalence checking, provide two expressions on separate lines:

```bash
echo -e "a and b\nb and a" | ttt eq
```

## Boolean Expression Grammar

ttt supports a flexible grammar for boolean expressions with multiple operator formats.

### Operators

| Operator         | Operation                          | Precedence |
|------------------|------------------------------------|------------|
| `!`, `¬`, `not`  | logical not (prefix)               | 1 (highest)|
| `&&`, `∧`, `and` | logical and                        | 2          |
| `||`, `∨`, `or`  | logical or                         | 3          |
| `xor`, `⊻`, `⊕`  | exclusive or                       | 4          |
| `->`, `→`        | material conditional/implication   | 5 (lowest) |

### Identifiers

- Variable names must be alphabetic characters (a-z, A-Z)
- Cannot use reserved keywords: `and`, `or`, `not`, `xor`
- Case-sensitive
- Maximum length: 50 characters

### Grammar Rules

```text
expression     = implication
implication    = xor (('->' | '→') xor)*
xor            = or (('xor' | '⊻' | '⊕') or)*
or             = and (('or' | '||' | '∨') and)*
and            = not (('and' | '&&' | '∧') not)*
not            = ('not' | '!' | '¬')? primary
primary        = identifier | '(' expression ')'
identifier     = [a-zA-Z] [a-zA-Z0-9_]*
```

## Syntax Errors

ttt uses miette to provide nice looking syntax errors:

```bash
ttt table "a and"
```

```text
Error: ttt::parser::unexpected_token

  × Unexpected token: expected identifier or '(', found end of input
   ╭─[expression:1:6]
 1 │ a and
   ·      ┬
   ·      ╰── unexpected token here
   ╰────
  help: The expression appears to be incomplete
```

## Misc

ttt was built primarily as an experiment with Claude Code.
