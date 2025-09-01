# ttt

`ttt` is a command line utility for checking truth tables and optimizing boolean functions. Aside from wanting to make a useful tool, this was also an experiment in using Claude Code.

## Usage
ttt can be used either to generate a truth table from a boolean expression or check the equivalence of a boolean expression.

### Generating truth tables
Use the `table` command to make a truth table.

``` shell
ttt table a or not b
```

### Checking equivalence
Use the `eq` command to check expression equivalency:

``` shell
ttt eq --left a or not b --right not a or b
```

### Checking for difference
Similar to `eq`, ttt can describe how two expressions differ:

``` shell
ttt eq --left a or not b --right not a or b
```

### Reducing an expression
Use the `reduce` command to simplify an expression, if possible.

## Grammar

ttt's grammar is designed to flexibly articulate boolean functions.
ttt only contains identifiers and operators.

### Operators

| Operator         | Operation                          |
|------------------|------------------------------------|
| `&&`, `∧`, `and` | logical and                        |
| `||`, `∨`, `or`  | logical or                         |
| `!`, `¬`, `not`  | logical not (prefix operator)      |
| `->`, `→`        | material conditional / implication |
| `xor`, `⊻`, `⊕`  | exclusive or                       |

### Grammar

``` text
unary operator  = not
binary operator = and | or | xor | implication
identifier      = alphabetic set of characters that is not a keyword
expr            = (unary operator)? identifier ((binary operator) expr)?
```

