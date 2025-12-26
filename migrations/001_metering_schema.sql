-- Metering Schema for Creto Enablement Layer
-- Follows Lago patterns for usage-based billing

-- Usage events table (append-only for audit trail)
CREATE TABLE IF NOT EXISTS usage_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id VARCHAR(255) NOT NULL UNIQUE,
    organization_id UUID NOT NULL,
    agent_id UUID NOT NULL,
    external_subscription_id VARCHAR(255),
    event_type VARCHAR(50) NOT NULL,
    code VARCHAR(100) NOT NULL,
    quantity BIGINT NOT NULL DEFAULT 1,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    properties JSONB NOT NULL DEFAULT '{}',
    delegation_depth SMALLINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for efficient queries by organization and time
CREATE INDEX idx_usage_events_org_time ON usage_events(organization_id, timestamp DESC);
CREATE INDEX idx_usage_events_agent ON usage_events(agent_id, timestamp DESC);
CREATE INDEX idx_usage_events_code ON usage_events(code, timestamp DESC);

-- Quotas table
CREATE TABLE IF NOT EXISTS quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    agent_id UUID,  -- NULL means org-wide quota
    resource VARCHAR(100) NOT NULL,
    limit_value BIGINT NOT NULL,
    period VARCHAR(20) NOT NULL,  -- hourly, daily, weekly, monthly, lifetime
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    current_usage BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, agent_id, resource, period_start)
);

CREATE INDEX idx_quotas_org ON quotas(organization_id);
CREATE INDEX idx_quotas_agent ON quotas(agent_id) WHERE agent_id IS NOT NULL;

-- Billable metrics configuration
CREATE TABLE IF NOT EXISTS billable_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    code VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    aggregation_type VARCHAR(20) NOT NULL,  -- count, sum, max, unique_count, latest
    field_name VARCHAR(100),  -- For sum/max aggregations
    recurring BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, code)
);

-- Pricing models
CREATE TABLE IF NOT EXISTS pricing_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    metric_id UUID NOT NULL REFERENCES billable_metrics(id),
    name VARCHAR(255) NOT NULL,
    strategy VARCHAR(20) NOT NULL,  -- flat_fee, per_unit, graduated, volume, package
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    amount_cents BIGINT,  -- For flat_fee
    unit_amount_cents BIGINT,  -- For per_unit
    package_size BIGINT,  -- For package pricing
    tiers JSONB,  -- For tiered pricing
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pricing_models_metric ON pricing_models(metric_id);

-- Invoices
CREATE TABLE IF NOT EXISTS invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    invoice_number VARCHAR(50) NOT NULL UNIQUE,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',  -- draft, issued, paid, failed, voided
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    subtotal_cents BIGINT NOT NULL DEFAULT 0,
    tax_cents BIGINT NOT NULL DEFAULT 0,
    discount_cents BIGINT NOT NULL DEFAULT 0,
    total_cents BIGINT NOT NULL DEFAULT 0,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    issued_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ,
    due_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_invoices_org ON invoices(organization_id);
CREATE INDEX idx_invoices_status ON invoices(status);

-- Invoice line items
CREATE TABLE IF NOT EXISTS invoice_line_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    metric_id UUID REFERENCES billable_metrics(id),
    description VARCHAR(255) NOT NULL,
    quantity BIGINT NOT NULL,
    unit_amount_cents BIGINT NOT NULL,
    amount_cents BIGINT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_line_items_invoice ON invoice_line_items(invoice_id);
