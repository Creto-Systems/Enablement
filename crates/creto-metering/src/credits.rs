//! Prepaid credits and wallet management.
//!
//! Supports prepaid credit packages that can be consumed before invoicing.
//! Follows Lago's credit management patterns.

use chrono::{DateTime, Utc};
use creto_common::{types::Money, CretoError, CretoResult, OrganizationId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// A wallet holding prepaid credits for an organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// Unique wallet ID.
    pub id: Uuid,

    /// Organization this wallet belongs to.
    pub organization_id: OrganizationId,

    /// Current credit balance in cents.
    pub balance_cents: i64,

    /// Total credits ever granted.
    pub credits_granted: i64,

    /// Total credits consumed.
    pub credits_consumed: i64,

    /// Currency code (default USD).
    pub currency: String,

    /// When the wallet was created.
    pub created_at: DateTime<Utc>,

    /// When the wallet was last updated.
    pub updated_at: DateTime<Utc>,

    /// Whether the wallet is active.
    pub active: bool,
}

impl Wallet {
    /// Create a new wallet with zero balance.
    pub fn new(organization_id: OrganizationId) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            organization_id,
            balance_cents: 0,
            credits_granted: 0,
            credits_consumed: 0,
            currency: "USD".to_string(),
            created_at: now,
            updated_at: now,
            active: true,
        }
    }

    /// Check if wallet has sufficient balance.
    pub fn has_sufficient_balance(&self, amount_cents: i64) -> bool {
        self.active && self.balance_cents >= amount_cents
    }

    /// Get current balance as Money.
    pub fn balance(&self) -> Money {
        Money::usd(self.balance_cents)
    }

    /// Grant credits to the wallet.
    pub fn grant_credits(&mut self, amount_cents: i64) -> CretoResult<()> {
        if amount_cents <= 0 {
            return Err(CretoError::InvalidUsageEvent(
                "Credit amount must be positive".to_string(),
            ));
        }

        self.balance_cents += amount_cents;
        self.credits_granted += amount_cents;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Consume credits from the wallet.
    pub fn consume_credits(&mut self, amount_cents: i64) -> CretoResult<()> {
        if amount_cents <= 0 {
            return Err(CretoError::InvalidUsageEvent(
                "Credit amount must be positive".to_string(),
            ));
        }

        if !self.has_sufficient_balance(amount_cents) {
            return Err(CretoError::QuotaExceeded {
                resource: "credits".to_string(),
                used: (self.credits_consumed + amount_cents) as u64,
                limit: self.credits_granted as u64,
            });
        }

        self.balance_cents -= amount_cents;
        self.credits_consumed += amount_cents;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Deactivate the wallet.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.updated_at = Utc::now();
    }
}

/// A credit transaction record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditTransaction {
    /// Unique transaction ID.
    pub id: Uuid,

    /// Wallet this transaction belongs to.
    pub wallet_id: Uuid,

    /// Organization ID.
    pub organization_id: OrganizationId,

    /// Transaction type.
    pub transaction_type: CreditTransactionType,

    /// Amount in cents (positive for grants, negative for consumption).
    pub amount_cents: i64,

    /// Balance after transaction.
    pub balance_after: i64,

    /// Optional reference (e.g., invoice ID, usage event ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,

    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// When the transaction occurred.
    pub created_at: DateTime<Utc>,
}

