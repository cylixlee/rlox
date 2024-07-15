# Error Codes

`rlox-analyzer` adopts `codespan-reporting` crate and thus able to report syntax
errors in an elegant way.

For more detail, see [errors.rs](src/errors.rs)

> **Note**: All error diagnostics are performed in scanning and parsing procedure.
> Codegen step may fail with no detailed message, and leads to panic at runtime.