import '@testing-library/jest-dom';

// Mock console methods for cleaner test output
global.console = {
  ...console,
  error: jest.fn(),
  warn: jest.fn(),
};