impl CreditTransaction {
    /// Create a new credit transaction.
    pub fn new(
        wallet_id: Uuid,
        organization_id: OrganizationId,
        transaction_type: CreditTransactionType,
        amount_cents: i64,
        balance_after: i64,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            wallet_id,
            organization_id,
            transaction_type,
            amount_cents,
            balance_after,
            reference_id: None,
            description: None,
            created_at: Utc::now(),
        }
    }

    /// Set the reference ID.
    pub fn with_reference(mut self, reference_id: impl Into<String>) -> Self {
        self.reference_id = Some(reference_id.into());
        self
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Type of credit transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditTransactionType {
    /// Credits granted (top-up, promotional, etc.).
    Grant,
    /// Credits consumed for usage.
    Consumption,
    /// Credits expired.
    Expiration,
    /// Credits refunded.
    Refund,
    /// Manual adjustment.
    Adjustment,
}

/// Manager for credit wallets and transactions.
pub struct CreditManager {
    /// In-memory wallet storage (for development).
    wallets: Arc<RwLock<HashMap<OrganizationId, Wallet>>>,
    /// Transaction history.
    transactions: Arc<RwLock<Vec<CreditTransaction>>>,
}

impl CreditManager {
    /// Create a new credit manager.
    pub fn new() -> Self {
        Self {
            wallets: Arc::new(RwLock::new(HashMap::new())),
            transactions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get or create a wallet for an organization.
    pub fn get_or_create_wallet(&self, organization_id: OrganizationId) -> Wallet {
        let mut wallets = self.wallets.write().unwrap();

        wallets
            .entry(organization_id)
            .or_insert_with(|| Wallet::new(organization_id))
            .clone()
    }

    /// Get wallet by organization ID.
    pub fn get_wallet(&self, organization_id: &OrganizationId) -> Option<Wallet> {
        let wallets = self.wallets.read().unwrap();
        wallets.get(organization_id).cloned()
    }

    /// Grant credits to an organization.
    pub fn grant_credits(
        &self,
        organization_id: OrganizationId,
        amount_cents: i64,
        description: Option<&str>,
    ) -> CretoResult<CreditTransaction> {
        let mut wallets = self.wallets.write().unwrap();

        let wallet = wallets
            .entry(organization_id)
            .or_insert_with(|| Wallet::new(organization_id));

        wallet.grant_credits(amount_cents)?;

        let transaction = CreditTransaction::new(
            wallet.id,
            organization_id,
            CreditTransactionType::Grant,
            amount_cents,
            wallet.balance_cents,
        );

        let transaction = if let Some(desc) = description {
            transaction.with_description(desc)
        } else {
            transaction
        };

        // Record transaction
        self.transactions.write().unwrap().push(transaction.clone());

        Ok(transaction)
    }

    /// Consume credits from an organization's wallet.
    pub fn consume_credits(
        &self,
        organization_id: &OrganizationId,
        amount_cents: i64,
        reference_id: Option<&str>,
    ) -> CretoResult<CreditTransaction> {
        let mut wallets = self.wallets.write().unwrap();

        let wallet = wallets.get_mut(organization_id).ok_or_else(|| {
            CretoError::BillingPeriodNotFound(format!(
                "Wallet not found for organization: {}",
                organization_id
            ))
        })?;

        wallet.consume_credits(amount_cents)?;

        let mut transaction = CreditTransaction::new(
            wallet.id,
            *organization_id,
            CreditTransactionType::Consumption,
            -amount_cents, // Negative for consumption
            wallet.balance_cents,
        );

        if let Some(ref_id) = reference_id {
            transaction = transaction.with_reference(ref_id);
        }

        // Record transaction
        self.transactions.write().unwrap().push(transaction.clone());

        Ok(transaction)
    }

    /// Check if organization has sufficient credits.
    pub fn has_sufficient_credits(
        &self,
        organization_id: &OrganizationId,
        amount_cents: i64,
    ) -> bool {
        let wallets = self.wallets.read().unwrap();

        wallets
            .get(organization_id)
            .map(|w| w.has_sufficient_balance(amount_cents))
            .unwrap_or(false)
    }

    /// Get balance for an organization.
    pub fn get_balance(&self, organization_id: &OrganizationId) -> i64 {
        let wallets = self.wallets.read().unwrap();

        wallets
            .get(organization_id)
            .map(|w| w.balance_cents)
            .unwrap_or(0)
    }

    /// Get transaction history for an organization.
    pub fn get_transactions(
        &self,
        organization_id: &OrganizationId,
        limit: Option<usize>,
    ) -> Vec<CreditTransaction> {
        let transactions = self.transactions.read().unwrap();

        let mut org_transactions: Vec<_> = transactions
            .iter()
            .filter(|t| &t.organization_id == organization_id)
            .cloned()
            .collect();

        // Sort by date descending
        org_transactions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if let Some(limit) = limit {
            org_transactions.truncate(limit);
        }

        org_transactions
    }

    /// Apply credits to reduce invoice total, return remaining amount to invoice.
    pub fn apply_credits_to_invoice(
        &self,
        organization_id: &OrganizationId,
        invoice_total_cents: i64,
        invoice_id: &str,
    ) -> CretoResult<CreditApplication> {
        let balance = self.get_balance(organization_id);

        if balance <= 0 {
            return Ok(CreditApplication {
                credits_applied: 0,
                remaining_to_invoice: invoice_total_cents,
            });
        }

        let credits_to_apply = balance.min(invoice_total_cents);
        let remaining = invoice_total_cents - credits_to_apply;

        if credits_to_apply > 0 {
            self.consume_credits(organization_id, credits_to_apply, Some(invoice_id))?;
        }

        Ok(CreditApplication {
            credits_applied: credits_to_apply,
            remaining_to_invoice: remaining,
        })
    }
}

impl Default for CreditManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of applying credits to an invoice.
#[derive(Debug, Clone)]
pub struct CreditApplication {
    /// Amount of credits applied.
    pub credits_applied: i64,
    /// Remaining amount to be invoiced.
    pub remaining_to_invoice: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let org_id = OrganizationId::new();
        let wallet = Wallet::new(org_id.clone());

        assert_eq!(wallet.organization_id, org_id);
        assert_eq!(wallet.balance_cents, 0);
        assert!(wallet.active);
    }

    #[test]
    fn test_grant_and_consume_credits() {
        let mut wallet = Wallet::new(OrganizationId::new());

        // Grant credits
        wallet.grant_credits(10000).unwrap(); // $100
        assert_eq!(wallet.balance_cents, 10000);
        assert_eq!(wallet.credits_granted, 10000);

        // Consume credits
        wallet.consume_credits(3000).unwrap(); // $30
        assert_eq!(wallet.balance_cents, 7000);
        assert_eq!(wallet.credits_consumed, 3000);

        // Try to consume more than balance
        let result = wallet.consume_credits(10000);
        assert!(result.is_err());
    }

    #[test]
    fn test_credit_manager_workflow() {
        let manager = CreditManager::new();
        let org_id = OrganizationId::new();

        // Grant credits
        let grant = manager
            .grant_credits(org_id.clone(), 50000, Some("Welcome bonus"))
            .unwrap();
        assert_eq!(grant.amount_cents, 50000);
        assert_eq!(grant.balance_after, 50000);

        // Check balance
        assert_eq!(manager.get_balance(&org_id), 50000);
        assert!(manager.has_sufficient_credits(&org_id, 30000));

        // Consume credits
        let consume = manager
            .consume_credits(&org_id, 20000, Some("INV-001"))
            .unwrap();
        assert_eq!(consume.amount_cents, -20000);
        assert_eq!(consume.balance_after, 30000);

        // Check updated balance
        assert_eq!(manager.get_balance(&org_id), 30000);
    }

    #[test]
    fn test_apply_credits_to_invoice() {
        let manager = CreditManager::new();
        let org_id = OrganizationId::new();

        // Grant $50 in credits
        manager.grant_credits(org_id.clone(), 5000, None).unwrap();

        // Apply to $80 invoice
        let application = manager
            .apply_credits_to_invoice(&org_id, 8000, "INV-001")
            .unwrap();

        assert_eq!(application.credits_applied, 5000);
        assert_eq!(application.remaining_to_invoice, 3000);
        assert_eq!(manager.get_balance(&org_id), 0);
    }

    #[test]
    fn test_apply_credits_exceeds_invoice() {
        let manager = CreditManager::new();
        let org_id = OrganizationId::new();

        // Grant $100 in credits
        manager.grant_credits(org_id.clone(), 10000, None).unwrap();

        // Apply to $30 invoice
        let application = manager
            .apply_credits_to_invoice(&org_id, 3000, "INV-002")
            .unwrap();

        assert_eq!(application.credits_applied, 3000);
        assert_eq!(application.remaining_to_invoice, 0);
        assert_eq!(manager.get_balance(&org_id), 7000); // $70 remaining
    }

    #[test]
    fn test_transaction_history() {
        let manager = CreditManager::new();
        let org_id = OrganizationId::new();

        // Multiple transactions
        manager.grant_credits(org_id.clone(), 10000, None).unwrap();
        manager.consume_credits(&org_id, 2000, None).unwrap();
        manager.grant_credits(org_id.clone(), 5000, None).unwrap();
        manager.consume_credits(&org_id, 1000, None).unwrap();

        let history = manager.get_transactions(&org_id, Some(10));
        assert_eq!(history.len(), 4);

        // Most recent first
        assert_eq!(
            history[0].transaction_type,
            CreditTransactionType::Consumption
        );
        assert_eq!(history[1].transaction_type, CreditTransactionType::Grant);
    }
}
