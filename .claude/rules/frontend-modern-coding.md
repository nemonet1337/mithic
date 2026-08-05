# Frontend Modern Coding Standards

## Core Principles
- Write concise, modern code by actively leveraging **current browser standard features** (Chrome and Firefox) and the latest language/library capabilities.
- Prefer native platform features over external libraries when they sufficiently solve the problem.
- Prefer the newest stable features of the tools in use (especially ES2026+ and Tailwind CSS v4) to reduce boilerplate and improve clarity.

## JavaScript / TypeScript (ES2026+)
- Use modern ECMAScript features that are supported in current Chrome and Firefox.
- Prefer native solutions for common tasks (e.g., structuredClone, Object.groupBy, Promise.withResolvers, Iterator helpers, etc.) when available.
- Favor declarative and expressive syntax over verbose imperative code.
- Use modern module patterns and top-level await where appropriate.

## Browser Standard Features
- Prefer native browser APIs over polyfills or heavy libraries when browser support (Chrome + Firefox) is sufficient.
- Examples of preferred native features:
  - Native CSS features (cascade layers, `@property`, `color-mix()`, container queries, `:has()`, etc.)
  - View Transitions API, Popover API, Anchor Positioning, and other modern UI primitives when applicable
  - Native form validation, dialog, and other built-in elements/behaviors
- Avoid large compatibility layers unless older browser support is an explicit project requirement.

## Tailwind CSS v4
- Use Tailwind CSS v4’s **CSS-first** configuration approach.
- Prefer the new engine and modern utilities (including improved gradients, container queries, and other v4 additions).
- Write concise utility-first markup. Avoid unnecessary custom CSS when Tailwind utilities or `@theme` can express the design.
- Leverage new v4 capabilities rather than falling back to older v3 patterns or workarounds.
- Keep custom design tokens and theme extensions in the CSS-first style recommended by Tailwind v4.

## General Frontend Practices
- Prefer progressive enhancement and modern defaults.
- Keep components focused and avoid over-abstraction.
- Favor clarity and short, intention-revealing code over defensive compatibility shims (unless required).
- When choosing libraries, prefer those that embrace modern browser features and have good tree-shaking / minimal runtime cost.