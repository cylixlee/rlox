# Error Codes

This document contains all error codes used by the `rlox-analyzer`.

Note that all error diagnostics are performed in scanning and parsing procedure.
Codegen step may fail with no detailed message, and leads to panic at runtime.

## Scanning

| Code  | Message                  | Explanation                             |
|-------|--------------------------|-----------------------------------------|
| E0001 | Unrecognized token       | Invalid token encountered               |
| E0002 | Unparsable float literal | Float literal cannot be parsed as `f64` |

## Parsing

| Code  | Message                   | Explanation                             |
|-------|---------------------------|-----------------------------------------|
| E0003 | Unexpected EOF            | Incomplete code segment                 |
| E0004 | Invalid prefix expression | Token cannot be prefix of an expression |
| E0005 | Unexpected token          | Expected another type of token          |