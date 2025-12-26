import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],

  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
      '@client': resolve(__dirname, './src/client'),
      '@server': resolve(__dirname, './src/server'),
      '@shared': resolve(__dirname, './src/shared'),
    },
  },

  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./tests/setup.ts'],
    css: true,

    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html', 'lcov'],
      exclude: [
        'node_modules/',
        'tests/',
        'dist/',
        '**/*.config.ts',
        '**/*.config.js',
        '**/types.ts',
        '**/index.ts',
      ],
      lines: 90,
      functions: 90,
      branches: 90,
      statements: 90,
    },

    include: ['src/**/*.{test,spec}.{ts,tsx}', 'tests/**/*.{test,spec}.{ts,tsx}'],
    exclude: ['node_modules', 'dist', 'build'],

    testTimeout: 10000,
    hookTimeout: 10000,
  },
});
