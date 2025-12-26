import { randomUUID } from 'crypto';
import { getDatabase } from '../database/connection';
import { Portfolio, Position, PerformanceMetrics } from '../../types/models';

export class PortfolioService {
  async getPortfolio(agentId: string): Promise<Portfolio | null> {
    const db = getDatabase();

    // Get portfolio
    const portfolioStmt = db.prepare('SELECT * FROM portfolios WHERE agent_id = ?');
    const portfolioRow = portfolioStmt.get(agentId) as any;

    if (!portfolioRow) {
      return null;
    }

    // Get positions
    const positionsStmt = db.prepare('SELECT * FROM positions WHERE portfolio_id = ?');
    const positionRows = positionsStmt.all(portfolioRow.id) as any[];

    const positions: Position[] = positionRows.map(row => ({
      id: row.id,
      portfolioId: row.portfolio_id,
      symbol: row.symbol,
      quantity: row.quantity,
      avgPrice: row.avg_price,
      currentPrice: row.current_price,
      marketValue: row.market_value,
      pnl: row.pnl,
      pnlPercent: row.pnl_percent,
      openedAt: new Date(row.opened_at),
      updatedAt: new Date(row.updated_at),
    }));

    return {
      id: portfolioRow.id,
      agentId: portfolioRow.agent_id,
      positions,
      totalValue: portfolioRow.total_value,
      cashBalance: portfolioRow.cash_balance,
      dailyChange: portfolioRow.daily_change,
      dailyChangePercent: portfolioRow.daily_change_percent,
      realizedPnL: portfolioRow.realized_pnl,
      unrealizedPnL: portfolioRow.unrealized_pnl,
      metrics: JSON.parse(portfolioRow.metrics),
      updatedAt: new Date(portfolioRow.updated_at),
    };
  }

  async updatePosition(
    agentId: string,
    symbol: string,
    quantity: number,
    price: number
  ): Promise<void> {
    const db = getDatabase();
    const portfolio = await this.getPortfolio(agentId);

    if (!portfolio) {
      throw new Error('Portfolio not found');
    }

    // Find existing position
    const existingPos = portfolio.positions.find(p => p.symbol === symbol);

    if (existingPos) {
      // Update existing position
      const newQuantity = existingPos.quantity + quantity;

      if (newQuantity <= 0) {
        // Close position
        const deleteStmt = db.prepare('DELETE FROM positions WHERE id = ?');
        deleteStmt.run(existingPos.id);
      } else {
        // Update position
        const totalCost = existingPos.quantity * existingPos.avgPrice + quantity * price;
        const newAvgPrice = totalCost / newQuantity;

        const updateStmt = db.prepare(`
          UPDATE positions
          SET quantity = ?, avg_price = ?, current_price = ?, market_value = ?, pnl = ?, pnl_percent = ?, updated_at = ?
          WHERE id = ?
        `);

        const marketValue = newQuantity * price;
        const pnl = marketValue - (newQuantity * newAvgPrice);
        const pnlPercent = (pnl / (newQuantity * newAvgPrice)) * 100;

        updateStmt.run(
          newQuantity,
          newAvgPrice,
          price,
          marketValue,
          pnl,
          pnlPercent,
          new Date().toISOString(),
          existingPos.id
        );
      }
    } else if (quantity > 0) {
      // Create new position
      const insertStmt = db.prepare(`
        INSERT INTO positions (id, portfolio_id, symbol, quantity, avg_price, current_price, market_value, pnl, pnl_percent, opened_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      `);

      const marketValue = quantity * price;
      const now = new Date().toISOString();

      insertStmt.run(
        randomUUID(),
        portfolio.id,
        symbol,
        quantity,
        price,
        price,
        marketValue,
        0,
        0,
        now,
        now
      );
    }

    // Update portfolio totals
    await this.recalculatePortfolio(agentId);
  }

  async recalculatePortfolio(agentId: string): Promise<void> {
    const db = getDatabase();
    const portfolio = await this.getPortfolio(agentId);

    if (!portfolio) {
      return;
    }

    const totalValue = portfolio.positions.reduce((sum, pos) => sum + pos.marketValue, 0) + portfolio.cashBalance;
    const unrealizedPnL = portfolio.positions.reduce((sum, pos) => sum + pos.pnl, 0);

    const updateStmt = db.prepare(`
      UPDATE portfolios
      SET total_value = ?, unrealized_pnl = ?, updated_at = ?
      WHERE id = ?
    `);

    updateStmt.run(
      totalValue,
      unrealizedPnL,
      new Date().toISOString(),
      portfolio.id
    );

    // Record history snapshot
    await this.recordHistory(portfolio.id, agentId, totalValue, portfolio.cashBalance);
  }

  async updateCashBalance(agentId: string, amount: number): Promise<void> {
    const db = getDatabase();
    const portfolio = await this.getPortfolio(agentId);

    if (!portfolio) {
      throw new Error('Portfolio not found');
    }

    const updateStmt = db.prepare(`
      UPDATE portfolios
      SET cash_balance = cash_balance + ?, updated_at = ?
      WHERE id = ?
    `);

    updateStmt.run(amount, new Date().toISOString(), portfolio.id);
    await this.recalculatePortfolio(agentId);
  }

  async getPortfolioHistory(agentId: string, days: number = 30): Promise<Array<{ timestamp: string; value: number }>> {
    const db = getDatabase();
    const portfolio = await this.getPortfolio(agentId);

    if (!portfolio) {
      return [];
    }

    const stmt = db.prepare(`
      SELECT timestamp, total_value as value
      FROM portfolio_history
      WHERE portfolio_id = ?
      ORDER BY timestamp DESC
      LIMIT ?
    `);

    const rows = stmt.all(portfolio.id, days) as any[];
    return rows.reverse();
  }

  private async recordHistory(portfolioId: string, agentId: string, totalValue: number, cashBalance: number): Promise<void> {
    const db = getDatabase();
    const stmt = db.prepare(`
      INSERT INTO portfolio_history (id, portfolio_id, agent_id, total_value, cash_balance, timestamp)
      VALUES (?, ?, ?, ?, ?, ?)
    `);

    stmt.run(
      randomUUID(),
      portfolioId,
      agentId,
      totalValue,
      cashBalance,
      new Date().toISOString()
    );
  }
}
