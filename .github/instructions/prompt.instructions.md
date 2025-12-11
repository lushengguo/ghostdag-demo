---
applyTo: '**'
---


## Core Principles
- **ATTENTION!!!!!!!** When generating shell commands, avoid using '> /dev/null' or similar output redirections, as this may prevent auto-approve and interactive confirmation steps from working.  
- **Complete code only** - No placeholders, no "shall I continue?"
- **Auto-fix errors** - Iterate and resolve issues yourself
- **One-shot multi-file** - Give all files in one response with full content
- **English by default** - Code, comments, variables in English unless specified
- **Do not create documentation files** - Only code files unless explicitly requested

## Quality Checklist
- ✅ Error handling + input validation
- ✅ Security: sanitize inputs, no hardcoded secrets
- ✅ Performance: efficient algorithms, lazy loading
- ✅ Tests: unit tests for business logic
- ✅ Documentation: JSDoc/rustdoc for public APIs
- ✅ Clean: no dead code, no console.log, DRY principle

## When to Ask
- Business logic is ambiguous
- Need external credentials/API keys
- Multiple valid approaches with tradeoffs

## Framework Best Practices
- **React/Next.js**: Functional components, hooks, server components
- **Rust**: `Result<T,E>`, `?` operator, no panics
- **Python**: Type hints, asyncio for I/O
- **Solidity/Move**: Reentrancy checks, math validation


Do not compile release mode code
