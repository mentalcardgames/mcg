.
├── cgdsl
│   ├── CHANGELOG.md
│   ├── eslint.config.mjs
│   ├── icons  # symbol of language
│   │   ├── cgdsl-dark-icon.svg
│   │   └── cgdsl-light-icon.svg
│   ├── package.json  # scripts
│   ├── src
│   │   ├── extension.ts  
│   │   ├── generate-grammar.ts  # generating grammar
│   │   ├── keywords.ts  # keeping track of keywords
│   │   └── test
│   ├── syntaxes
│   │   └── cgdsl.tmLanguage.json  # generated grammar
│   ├── themes  # themes for coloring
│   │   └── cgdsl-dark.json
│   ├── tsconfig.json
│   └── vsc-extension-quickstart.md
├── code_gen
│   └── src
│       └── lib.rs  # #[spanned_ast] generation logic for front_end
├── docs
│   ├── adr  # ADRs (Architecture Design Records)
│   │   ├── 0001-using-pest-for-parsing.md
│   │   ├── 0002-walker-visitor.md
│   │   └── 0003-code-gen.md
│   ├── architecture
│   │   ├── architecture.tex  # architecture document
│   │   └── diagrams  # architecture diagrams (in .puml)
│   └── development.md
├── front_end
│   ├── build.rs  # generates dummy auto-completion
│   └── src
│       ├── arbitrary.rs  # testing logic for generating an arbitrary Abstract Syntax Tree
│       ├── ast.rs  # declaration of Abstract Syntax Tree
│       ├── fmt_ast.rs  # formatter logic of Abstract Syntax Tree (should mirror the corresponding grammar rules)
│       ├── fsm_to_dot.rs  # transform an FSM (the IR) into a *.dot (for visualization)
│       ├── grammar.pest  # grammar
│       ├── ir.rs  # IR transformation and logic
│       ├── lib.rs
│       ├── lower.rs  # lower trait declaration
│       ├── parser.rs  # parse tree to Abstract Syntax Tree logic
│       ├── semantic.rs  # dummy semantic checks
│       ├── spans.rs  # span logic and declaration
│       ├── symbols.rs  # dummy symbol checks
│       ├── tests.rs
│       ├── validation.rs  # validation functions for an Abstract Syntax Tree (semantic, symbol, program)
│       └── walker.rs  # walker logic and declaration
├── lsp_server
│   └── src
│       ├── completion.rs  # auto-completion logic
│       ├── error_to_diagnostics.rs  # helper for transforming custom errors into tower-lsp Diagnostics
│       ├── lsp.rs  # lsp logic
│       ├── main.rs  # server logic
│       ├── rope.rs  # document logic with rope
│       ├── semantic_highlighting.rs  # defining semantic tokens and highlighting
│       ├── tests.rs
│       └── validation.rs  # validation for diagnostics
└── structure.md