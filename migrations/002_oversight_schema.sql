-- Oversight Schema for Creto Enablement Layer
-- Human-in-the-loop approval workflows following HumanLayer patterns

-- Oversight requests
CREATE TABLE IF NOT EXISTS oversight_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    agent_id UUID NOT NULL,
    action_type VARCHAR(50) NOT NULL,  -- transaction, data_access, external_api, code_execution, communication
    action_data JSONB NOT NULL,
    description TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, in_review, approved, rejected, escalated, timed_out, cancelled
    priority VARCHAR(10) NOT NULL DEFAULT 'medium',  -- low, medium, high, critical
    context JSONB NOT NULL DEFAULT '{}',
    timeout_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_oversight_requests_org ON oversight_requests(organization_id);
CREATE INDEX idx_oversight_requests_agent ON oversight_requests(agent_id);
CREATE INDEX idx_oversight_requests_status ON oversight_requests(status);
CREATE INDEX idx_oversight_requests_timeout ON oversight_requests(timeout_at) WHERE status = 'pending';

-- Request reviewers (who can approve/reject)
CREATE TABLE IF NOT EXISTS request_reviewers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL REFERENCES oversight_requests(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    role VARCHAR(50),  -- Optional role-based assignment
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(request_id, user_id)
);

CREATE INDEX idx_request_reviewers_user ON request_reviewers(user_id);

-- Approvals/decisions
CREATE TABLE IF NOT EXISTS approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL REFERENCES oversight_requests(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL,
    decision VARCHAR(20) NOT NULL,  -- approve, reject, abstain, request_info, escalate
    reason TEXT,
    weight INTEGER NOT NULL DEFAULT 1,
    decided_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(request_id, reviewer_id)
);

CREATE INDEX idx_approvals_request ON approvals(request_id);
CREATE INDEX idx_approvals_reviewer ON approvals(reviewer_id);

-- State transitions (audit trail)
CREATE TABLE IF NOT EXISTS state_transitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL REFERENCES oversight_requests(id) ON DELETE CASCADE,
    from_status VARCHAR(20) NOT NULL,
    to_status VARCHAR(20) NOT NULL,
    actor_type VARCHAR(20) NOT NULL,  -- system, user, policy
    actor_id UUID,  -- NULL for system
    reason TEXT,
    transitioned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_state_transitions_request ON state_transitions(request_id, transitioned_at);

-- Quorum configurations per organization
CREATE TABLE IF NOT EXISTS quorum_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    required_approvals INTEGER NOT NULL DEFAULT 1,
    required_weight INTEGER,
    any_rejection_rejects BOOLEAN NOT NULL DEFAULT false,
    require_unanimous BOOLEAN NOT NULL DEFAULT false,
    action_type VARCHAR(50),  -- NULL means default for org
    min_amount_cents BIGINT,  -- For tiered quorums
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, name)
);

-- Notification channels
CREATE TABLE IF NOT EXISTS notification_channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    channel_type VARCHAR(20) NOT NULL,  -- slack, email, teams, sms, webhook, in_app
    name VARCHAR(100) NOT NULL,
    config JSONB NOT NULL,  -- Channel-specific configuration
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notification_channels_org ON notification_channels(organization_id);

-- Notification history
CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL REFERENCES oversight_requests(id) ON DELETE CASCADE,
    channel_id UUID NOT NULL REFERENCES notification_channels(id),
    notification_type VARCHAR(20) NOT NULL,  -- request, reminder, decision
    message_id VARCHAR(255),  -- External message ID for tracking
    status VARCHAR(20) NOT NULL,  -- sent, delivered, failed
    error_message TEXT,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notifications_request ON notifications(request_id);

-- Escalation rules
CREATE TABLE IF NOT EXISTS escalation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    trigger_after_minutes INTEGER NOT NULL,
    escalate_to_role VARCHAR(50),
    escalate_to_user_id UUID,
    notify_channel_id UUID REFERENCES notification_channels(id),
    action_type VARCHAR(50),  -- NULL means all action types
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_escalation_rules_org ON escalation_rules(organization_id);
