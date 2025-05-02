# Copilot Instructions

## Styling

For this repo we use scss, all styles go in `packages/yew-frontend/src/styles` and are imported via `packages/yew-frontend/src/styles/index.scss`

## Code Organization

- This is a Rust workspace project with members in the `packages/` directory
- The codebase follows a frontend/backend architecture:
  - `packages/yew-frontend/` - Frontend code using Yew framework
  - `packages/actix-backend/` - Backend code using Actix
  - `packages/shared/` - Shared code between frontend and backend

## Best Practices

- Avoid adding comments unless explicitly requested
- Avoid explanations unless explicitly requested
- Focus tests on recently added code
- Avoid examples unless explicitly requested
- Adhere to functional programming principles where applicable
- Use the same programming language for tests as the code being tested
- Use the same testing framework for tests as the code being tested

## Code Design Principles

- Apply the DRY (Don't Repeat Yourself) principle by eliminating code duplication through abstraction
- Follow the KISS (Keep It Simple, Stupid) principle by writing straightforward and maintainable code
- Adopt the YAGNI (You Aren't Gonna Need It) principle by implementing features only when necessary
- Implement the SOLID principles to enhance code maintainability and scalability:
  - Single Responsibility: Ensure each module or function has a single responsibility
  - Open-Closed: Design modules to be open for extension but closed for modification
  - Liskov Substitution: Use subtypes that are substitutable for their base types
  - Interface Segregation: Create specific interfaces rather than general-purpose ones
  - Dependency Inversion: Use abstractions to depend on interfaces rather than concrete implementations

## Performance Considerations

- Avoid premature optimization; prioritize code clarity and correctness
- For release builds, the project uses aggressive optimization settings:
  - Panic strategy: abort
  - Code generation units: 1
  - Optimization level: 'z' (size optimization)
  - Link-time optimization: enabled

## Coding Standards

- Maintain consistent naming conventions and code formatting throughout the project
- Write modular and reusable code to facilitate easier maintenance and testing
- Follow Rust idioms and best practices

## Bevy

We are using latest, SpriteBundle is deprecated, Sprite.
