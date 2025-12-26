-- Runtime Schema for Creto Enablement Layer
-- Sandboxed execution environment following Agent Sandbox patterns

-- Sandboxes
CREATE TABLE IF NOT EXISTS sandboxes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    agent_id UUID NOT NULL,
    runtime VARCHAR(50) NOT NULL,  -- python3.11, node20, deno, etc.
    state VARCHAR(20) NOT NULL DEFAULT 'creating',  -- creating, ready, running, paused, stopped, failed, terminated
    config JSONB NOT NULL,  -- SandboxConfig serialized
    runtime_handle VARCHAR(255),  -- Backend-specific handle
    resource_limits JSONB NOT NULL,
    network_policy VARCHAR(20) NOT NULL DEFAULT 'restricted',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    terminated_at TIMESTAMPTZ
);

CREATE INDEX idx_sandboxes_org ON sandboxes(organization_id);
CREATE INDEX idx_sandboxes_agent ON sandboxes(agent_id);
CREATE INDEX idx_sandboxes_state ON sandboxes(state);
CREATE INDEX idx_sandboxes_runtime ON sandboxes(runtime, state);

-- Execution requests
CREATE TABLE IF NOT EXISTS execution_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sandbox_id UUID NOT NULL REFERENCES sandboxes(id),
    code TEXT NOT NULL,
    entry_point VARCHAR(255),
    input JSONB NOT NULL DEFAULT 'null',
    timeout_seconds INTEGER NOT NULL DEFAULT 300,
    status VARCHAR(20) NOT NULL DEFAULT 'queued',  -- queued, running, completed, failed, timed_out, cancelled
    queued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    duration_ms BIGINT
);

CREATE INDEX idx_execution_requests_sandbox ON execution_requests(sandbox_id);
CREATE INDEX idx_execution_requests_status ON execution_requests(status);

-- Execution results
CREATE TABLE IF NOT EXISTS execution_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL UNIQUE REFERENCES execution_requests(id) ON DELETE CASCADE,
    output JSONB NOT NULL DEFAULT 'null',
    stdout TEXT,
    stderr TEXT,
    error_code VARCHAR(50),
    error_message TEXT,
    error_stack TEXT,
    error_line INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Resource usage snapshots
CREATE TABLE IF NOT EXISTS resource_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sandbox_id UUID NOT NULL REFERENCES sandboxes(id) ON DELETE CASCADE,
    memory_bytes BIGINT NOT NULL,
    peak_memory_bytes BIGINT NOT NULL,
    cpu_time_ms BIGINT NOT NULL,
    wall_time_ms BIGINT NOT NULL,
    disk_bytes BIGINT NOT NULL,
    process_count INTEGER NOT NULL,
    open_file_count INTEGER NOT NULL,
    network_bytes_sent BIGINT NOT NULL DEFAULT 0,
    network_bytes_received BIGINT NOT NULL DEFAULT 0,
    connection_count INTEGER NOT NULL DEFAULT 0,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_resource_usage_sandbox ON resource_usage(sandbox_id, recorded_at DESC);

-- Warm pool configuration
CREATE TABLE IF NOT EXISTS warm_pool_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    runtime VARCHAR(50) NOT NULL,
    min_warm INTEGER NOT NULL DEFAULT 2,
    max_warm INTEGER NOT NULL DEFAULT 10,
    idle_timeout_seconds INTEGER NOT NULL DEFAULT 300,
    resource_limits JSONB NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, runtime)
);

-- Secret mounts (references to secrets, not the secrets themselves)
CREATE TABLE IF NOT EXISTS secret_mounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sandbox_id UUID NOT NULL REFERENCES sandboxes(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    mount_type VARCHAR(20) NOT NULL,  -- environment_variable, file, directory
    mount_path VARCHAR(255),  -- For file/directory mounts
    source_type VARCHAR(20) NOT NULL,  -- vault, organization_secret, agent_credential, inline
    source_reference VARCHAR(255) NOT NULL,  -- Path or name in source
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(sandbox_id, name)
);

CREATE INDEX idx_secret_mounts_sandbox ON secret_mounts(sandbox_id);

-- Network egress rules
CREATE TABLE IF NOT EXISTS network_egress_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    sandbox_id UUID,  -- NULL means org-wide rule
    rule_type VARCHAR(10) NOT NULL,  -- allow, deny
    host_pattern VARCHAR(255) NOT NULL,  -- Glob pattern or exact host
    port INTEGER,  -- NULL means all ports
    protocol VARCHAR(10),  -- tcp, udp, NULL means both
    priority INTEGER NOT NULL DEFAULT 0,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_network_egress_org ON network_egress_rules(organization_id);
CREATE INDEX idx_network_egress_sandbox ON network_egress_rules(sandbox_id) WHERE sandbox_id IS NOT NULL;
