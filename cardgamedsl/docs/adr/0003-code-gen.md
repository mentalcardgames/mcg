Writing repetitive code for all structs is very annoying. Generate the needed walker/lowering and spanned logic and
keep the ast.rs as single point of truth.

Alternative (future):
Consider salsa for handling spans and validation.

