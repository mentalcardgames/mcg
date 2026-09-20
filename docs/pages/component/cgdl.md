---
outline: deep
---

# Card Game Description Language (CGDL)

This document describes the design and architecture of the `front_end` and `code_gen` crates within the Card Game Domain-Specific Language (CGDSL) workspace. It provides an overview of the Abstract Syntax Tree (AST), the Finite State Machine (FSM) representation, the core module structure, and how code generation reduces boilerplate for the compiler.

## Code Generation (`code_gen` crate)

The `code_gen` crate exists to eliminate repetitive boilerplate when maintaining the compiler's Abstract Syntax Tree (AST). Building a compiler often requires multiple versions of an AST (e.g., one that tracks source code locations/spans for error reporting, and an unspanned one for simpler evaluation), along with logic to traverse and transform them.

### `#[spanned_ast]` Macro
The core feature of `code_gen` is the `#[spanned_ast]` module attribute macro. It allows developers to define the AST using straightforward, "pure" structs and enums without cluttering them with span tracking and metadata types.

When applied to a module containing pure AST definitions, the macro automatically generates:
1. **Spanned Counterparts**: It creates a mirrored `_spanned` module where every node is wrapped in a `Spanned<T>` wrapper (e.g., `Template` becomes `STemplate = Spanned<Template>`). This allows the compiler to retain precise code location metadata for error diagnostics without manually defining separate spanned versions of each struct.
2. **Lowering Logic (`Lower` trait)**: It auto-generates the transformation logic to convert spanned AST nodes back down into their unspanned counterparts, shedding the location metadata when it is no longer required.
3. **Walker Logic (`Walker` trait)**: It implements a visitor pattern (`AstPass`). The macro builds `walk` implementations to recursively traverse the hierarchical AST elements automatically.
4. **Stripping Trait Derives**: It intelligently scrubs testing traits (like `Arbitrary` or `proptest`) from the generated spanned nodes to keep the generated code clean and ensure traits are only applied where appropriate.

**Why this helps:** Defining a language grammar inherently leads to a deeply nested, bulky AST. Manually mirroring dozens of struct definitions the spanned versions and hand-writing recursive traversal patterns for every single node type are notoriously error-prone and tedious. The `#[spanned_ast]` macro automates this lifecycle entirely, allowing language grammar iteration with zero friction.

## Front End (`front_end` crate)

The `front_end` crate is responsible for correctly processing the DSL, defining the language grammar, parsing the source code, performing semantic analysis, and ultimately lowering the program structure into an intermediate state machine.

### Module Organization
The `front_end` crate is structured into several key functional modules:

- **`ast`**: The heart of the syntax definitions. Contains standard domain structs like `GameRule`, `EndCondition`, `LocationCollection`, `IntCompare`, etc. Driven heavily by the `#[spanned_ast]` macro.
- **`parser`**: Wraps the grammar (compiled usually through a tool like `pest`) and provides functions to consume raw `.cg` string inputs and emit the `Spanned` AST.
- **`ir`**: The Intermediate Representation module. This transforms the recursive AST control flow into an unrolled Finite State Machine (FSM) network.
- **`semantic` / `validation` / `symbols`**: Perform type checking, symbol resolution, and domain validation. Ensures memory locations, variables, and player contexts actually exist and make logical sense before simulation.
- **`walker`**: Traits for visiting and traversing AST nodes recursively.
- **`spans`**: Contains the source tracking structures (`Span`) that index origin rows and columns, used extensively during diagnostic error reporting.
- **`lower`**: Basic traits detailing how spanned AST variants drop their span tokens to become purely interpreted AST logic.
- **`arbitrary`**: Defines utilities and strategies for generating randomized AST constructions, heavily used for fuzz testing and validation.
- **`fmt_ast` & `fsm_to_dot`**: Presentation utilities. `fmt_ast` implements user-friendly representation formats for AST nodes, while `fsm_to_dot` generates visualizing graphs (Graphviz DOT format) for the parsed memory/state machine paths.

