import eslint from '@eslint/js';
import tseslint from '@typescript-eslint/eslint-plugin';
import tsparser from '@typescript-eslint/parser';
import prettierConfig from 'eslint-config-prettier';

export default [
  eslint.configs.recommended,
  {
    files: ['src/**/*.ts'],
    languageOptions: {
      parser: tsparser,
      parserOptions: {
        project: './tsconfig.json',
        ecmaVersion: 2022,
        sourceType: 'module',
      },
    },
    plugins: {
      '@typescript-eslint': tseslint,
    },
    rules: {
      ...tseslint.configs.recommended.rules,
      // Type safety - Warnings for gradual improvement
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/no-unused-vars': [
        'warn',
        { 
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_'
        },
      ],
      // Console statements - Allow in specific contexts
      'no-console': ['warn', { allow: ['warn', 'error', 'info', 'debug'] }],
      'no-undef': 'off', // TypeScript handhabt das
      // Modern ES features
      'prefer-const': 'warn',
      'no-var': 'error',
    },
  },
  // CLI and bin files - more lenient rules
  {
    files: ['src/cli/**/*.ts', 'bin/**/*.js'],
    rules: {
      'no-console': 'off', // Console is expected in CLI
      '@typescript-eslint/no-explicit-any': 'off', // CLI uses flexible types
      '@typescript-eslint/no-unused-vars': 'off', // May have interface params
    },
  },
  // Test files - more lenient rules
  {
    files: ['src/tests/**/*.ts', '**/*.test.ts', '**/*.spec.ts'],
    rules: {
      '@typescript-eslint/no-explicit-any': 'off', // Tests often need any
      'no-console': 'off', // Console is fine in tests
    },
  },
  // Legacy/adapter files and feature code - gradual migration
  {
    files: [
      'src/adapters/**/*.ts',
      'src/services/**/*.ts',
      'src/legacy/**/*.ts',
      'src/generators/**/*.ts',
      'src/parsers/**/*.ts',
      'src/api/**/*.ts',
      'src/sdk/**/*.ts',
      'src/core/**/*.ts',
      'src/analyzers/**/*.ts',
      'src/tests/**/*.ts',
      'src/reports/**/*.ts',
      'src/interfaces/**/*.ts',
      'src/types/**/*.ts',
      'src/types.ts',
      'src/utils/**/*.ts',
      'src/validators/**/*.ts',
      'src/index.ts',
    ],
    rules: {
      '@typescript-eslint/no-explicit-any': 'off', // Legacy code - refactor phase 2
      '@typescript-eslint/no-unused-vars': 'off', // May have interface placeholders
      'no-console': 'off', // May need debug/info output
    },
  },
  {
    ignores: [
      'dist/**',
      'node_modules/**',
      'coverage/**',
      '*.js',
      'bin/audit.js',
      '.husky/**',
      'jest.config.js',
      'scripts/**',
    ],
  },
  prettierConfig,
];
