The walker/visitor-pattern is a simple modular and extendible. This is great for fast 
prototyping and extending the grammar and Abstract Syntax Tree without having to worry about too many things at once.
If the files become really big then it makes sense to switch to a more optimized validation (e.g. query-based).
It only makes sense to optimize and refactor everything if there are fewer additions to the grammar and/or the validation becomes very slow.

Alternatives (future):
salsa