### Abstract Syntax Tree (AST)

The AST acts as the hierarchical syntax representation. When the user specifies rules (such as *"move >= 3 and <= 10 from ExampleLocation to ExampleLocation1"*), parsing transforms this unstructured text into strict rust enums and structs natively defined in `front_end/src/ast.rs`. 

The AST enforces properties like:
- **Variable Constraints:** Grouping integer limits (`IntRange`), equality operators (`IntCompare`), or quantities (`Repititions`).
- **Target Addressing:** Specific ownership contexts are differentiated directly in the type system, e.g., `Owner` (Players, Teams, Table) vs `SingleOwner` (`UseMemory`).
- **Scope Definitions:** Capturing contexts like `OutOf::CurrentStage` to identify how logic branches inside looping stages or card actions.

The codebase interacts heavily with the *Spanned AST* (such as `SGame`), retaining contextual bounds for accurate tracebacks.

### Intermediate Representation (IR / FSM)

To evaluate game loops natively and deterministically, the AST is lowered into a **Finite State Machine (FSM)** defined in `front_end/src/ir.rs`. This FSM acts as the execution graph (the Intermediate Representation) simulating chronological progression.

- **`StateID` (Graph Nodes):** Represents a distinct state or checkpoint within the game iteration. A state inherently has no logic; it holds graph topology relationships.
- **`Edge` and `Payload` (Graph Vertices):** The connective tissue between states is an `Edge`, mapping a source `StateID` to a destination `StateID`. Edges hold `Payload`s heavily influencing the game flow rules.
  - A payload contains contextual game logic: evaluating a condition (`Payload::Condition`), firing an action (`Payload::Action(GameRule)`), triggering endgame flags (`EndGame`, `EndCurrentStage`), or requesting a multi-choice dialogue (`Payload::Choice`).
- **`Meta` Data:** Edges also track supplemental metadata like `SimStageEndCondition` allowing external simulation runners to gauge context on when limits are reached globally across nodes.

By mapping `SGame.to_graph()`, the procedural DSL script unravels completely. The resulting map guarantees paths between operations can be validated, checked for strict connectivity (e.g., verifying `reachable_from_entry()`), and easily serialized for arbitrary runtime systems simulating the game via state transitions.


## Usage: Parsing and Generating Data Structures

To interact with the language programmatically, you can leverage the exposed parser definitions from `pest_consume` to read a raw `.cg` program and directly translate it into the fully constructed internal representations (AST and IR).

Below is an example of loading a file, parsing it, and generating all variations of the AST and execution Intermediate Representation / Finite State Machine (FSM):

```rust
use pest_consume::Parser;
use front_end::parser::{CGDSLParser, Rule};
use front_end::lower::Lower;

// 1. Loading the file into a String
let input_code = std::fs::read_to_string("my_game.cg").expect("Failed to read file");

// 2. Initial Parsing with `pest_consume`
// This parses the string into structural nodes defined by our pest grammar
let nodes = CGDSLParser::parse_with_userdata(Rule::file, &input_code, Default::default()).unwrap_or_else(|e| {
    // Alternatively, use CGDSLParser::parse if no custom userdata/symbol table is strictly required initially
    CGDSLParser::parse(Rule::file, &input_code).expect("Parsing failed")
});
let root_node = nodes.single().expect("Expected a single root node");

// 3. Spanned AST
// Generating the exact mirrored representation of the code, retaining position tokens
// useful for error reporting.
let spanned_ast = CGDSLParser::file(root_node).expect("Failed to map into AST");

// 4. Lowered AST
// Stripping the spanned metadata to obtain the "pure" unspanned logic objects
let lowered_ast = spanned_ast.lower();

// 5. Spanned IR (FSM)
// Converting the execution graph (control flow) into a traversable State Machine
// utilizing exact token mappings.
let spanned_ir = spanned_ast.to_graph();

// 6. Lowered IR (FSM)
// Generating a clean logic-only execution map, optimizing away the parse spans.
let lowered_ir = spanned_ast.to_lowered_graph();
```
