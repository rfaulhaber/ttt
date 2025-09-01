# ttt

`ttt` is a command line utility for checking truth tables and optimizing boolean functions. Aside from wanting to make a useful tool, this was also an experiment in using Claude Code.

## Usage
ttt can be used either to generate a truth table from a boolean expression or check the equivalence of a boolean expression.

### Generating truth tables
Use the `table` command to make a truth table.

``` shell
ttt table "a or not b"
```

```text
   a   b  Result
----------------
   F   F       T
   F   T       F
   T   F       T
   T   T       T
```

### Checking equivalence
Use the `eq` command to check expression equivalency:

``` shell
ttt eq "a or not b" "not a or b"
```

```text
✗ Expressions are not equivalent
  Left:  a or not b
  Right: not a or b

Differences:
  a=F b=T → Left=F, Right=T
  a=T b=F → Left=T, Right=F
```

### Reducing an expression
Use the `reduce` command to simplify an expression, if possible.

```shell
ttt reduce "a and a or not b"
```

```text
Expression: ((a ∧ a) ∨ ¬b)
Reduced form: (¬b ∨ a)
```

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

