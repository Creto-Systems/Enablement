# Testing Guide - London School TDD

## Overview

This project follows the **London School (mockist)** approach to TDD:
- Focus on **behavior** and **interactions** between objects
- Use **mocks** to isolate units and define contracts  
- Test **how objects collaborate**, not just what they contain
- Drive development **outside-in** from user behavior to implementation

## Running Tests

```bash
npm test                 # Run all tests
npm test:coverage        # Generate coverage report  
npm test:ui              # Open Vitest UI
```

## Test Coverage

All components and hooks have comprehensive tests:
- ✅ AgentCard - 8 tests
- ✅ PortfolioChart - 9 tests
- ✅ TradeForm - 10 tests
- ✅ UsageMeter - 10 tests
- ✅ ApprovalCard - 11 tests
- ✅ usePortfolio - 9 tests
- ✅ useTrade - 9 tests

**Total: 66+ tests targeting 90% coverage**
