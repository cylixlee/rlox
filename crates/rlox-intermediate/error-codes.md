# Error Codes

`rlox-analyzer` adopts `codespan-reporting` crate and thus able to report syntax
errors in an elegant way.

For more detail, see [errors.rs](src/errors.rs)

> **Note**: `rlox-analyzer` and other components of `rlox` provide **no** guarantee on
> programs' correctness. Wrong programs may compile, and lead to panic at runtime.