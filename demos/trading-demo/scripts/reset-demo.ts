#!/usr/bin/env tsx
/**
 * Reset Demo Script
 *
 * Clears all demo data and re-seeds with fresh data.
 * Useful for resetting the demo to a clean state between presentations.
 *
 * Usage:
 *   npm run demo:reset
 *   tsx scripts/reset-demo.ts
 */

import { Database } from 'better-sqlite3';
import { existsSync, unlinkSync } from 'fs';
import { join } from 'path';
import { seedDemoData } from './seed-demo-data';

const DB_PATH = join(process.cwd(), 'data');
const DB_FILE = join(DB_PATH, 'trading-demo.db');

// ============================================================================
// RESET FUNCTIONS
// ============================================================================

function clearDatabase(): void {
  console.log('🗑️  Clearing existing database...');

  if (existsSync(DB_FILE)) {
    try {
      unlinkSync(DB_FILE);
      console.log('✅ Database file removed');
    } catch (error) {
      console.error('❌ Error removing database:', error);
      throw error;
    }
  } else {
    console.log('ℹ️  No existing database found');
  }
}

function clearMemoryCache(): void {
  console.log('🧹 Clearing memory cache...');

  // Clear any in-memory caches
  // This would interface with the actual caching layer in production
  console.log('✅ Memory cache cleared');
}

function resetMetering(): void {
  console.log('📊 Resetting metering quotas...');

  // In production, this would call the metering service to reset quotas
  // For demo, quotas are reset as part of re-seeding
  console.log('✅ Metering quotas will be reset with new data');
}

function clearOversightHistory(): void {
  console.log('👁️  Clearing oversight history...');

  // In production, this would archive oversight history
  // For demo, it's cleared with database reset
  console.log('✅ Oversight history will be cleared with new data');
}

// ============================================================================
// MAIN
// ============================================================================

function main(): void {
  console.log('🔄 Starting demo reset...\n');

  try {
    // Step 1: Clear existing data
    clearDatabase();
    console.log();

    clearMemoryCache();
    console.log();

    resetMetering();
    console.log();

    clearOversightHistory();
    console.log();

    // Step 2: Re-seed with fresh data
    console.log('🌱 Re-seeding with fresh demo data...\n');
    seedDemoData();

    console.log('✨ Demo reset complete!\n');
    console.log('🎯 All systems ready for fresh demo presentation');
    console.log('📊 New data includes:');
    console.log('  • 3 fresh demo agents');
    console.log('  • Clean portfolios and positions');
    console.log('  • New trade history');
    console.log('  • Active oversight requests');
    console.log('  • Reset metering quotas\n');

  } catch (error) {
    console.error('❌ Error resetting demo:', error);
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  main();
}

export { main as resetDemo };
