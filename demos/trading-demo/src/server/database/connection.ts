/**
 * Database Connection - Better-SQLite3 Setup
 *
 * Initializes SQLite database for Trading Demo with proper schema and seed data.
 * Uses better-sqlite3 for synchronous, high-performance database operations.
 */

import Database from 'better-sqlite3';
import { readFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { mkdirSync } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

/**
 * Database Configuration
 */
export interface DatabaseConfig {
  /** Database file path (or ':memory:' for in-memory) */
  filename: string;

  /** Enable verbose logging */
  verbose?: boolean;

  /** Read-only mode */
  readonly?: boolean;

  /** File must exist */
  fileMustExist?: boolean;
}

/**
 * Default Development Configuration
 */
const defaultConfig: DatabaseConfig = {
  filename: process.env.NODE_ENV === 'test'
    ? ':memory:'
    : process.env.DB_PATH || './data/trading-demo.db',
  verbose: process.env.NODE_ENV === 'development',
  readonly: false,
  fileMustExist: false,
};

/**
 * SQLite Database Instance
 */
let db: Database.Database | null = null;

/**
 * Initialize Database Connection
 */
export async function initializeDatabase(config: Partial<DatabaseConfig> = {}): Promise<Database.Database> {
  const finalConfig: DatabaseConfig = { ...defaultConfig, ...config };

  try {
    // Ensure data directory exists
    if (finalConfig.filename !== ':memory:') {
      const dbDir = dirname(finalConfig.filename);
      mkdirSync(dbDir, { recursive: true });
    }

    // Create database connection
    db = new Database(finalConfig.filename, {
      verbose: finalConfig.verbose ? console.log : undefined,
      readonly: finalConfig.readonly,
      fileMustExist: finalConfig.fileMustExist,
    });

    // Enable WAL mode for better performance
    db.pragma('journal_mode = WAL');

    // Enable foreign keys
    db.pragma('foreign_keys = ON');

    console.log('✅ Database connection initialized');

    // Create schema
    await createSchema(db);

    return db;
  } catch (error) {
    console.error('❌ Database initialization failed:', error);
    throw error;
  }
}

/**
 * Create Database Schema
 */
async function createSchema(database: Database.Database): Promise<void> {
  try {
    const schemaPath = join(__dirname, 'schema.sql');
    const schema = readFileSync(schemaPath, 'utf-8');

    // Execute schema SQL
    database.exec(schema);

    console.log('✅ Database schema created');
  } catch (error) {
    console.error('❌ Schema creation failed:', error);
    throw error;
  }
}

/**
 * Get Database Connection
 */
export function getDatabase(): Database.Database {
  if (!db) {
    throw new Error('Database not initialized. Call initializeDatabase() first.');
  }
  return db;
}

/**
 * Close Database Connection
 */
export async function closeDatabase(): Promise<void> {
  if (db) {
    db.close();
    db = null;
    console.log('✅ Database connection closed');
  }
}

/**
 * Execute Query (for compatibility)
 */
export async function executeQuery<T = any>(
  query: string,
  parameters?: any[]
): Promise<T[]> {
  const database = getDatabase();
  const stmt = database.prepare(query);
  return stmt.all(...(parameters || [])) as T[];
}

/**
 * Health Check
 */
export async function checkDatabaseHealth(): Promise<boolean> {
  try {
    const database = getDatabase();
    database.prepare('SELECT 1').get();
    return true;
  } catch (error) {
    console.error('Database health check failed:', error);
    return false;
  }
}

/**
 * Transaction Helper
 */
export function transaction<T>(fn: (db: Database.Database) => T): T {
  const database = getDatabase();
  return database.transaction(fn)();
}

/**
 * Prepare Statement Helper
 */
export function prepare(query: string) {
  return getDatabase().prepare(query);
}
