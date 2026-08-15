# Project Guidelines

## Code Style & Architecture

- **Clean Code & Functional Style**:
  - Prefer declarative and functional patterns (iterator pipelines, `map`, `filter_map`, `find_map`, `fold`, `collect`) over imperative loops where appropriate.
  - Emphasize immutability, pure functions, and minimal mutable state.
  - Adhere strictly to the DRY (Don't Repeat Yourself) principle and single-responsibility functions.
  - Favor zero-allocation patterns (e.g. using `Cow`, borrowed slices, component push on `Multiaddr`, deferred allocations).

- **Idiomatic Rust**:
  - Encapsulate parameters in cohesive configuration structs rather than long argument lists.
  - Ensure type safety with dedicated enums/structs instead of untyped JSON primitives.
  - Handle errors cleanly using `Result` with contextual error messages; never ignore potential serialization or I/O failures.
