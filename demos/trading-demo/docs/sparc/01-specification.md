# SPARC Specification: Autonomous Portfolio Manager

**Version:** 1.0.0
**Date:** 2025-12-26
**Status:** Draft
**Author:** SPARC Specification Agent

---

## 1. Executive Summary

### 1.1 Purpose
The Autonomous Portfolio Manager is an investor demonstration platform showcasing creto-metering and creto-oversight capabilities for AI-driven trading agents. The system demonstrates quota enforcement, usage tracking, and human-in-the-loop approval workflows for high-value financial decisions.

### 1.2 Scope
This system provides:
- **Agent Provisioning Dashboard**: Create and configure autonomous trading agents with budget controls
- **Real-Time Trading Dashboard**: Monitor portfolio performance, API usage, and budget consumption
- **Oversight Approval Workflow**: Human approval system for high-value trades via Slack integration
- **Metering Infrastructure**: Track and enforce API quotas, budget limits, and usage patterns
- **Investor Analytics**: Demonstrate AI governance and cost control for stakeholder presentations

### 1.3 Key Stakeholders
- **Investors**: Primary audience for demo presentations
- **Product Team**: Demonstration owners and presenters
- **Development Team**: Implementation and maintenance
- **Compliance Team**: Oversight and governance requirements

### 1.4 Success Criteria
- Demo runs flawlessly in investor presentations (99.9% uptime during demos)
- Clear visualization of creto-metering value proposition (real-time budget tracking)
- Compelling oversight workflow demonstration (< 30 second approval cycle)
- Scalable to 100+ concurrent demo instances for distributed presentations

---

## 2. Functional Requirements

### 2.1 Agent Lifecycle Management

#### FR-001: Agent Creation
**Priority:** HIGH
**Description:** Users shall create autonomous trading agents with configurable parameters.

**Acceptance Criteria:**
- [ ] User can access agent creation form from dashboard
- [ ] Required fields: agent name, trading strategy, initial budget, risk tolerance
- [ ] Optional fields: asset preferences, trading hours, rebalancing frequency
- [ ] System validates budget ≤ $100,000 per agent
- [ ] System assigns unique agent ID upon creation
- [ ] Agent appears in dashboard within 2 seconds of creation
- [ ] Audit log records agent creation with timestamp and user ID

**Edge Cases:**
- Duplicate agent names → Append UUID suffix
- Invalid budget values → Show validation error with range
- Network timeout during creation → Implement retry with idempotency key

#### FR-002: Agent Configuration
**Priority:** HIGH
**Description:** Users shall modify agent parameters after creation.

**Acceptance Criteria:**
- [ ] User can edit agent name, budget cap, risk tolerance
- [ ] Changes require confirmation dialog for budgets > $50K
- [ ] System validates configuration before saving
- [ ] Configuration changes take effect within 5 seconds
- [ ] Historical configurations are versioned and retrievable
- [ ] Audit trail shows who changed what and when

**Constraints:**
- Cannot reduce budget below current portfolio value
- Risk tolerance must be within [1-10] scale
- Active agents cannot have trading strategy changed (must pause first)

#### FR-003: Agent Monitoring
**Priority:** HIGH
**Description:** Users shall monitor agent status, performance, and resource consumption in real-time.

**Acceptance Criteria:**
- [ ] Dashboard displays agent status (active, paused, stopped, error)
- [ ] Portfolio value updates every 5 seconds via WebSocket
- [ ] Trade history shows last 50 transactions with pagination
- [ ] Metering panel shows API calls consumed vs. quota
- [ ] Budget burn-down chart displays daily spending rate
- [ ] Performance metrics: ROI, Sharpe ratio, max drawdown
- [ ] Alert notifications for quota thresholds (80%, 90%, 100%)

#### FR-004: Agent Termination
**Priority:** MEDIUM
**Description:** Users shall safely terminate agents and liquidate positions.

**Acceptance Criteria:**
- [ ] User can pause agent (stop trading, maintain positions)
- [ ] User can stop agent (liquidate positions, preserve data)
- [ ] User can delete agent (full cleanup after 30-day retention)
- [ ] Liquidation creates market orders for all positions
- [ ] Final portfolio report generated within 10 seconds
- [ ] Confirmation dialog requires typing agent name for deletion
- [ ] Terminated agents remain in "archived" view for 90 days

### 2.2 Portfolio Management

#### FR-005: Portfolio Visualization
**Priority:** HIGH
**Description:** Users shall view real-time portfolio composition and performance.

**Acceptance Criteria:**
- [ ] Pie chart shows asset allocation by percentage
- [ ] Line chart displays portfolio value over time (1H, 1D, 1W, 1M, ALL)
- [ ] Table lists positions: symbol, quantity, avg cost, current price, P&L
- [ ] Total portfolio value displayed prominently with % change
- [ ] Color coding: green for profit, red for loss, yellow for warnings
- [ ] Supports up to 100 positions per agent
- [ ] Charts render within 500ms using cached data

#### FR-006: Trade Execution
**Priority:** HIGH
**Description:** Agents shall execute trades automatically based on strategy, with metering checks.

**Acceptance Criteria:**
- [ ] Agent analyzes market data every 60 seconds
- [ ] Trade decisions logged with reasoning (for demo transparency)
- [ ] Metering check before each trade API call
- [ ] Trade execution within 200ms of decision (market orders)
- [ ] Failed trades logged with error details and retried (max 3 attempts)
- [ ] Trade confirmations stored with timestamp, price, fees
- [ ] Portfolio automatically rebalances post-trade

#### FR-007: Manual Trade Override
**Priority:** MEDIUM
**Description:** Users shall manually execute trades on behalf of agents for demo purposes.

**Acceptance Criteria:**
- [ ] Dashboard provides "Manual Trade" button per agent
- [ ] Modal form: symbol, action (buy/sell), quantity, order type
- [ ] Preview shows estimated cost, fees, new portfolio state
- [ ] Requires two-factor authentication for trades > $10K
- [ ] Manual trades bypass oversight workflow (demo control)
- [ ] Clearly marked as "manual" in trade history

### 2.3 Metering Integration (creto-metering)

#### FR-008: Quota Enforcement
**Priority:** HIGH
**Description:** System shall enforce API quota limits per agent with real-time tracking.

**Acceptance Criteria:**
- [ ] Each agent assigned quota: 10,000 API calls/month, $10K budget/month
- [ ] Pre-call quota check blocks requests when quota exhausted
- [ ] Quota resets on 1st of each month at 00:00 UTC
- [ ] Quota buffer (5%) allows burst traffic before hard block
- [ ] Rate limiting: max 100 API calls/minute per agent
- [ ] HTTP 429 response when rate limit exceeded with retry-after header
- [ ] Admin override capability for quota increases (demo flexibility)

#### FR-009: Usage Tracking
**Priority:** HIGH
**Description:** System shall track and display granular usage metrics per agent.

**Acceptance Criteria:**
- [ ] Dashboard shows API calls consumed (today, this week, this month)
- [ ] Budget burn-down chart: cumulative spend vs. time
- [ ] Detailed usage log: timestamp, endpoint, cost, response time
- [ ] Exportable usage reports (CSV, JSON) for billing demos
- [ ] Real-time usage updates via WebSocket (< 1 second latency)
- [ ] Historical usage data retained for 12 months
- [ ] Aggregated usage statistics across all agents

#### FR-010: Usage Alerts
**Priority:** MEDIUM
**Description:** System shall notify users when quota thresholds are reached.

**Acceptance Criteria:**
- [ ] Email alert at 80% quota consumption
- [ ] Dashboard banner at 90% quota consumption
- [ ] Slack notification at 100% quota exhausted
- [ ] Alert includes: current usage, quota limit, projected exhaustion date
- [ ] Users can configure alert thresholds (50%, 75%, 90%)
- [ ] Alerts deduplicated (max 1 alert per threshold per day)
- [ ] Alert history viewable in notification center

#### FR-011: Metering Analytics
**Priority:** MEDIUM
**Description:** System shall provide analytics dashboard for usage patterns and optimization.

**Acceptance Criteria:**
- [ ] Heatmap: API calls by hour of day and day of week
- [ ] Cost breakdown: trading API (70%), market data (20%), other (10%)
- [ ] Efficiency metrics: cost per trade, API calls per $1 traded
- [ ] Comparison view: agent vs. agent, current vs. previous month
- [ ] Recommendations for quota optimization based on usage patterns
- [ ] Exportable analytics reports for investor presentations

### 2.4 Oversight Integration (creto-oversight)

#### FR-012: Approval Trigger Conditions
**Priority:** HIGH
**Description:** System shall automatically trigger human approval for high-value or high-risk trades.

**Acceptance Criteria:**
- [ ] Trigger conditions configurable per agent (default: trade > $50K)
- [ ] Additional triggers: unusual volatility, concentrated positions (>30% portfolio)
- [ ] Trigger evaluation within 50ms of trade decision
- [ ] Triggered trades enter "pending approval" state
- [ ] Agent pauses further trading until approval/rejection
- [ ] Trigger logic supports AND/OR conditions for complex rules
- [ ] Dry-run mode simulates triggers without pausing (for testing)

#### FR-013: Approval Request Creation
**Priority:** HIGH
**Description:** System shall generate detailed approval requests with context.

**Acceptance Criteria:**
- [ ] Request includes: agent name, trade details, reasoning, portfolio impact
- [ ] Risk assessment: estimated P&L, position concentration, correlation
- [ ] Visual preview: before/after portfolio pie charts
- [ ] Automatically routed to configured approvers (default: Slack channel)
- [ ] Request ID for tracking and auditing
- [ ] Expiration timer: 10 minutes before auto-rejection (safety)
- [ ] Historical context: similar past trades and outcomes

#### FR-014: Slack Approval Workflow
**Priority:** HIGH
**Description:** Approvers shall review and approve/reject trades via Slack.

**Acceptance Criteria:**
- [ ] Slack message with embedded approval form (buttons: Approve, Reject, More Info)
- [ ] Message includes trade summary, risk score, portfolio impact visualization
- [ ] "More Info" button links to detailed dashboard view
- [ ] Approval/rejection recorded within 2 seconds of button click
- [ ] Slack thread shows approval decision with timestamp and approver name
- [ ] Notifications to other configured channels (email, SMS) as fallback
- [ ] Approval requires authentication (Slack OAuth)

#### FR-015: Approval Decision Processing
**Priority:** HIGH
**Description:** System shall execute or cancel trades based on approval decisions.

**Acceptance Criteria:**
- [ ] Approved trades execute within 5 seconds of approval
- [ ] Rejected trades cancel and log rejection reason
- [ ] Agent resumes trading after decision processed
- [ ] Trade history shows approval status and approver identity
- [ ] Performance metrics track approval latency (target: < 30 seconds)
- [ ] Escalation path if no decision within 10 minutes (default: reject)
- [ ] A/B testing capability: compare agent performance with/without oversight

#### FR-016: Oversight Analytics
**Priority:** MEDIUM
**Description:** System shall provide analytics on approval patterns and bottlenecks.

**Acceptance Criteria:**
- [ ] Dashboard shows: approval rate, avg approval time, approver response rate
- [ ] Trend analysis: triggers per day, approval rate over time
- [ ] Bottleneck identification: slow approvers, peak trigger times
- [ ] Simulated impact: "what if we auto-approved < $75K trades?"
- [ ] Exportable reports for compliance and investor presentations
- [ ] Real-time approval queue status for demo monitoring

### 2.5 User Interface & Experience

#### FR-017: Dashboard Navigation
**Priority:** HIGH
**Description:** Users shall navigate intuitively between dashboard sections.

**Acceptance Criteria:**
- [ ] Top navigation: Agents, Portfolio, Usage, Oversight, Settings
- [ ] Breadcrumb trail shows current location
- [ ] Global search: find agents by name, ID, or strategy
- [ ] Keyboard shortcuts for power users (e.g., Cmd+K for search)
- [ ] Responsive design: desktop (1920x1080), tablet (1024x768), mobile (375x667)
- [ ] Dark mode toggle with persisted preference
- [ ] Accessibility: ARIA labels, keyboard navigation, screen reader support

#### FR-018: Real-Time Updates
**Priority:** HIGH
**Description:** Dashboard shall reflect system state changes in real-time.

**Acceptance Criteria:**
- [ ] WebSocket connection for live updates (auto-reconnect on disconnect)
- [ ] Portfolio value updates every 5 seconds
- [ ] Trade notifications appear within 1 second of execution
- [ ] Approval requests toast notification with sound (configurable)
- [ ] Connection status indicator (green dot = connected)
- [ ] Offline mode: cache last known state, sync on reconnect
- [ ] Update batching: group updates within 200ms window to reduce UI thrashing

#### FR-019: Data Export & Reporting
**Priority:** MEDIUM
**Description:** Users shall export data for offline analysis and presentations.

**Acceptance Criteria:**
- [ ] Export formats: CSV, JSON, PDF
- [ ] Exportable data: trade history, usage logs, performance reports, audit trails
- [ ] Custom date range selection for exports
- [ ] PDF reports include charts, tables, and executive summary
- [ ] Scheduled exports: daily/weekly/monthly email delivery
- [ ] Export queue with progress indicator for large datasets
- [ ] Rate limiting: max 10 exports per hour per user

#### FR-020: Admin Panel
**Priority:** MEDIUM
**Description:** Administrators shall manage system configuration and demo settings.

**Acceptance Criteria:**
- [ ] User management: create/edit/delete users, assign roles
- [ ] System settings: quota defaults, approval thresholds, alert configurations
- [ ] Demo mode toggle: enables synthetic data generation for presentations
- [ ] Feature flags: enable/disable oversight, metering, trading strategies
- [ ] Audit log viewer with filtering and search
- [ ] Health check dashboard: API status, database connections, WebSocket health
- [ ] One-click demo reset: clear all agents and data for clean demo start

---

## 3. Non-Functional Requirements

### 3.1 Performance

#### NFR-001: API Response Time
**Category:** Performance
**Description:** API endpoints shall respond within acceptable latency thresholds.

**Requirements:**
- 95th percentile latency < 200ms for all GET requests
- 95th percentile latency < 500ms for POST/PUT requests (trades)
- 99th percentile latency < 1000ms for all requests
- Database queries optimized with indexes (execution time < 50ms)
- CDN caching for static assets (cache hit rate > 95%)

**Measurement:**
- Application Performance Monitoring (APM) with Datadog/New Relic
- Synthetic monitoring: ping endpoints every 30 seconds
- Load testing: maintain latency under 1000 concurrent users

#### NFR-002: UI Responsiveness
**Category:** Performance
**Description:** User interface shall update quickly without blocking interactions.

**Requirements:**
- Initial page load < 2 seconds (Lighthouse performance score > 90)
- Time to Interactive (TTI) < 3 seconds
- Chart rendering < 500ms for datasets up to 10,000 points
- WebSocket message processing < 100ms per event
- Smooth scrolling at 60fps (no jank during scroll)

**Measurement:**
- Lighthouse CI in build pipeline
- Real User Monitoring (RUM) for field data
- Chrome DevTools Performance profiling

#### NFR-003: Scalability
**Category:** Performance
**Description:** System shall handle growth in users, agents, and data volume.

**Requirements:**
- Support 100 concurrent trading agents (10 trades/minute each)
- Support 500 concurrent dashboard users
- Handle 50,000 API requests/minute with auto-scaling
- Database: 10M trades, 1M agents, 100M usage logs (5-year retention)
- Horizontal scaling: add app servers without code changes

**Measurement:**
- Load testing with K6 or Gatling (target: 10,000 RPS)
- Database query performance testing (pgBench)
- Auto-scaling tests: trigger scale events and measure recovery time

### 3.2 Security

#### NFR-004: Authentication & Authorization
**Category:** Security
**Description:** System shall implement secure authentication and role-based access control.

**Requirements:**
- JWT authentication with 1-hour expiry and refresh tokens
- RBAC roles: Admin, Trader, Viewer, Approver
- Password requirements: min 12 chars, uppercase, lowercase, number, symbol
- Multi-factor authentication (MFA) required for Admin and Approver roles
- Session timeout after 30 minutes of inactivity
- Rate limiting: max 5 login attempts per 15 minutes per IP

**Validation:**
- Penetration testing for authentication bypass
- OWASP ZAP scanning for session vulnerabilities
- Compliance audit: SOC 2 Type II requirements

#### NFR-005: Data Encryption
**Category:** Security
**Description:** Sensitive data shall be encrypted in transit and at rest.

**Requirements:**
- TLS 1.3 for all API traffic (disable TLS 1.0/1.1)
- Database encryption at rest (AES-256)
- Encrypted backups with key rotation every 90 days
- Secrets management via HashiCorp Vault or AWS Secrets Manager
- PII redaction in logs (email → e***@***.com)

**Validation:**
- SSL Labs scan: A+ rating
- Database encryption verification with compliance tools
- Secrets scanning in CI/CD (detect hardcoded keys)

#### NFR-006: Audit Logging
**Category:** Security
**Description:** System shall maintain comprehensive audit trails for compliance.

**Requirements:**
- Log all user actions: login, agent creation, trades, approvals, config changes
- Log format: JSON with timestamp, user ID, action, resource, IP, user agent
- Immutable audit logs (append-only, no deletion)
- Centralized logging to SIEM (Splunk/ELK)
- Retention: 7 years for financial transactions, 3 years for user actions
- Tamper detection: cryptographic signatures on log batches

**Validation:**
- Audit log completeness testing (verify all actions logged)
- Compliance review: SOX, GDPR, PCI-DSS requirements
- Log integrity verification with checksums

### 3.3 Reliability

#### NFR-007: Availability
**Category:** Reliability
**Description:** System shall maintain high availability for investor demos.

**Requirements:**
- 99.9% uptime SLA (max 43 minutes downtime/month)
- Zero-downtime deployments with blue-green or canary strategies
- Graceful degradation: show cached data if real-time updates fail
- Health checks every 10 seconds with auto-restart on failure
- Multi-region deployment for disaster recovery (RTO: 15 minutes, RPO: 5 minutes)

**Measurement:**
- Uptime monitoring with Pingdom/StatusCake
- Incident response time tracking (alert → mitigation)
- Chaos engineering tests (Netflix Chaos Monkey)

#### NFR-008: Data Consistency
**Category:** Reliability
**Description:** System shall maintain data integrity across distributed components.

**Requirements:**
- ACID transactions for financial operations (trades, budgets)
- Idempotent API endpoints (retry-safe)
- Exactly-once trade execution (deduplication by idempotency key)
- Database backups every 6 hours with point-in-time recovery
- Data validation: schema enforcement, type checking, range validation

**Validation:**
- Transaction isolation testing (concurrent trade conflicts)
- Backup restoration testing (monthly drills)
- Data integrity checksums (detect corruption)

#### NFR-009: Error Handling
**Category:** Reliability
**Description:** System shall handle errors gracefully and provide actionable feedback.

**Requirements:**
- User-friendly error messages (no stack traces in production)
- Correlation IDs for tracing errors across services
- Automatic retry with exponential backoff for transient failures
- Circuit breaker pattern for external API calls (fail fast after 5 consecutive errors)
- Error budgets: 0.1% error rate threshold triggers incident review

**Validation:**
- Fault injection testing (simulate API failures, database timeouts)
- Error message UX review (non-technical users can understand)
- Error monitoring dashboards (error rate, types, affected endpoints)

### 3.4 Usability

#### NFR-010: Accessibility
**Category:** Usability
**Description:** System shall be accessible to users with disabilities per WCAG 2.1 AA.

**Requirements:**
- Keyboard navigation for all interactive elements (no mouse required)
- Screen reader support with ARIA labels and live regions
- Color contrast ratio ≥ 4.5:1 for text, ≥ 3:1 for UI components
- Resizable text up to 200% without loss of functionality
- No content flashes more than 3 times per second (seizure prevention)
- Closed captions for demo videos

**Validation:**
- Automated testing with axe DevTools
- Manual testing with NVDA/JAWS screen readers
- Accessibility audit by certified WCAG evaluator

#### NFR-011: Internationalization
**Category:** Usability
**Description:** System shall support multiple locales for global investor demos.

**Requirements:**
- UI text externalized to i18n files (English, Spanish, Mandarin)
- Currency formatting per locale (USD, EUR, CNY)
- Date/time formatting per locale (MM/DD/YYYY vs. DD/MM/YYYY)
- Right-to-left (RTL) layout support for Arabic/Hebrew (future)
- Number formatting with locale-specific separators (1,000.00 vs. 1.000,00)

**Validation:**
- Localization testing with native speakers
- Pseudo-localization testing (detect hard-coded strings)
- Cultural appropriateness review (avoid offensive imagery/language)

### 3.5 Maintainability

#### NFR-012: Code Quality
**Category:** Maintainability
**Description:** Codebase shall adhere to best practices for long-term maintainability.

**Requirements:**
- Test coverage ≥ 90% (unit + integration + e2e)
- Linting rules enforced (ESLint, Prettier, TypeScript strict mode)
- Code review approval required for all PRs (min 1 approver)
- Cyclomatic complexity ≤ 10 per function
- No code duplication > 5 lines (DRY principle)
- Documentation: inline comments for complex logic, README per module

**Validation:**
- SonarQube code quality gate (A rating required)
- Code coverage reports in CI/CD (block merge if < 90%)
- Dependency vulnerability scanning (Snyk/Dependabot)

#### NFR-013: Monitoring & Observability
**Category:** Maintainability
**Description:** System shall provide visibility into runtime behavior for debugging.

**Requirements:**
- Structured logging with correlation IDs across services
- Distributed tracing with OpenTelemetry (visualize request flows)
- Metrics: RED (Rate, Errors, Duration) + USE (Utilization, Saturation, Errors)
- Alerting: PagerDuty integration for critical errors
- Dashboards: Grafana for real-time metrics, Kibana for log analysis
- Performance profiling tools enabled in staging (CPU, memory, I/O)

**Validation:**
- Incident response drills (mean time to detection < 5 minutes)
- Dashboard usability testing (engineers can diagnose issues in < 10 minutes)
- Alert tuning (reduce false positives to < 5%)

---

## 4. User Stories

### 4.1 Agent Management

#### US-001: Create Trading Agent
**As a** product manager preparing for an investor demo
**I want** to quickly create a trading agent with a $10K budget
**So that** I can showcase autonomous portfolio management capabilities

**Acceptance Criteria:**
- Given I'm on the dashboard
- When I click "Create Agent" and fill the form (name: "Demo Agent", budget: $10,000, strategy: "Balanced")
- Then the agent appears in my agent list within 2 seconds
- And I see a confirmation notification
- And the agent starts in "paused" state (ready to activate)

**Priority:** HIGH
**Story Points:** 5

---

#### US-002: Monitor Agent Performance
**As a** trader using the platform
**I want** to view real-time portfolio performance and trade history
**So that** I can verify the agent is performing as expected

**Acceptance Criteria:**
- Given I have an active agent with trades
- When I open the agent's portfolio view
- Then I see a line chart of portfolio value over the last 24 hours
- And I see a pie chart of asset allocation
- And I see a table of the 50 most recent trades with P&L
- And the data updates every 5 seconds without page refresh

**Priority:** HIGH
**Story Points:** 8

---

#### US-003: Pause Agent During Market Volatility
**As a** risk manager
**I want** to immediately pause an agent's trading during unusual market conditions
**So that** I can prevent potential losses

**Acceptance Criteria:**
- Given I have an active agent that's trading
- When I click "Pause Agent" in the dashboard
- Then the agent stops placing new trades within 5 seconds
- And existing open orders are cancelled
- And I see the agent status change to "Paused"
- And I receive a confirmation notification

**Priority:** MEDIUM
**Story Points:** 3

---

### 4.2 Metering & Usage Tracking

#### US-004: View API Usage Dashboard
**As a** CFO evaluating AI cost management tools
**I want** to see detailed API usage and budget consumption
**So that** I can assess the value of creto-metering for cost control

**Acceptance Criteria:**
- Given I have agents with historical usage data
- When I navigate to the "Usage" tab
- Then I see a burn-down chart showing cumulative budget spend over time
- And I see API calls consumed vs. quota limit (e.g., "7,342 / 10,000 calls")
- And I see a breakdown of costs by API endpoint (trading, market data, analytics)
- And I can filter by date range (today, last 7 days, last 30 days)

**Priority:** HIGH
**Story Points:** 8

---

#### US-005: Receive Quota Alert
**As a** platform administrator
**I want** to be notified when an agent approaches its quota limit
**So that** I can take action before the agent is throttled

**Acceptance Criteria:**
- Given an agent has consumed 80% of its API quota
- When the threshold is crossed
- Then I receive an email alert within 1 minute
- And I see a banner notification in the dashboard
- And the alert includes current usage, quota limit, and projected exhaustion date
- And I can click "Increase Quota" to raise the limit

**Priority:** MEDIUM
**Story Points:** 5

---

#### US-006: Export Usage Report for Billing
**As a** finance analyst
**I want** to export detailed usage logs for billing reconciliation
**So that** I can accurately charge internal teams for AI usage

**Acceptance Criteria:**
- Given I have usage data for the last 30 days
- When I click "Export Usage Report" and select "CSV" format
- Then a CSV file downloads within 5 seconds
- And the file includes columns: timestamp, agent_id, endpoint, cost, response_time
- And the file totals match the dashboard's displayed usage
- And I can filter by agent, date range, or endpoint before exporting

**Priority:** MEDIUM
**Story Points:** 3

---

### 4.3 Oversight & Approval Workflows

#### US-007: Trigger Approval for High-Value Trade
**As a** compliance officer
**I want** high-value trades to require human approval
**So that** we maintain control over risky financial decisions

**Acceptance Criteria:**
- Given an agent wants to execute a $75,000 trade
- When the trade decision is made
- Then the system creates an approval request
- And the agent pauses trading until approval/rejection
- And I receive a Slack notification within 10 seconds
- And the request includes trade details, risk assessment, and portfolio impact

**Priority:** HIGH
**Story Points:** 8

---

#### US-008: Approve Trade via Slack
**As a** fund manager
**I want** to approve or reject trades directly from Slack
**So that** I can quickly respond without switching contexts

**Acceptance Criteria:**
- Given I received a trade approval request in Slack
- When I click "Approve" in the Slack message
- Then the trade executes within 5 seconds
- And the Slack thread updates with "Approved by @username at 3:45 PM"
- And the agent resumes trading
- And I see the approved trade in the dashboard trade history

**Priority:** HIGH
**Story Points:** 5

---

#### US-009: Reject Trade with Reason
**As a** risk manager
**I want** to reject a proposed trade and provide reasoning
**So that** the system can learn from my decisions

**Acceptance Criteria:**
- Given I received a trade approval request
- When I click "Reject" and enter reason "Position too concentrated in tech sector"
- Then the trade is cancelled
- And the rejection reason is logged in the audit trail
- And the agent resumes trading (avoiding similar trades)
- And I see the rejection in the oversight analytics dashboard

**Priority:** MEDIUM
**Story Points:** 5

---

#### US-010: View Approval Analytics
**As a** product owner
**I want** to analyze approval patterns and bottlenecks
**So that** I can optimize oversight thresholds for the demo

**Acceptance Criteria:**
- Given I have 30 days of approval history
- When I navigate to "Oversight Analytics"
- Then I see approval rate (e.g., "85% approved, 15% rejected")
- And I see average approval time (e.g., "23 seconds")
- And I see a chart of triggers per day
- And I see recommendations: "Increase threshold to $75K to reduce approvals by 30%"

**Priority:** MEDIUM
**Story Points:** 8

---

### 4.4 Real-Time Dashboard Experience

#### US-011: View Live Portfolio Updates
**As a** demo presenter
**I want** the dashboard to update in real-time during live demos
**So that** investors see the dynamic nature of autonomous trading

**Acceptance Criteria:**
- Given I have the dashboard open during a demo
- When a trade executes
- Then I see a toast notification: "AAPL: Bought 10 shares at $150.25"
- And the portfolio chart updates within 1 second
- And the asset allocation pie chart re-renders with new percentages
- And there's a smooth animation (no jarring jumps)

**Priority:** HIGH
**Story Points:** 8

---

#### US-012: Toggle Dark Mode
**As a** user presenting in different lighting conditions
**I want** to switch between light and dark themes
**So that** the dashboard is readable in any environment

**Acceptance Criteria:**
- Given I'm viewing the dashboard
- When I click the theme toggle in the top-right corner
- Then the entire UI switches to dark mode within 300ms
- And my preference is saved (persists on page reload)
- And all charts and graphs adjust colors for visibility
- And WCAG contrast requirements are still met

**Priority:** LOW
**Story Points:** 3

---

### 4.5 Admin & Configuration

#### US-013: Enable Demo Mode
**As a** sales engineer preparing for a presentation
**I want** to enable demo mode with synthetic data
**So that** the demo runs smoothly without real market dependencies

**Acceptance Criteria:**
- Given I'm an admin user
- When I toggle "Demo Mode" in settings
- Then the system uses pre-recorded market data for trades
- And trades execute instantly (no real broker delays)
- And I can "fast-forward" time to show 30 days of activity in 5 minutes
- And a banner displays "DEMO MODE ACTIVE" to avoid confusion

**Priority:** HIGH
**Story Points:** 13

---

#### US-014: Configure Approval Thresholds
**As a** platform administrator
**I want** to customize approval triggers per agent
**So that** I can demonstrate different governance scenarios

**Acceptance Criteria:**
- Given I'm viewing an agent's configuration
- When I edit "Oversight Rules" and set "Require approval for trades > $25,000"
- Then the new threshold saves and takes effect within 5 seconds
- And the system validates the threshold is a positive number
- And I can preview how many historical trades would have triggered this rule
- And changes are logged in the audit trail

**Priority:** MEDIUM
**Story Points:** 5

---

#### US-015: Reset Demo Environment
**As a** product manager running multiple investor demos per day
**I want** to reset all agents and data with one click
**So that** each demo starts with a clean slate

**Acceptance Criteria:**
- Given I'm an admin user
- When I click "Reset Demo Environment" and confirm
- Then all agents are deleted
- And all trades, usage logs, and portfolios are cleared
- And default demo agents are re-created (3 agents with different strategies)
- And the reset completes within 10 seconds
- And I see a success message: "Demo environment ready"

**Priority:** MEDIUM
**Story Points:** 8

---

### 4.6 Security & Compliance

#### US-016: Require MFA for Approvers
**As a** security administrator
**I want** to enforce multi-factor authentication for approvers
**So that** we prevent unauthorized trade approvals

**Acceptance Criteria:**
- Given a user has the "Approver" role
- When they attempt to log in
- Then they must complete MFA (TOTP or SMS code)
- And they cannot approve trades until MFA is verified
- And MFA sessions expire after 8 hours
- And failed MFA attempts are logged for security auditing

**Priority:** MEDIUM
**Story Points:** 5

---

#### US-017: Audit Trail Viewer
**As a** compliance officer
**I want** to search and filter the audit log
**So that** I can investigate specific user actions or security events

**Acceptance Criteria:**
- Given I have admin access
- When I navigate to "Audit Log"
- Then I see a table of all logged actions (paginated, 100 per page)
- And I can filter by: user, action type, date range, resource ID
- And I can search by keywords (e.g., "agent-123", "trade rejected")
- And I can export filtered results to CSV
- And entries show: timestamp, user, IP, action, resource, outcome

**Priority:** MEDIUM
**Story Points:** 5

---

### 4.7 Performance & Reliability

#### US-018: Graceful Degradation on WebSocket Failure
**As a** user with an unstable network connection
**I want** the dashboard to continue working if real-time updates fail
**So that** I can still view cached data and perform actions

**Acceptance Criteria:**
- Given the WebSocket connection is lost
- When I continue using the dashboard
- Then I see a "Live updates paused" banner
- And cached portfolio data remains visible
- And I can still execute manual trades (with loading spinner)
- And the system automatically reconnects when the network recovers
- And updates sync after reconnection (no data loss)

**Priority:** MEDIUM
**Story Points:** 8

---

#### US-019: Fast Page Load for Investor Demos
**As a** presenter opening the dashboard in front of investors
**I want** the page to load in under 2 seconds
**So that** there's no awkward waiting during the demo

**Acceptance Criteria:**
- Given I have a stable internet connection
- When I navigate to the dashboard URL
- Then the page is fully interactive within 2 seconds
- And critical content (agent list, portfolio summary) loads first
- And non-critical content (historical charts) lazy-loads afterward
- And Lighthouse performance score is > 90

**Priority:** HIGH
**Story Points:** 8

---

#### US-020: Recover from Failed Trade Execution
**As a** system
**I want** to retry failed trades automatically
**So that** transient errors don't disrupt the demo

**Acceptance Criteria:**
- Given a trade execution fails due to a network timeout
- When the failure is detected
- Then the system retries the trade after 1 second (exponential backoff: 1s, 2s, 4s)
- And a maximum of 3 retry attempts are made
- And if all retries fail, the trade is marked "Failed" with error details
- And the user receives a notification: "Trade failed after 3 attempts"
- And the agent continues trading (doesn't stop due to one failure)

**Priority:** MEDIUM
**Story Points:** 5

---

## 5. API Contracts

### 5.1 Agent Management API

#### POST /api/v1/agents
**Description:** Create a new trading agent
**Authentication:** Required (JWT)
**Authorization:** Roles: Admin, Trader

**Request Body:**
```json
{
  "name": "Demo Agent Alpha",
  "strategy": "balanced",
  "initialBudget": 10000,
  "quotaConfig": {
    "apiCalls": 10000,
    "budgetPerMonth": 10000
  },
  "riskTolerance": 5,
  "assetPreferences": ["AAPL", "GOOGL", "MSFT"],
  "tradingHours": {
    "start": "09:30",
    "end": "16:00",
    "timezone": "America/New_York"
  }
}
```

**Response (201 Created):**
```json
{
  "agentId": "agt_7f8a9b2c3d4e5f6g",
  "name": "Demo Agent Alpha",
  "status": "paused",
  "createdAt": "2025-12-26T10:30:00Z",
  "portfolio": {
    "value": 10000.00,
    "cash": 10000.00,
    "positions": []
  },
  "quotaConfig": {
    "apiCalls": 10000,
    "budgetPerMonth": 10000,
    "usage": {
      "apiCalls": 0,
      "budgetSpent": 0.00
    }
  }
}
```

**Error Responses:**
- 400 Bad Request: Invalid parameters
- 401 Unauthorized: Missing or invalid JWT
- 403 Forbidden: Insufficient permissions
- 422 Unprocessable Entity: Budget exceeds limit

---

#### GET /api/v1/agents/{agentId}/portfolio
**Description:** Retrieve agent's current portfolio and performance
**Authentication:** Required (JWT)
**Authorization:** Roles: Admin, Trader, Viewer

**Path Parameters:**
- `agentId` (string, required): Agent identifier

**Query Parameters:**
- `includeHistory` (boolean, optional): Include historical portfolio values (default: false)
- `timeRange` (string, optional): Time range for history (1H, 1D, 1W, 1M, ALL)

**Response (200 OK):**
```json
{
  "agentId": "agt_7f8a9b2c3d4e5f6g",
  "portfolio": {
    "totalValue": 12547.89,
    "cash": 2341.23,
    "positions": [
      {
        "symbol": "AAPL",
        "quantity": 50,
        "averageCost": 145.30,
        "currentPrice": 150.25,
        "marketValue": 7512.50,
        "unrealizedPnL": 247.50,
        "unrealizedPnLPercent": 3.40
      },
      {
        "symbol": "GOOGL",
        "quantity": 20,
        "averageCost": 138.50,
        "currentPrice": 142.71,
        "marketValue": 2854.20,
        "unrealizedPnL": 84.20,
        "unrealizedPnLPercent": 3.04
      }
    ],
    "performance": {
      "totalReturn": 2547.89,
      "totalReturnPercent": 25.48,
      "dayReturn": 123.45,
      "dayReturnPercent": 0.99,
      "sharpeRatio": 1.82,
      "maxDrawdown": -5.2
    }
  },
  "history": [
    {
      "timestamp": "2025-12-26T09:00:00Z",
      "value": 12400.00
    },
    {
      "timestamp": "2025-12-26T10:00:00Z",
      "value": 12547.89
    }
  ]
}
```

**Error Responses:**
- 404 Not Found: Agent does not exist
- 401 Unauthorized: Missing or invalid JWT

---

#### POST /api/v1/agents/{agentId}/trades
**Description:** Execute a trade on behalf of an agent
**Authentication:** Required (JWT)
**Authorization:** Roles: Admin, Trader

**Path Parameters:**
- `agentId` (string, required): Agent identifier

**Request Body:**
```json
{
  "symbol": "AAPL",
  "action": "buy",
  "quantity": 10,
  "orderType": "market",
  "limitPrice": null,
  "manual": true
}
```

**Response (202 Accepted):**
```json
{
  "tradeId": "trd_8g9h0i1j2k3l4m5n",
  "agentId": "agt_7f8a9b2c3d4e5f6g",
  "symbol": "AAPL",
  "action": "buy",
  "quantity": 10,
  "orderType": "market",
  "status": "pending",
  "submittedAt": "2025-12-26T10:45:00Z",
  "estimatedCost": 1502.50,
  "fees": 1.00
}
```

**Response (200 OK - Completed):**
```json
{
  "tradeId": "trd_8g9h0i1j2k3l4m5n",
  "status": "completed",
  "executedAt": "2025-12-26T10:45:02Z",
  "executedPrice": 150.23,
  "totalCost": 1503.30,
  "fees": 1.00
}
```

**Error Responses:**
- 400 Bad Request: Invalid trade parameters
- 402 Payment Required: Insufficient budget/quota
- 409 Conflict: Agent is paused or requires approval

---

#### PATCH /api/v1/agents/{agentId}
**Description:** Update agent configuration
**Authentication:** Required (JWT)
**Authorization:** Roles: Admin, Trader

**Request Body (partial update):**
```json
{
  "name": "Demo Agent Alpha - Updated",
  "riskTolerance": 7,
  "quotaConfig": {
    "apiCalls": 15000
  }
}
```

**Response (200 OK):**
```json
{
  "agentId": "agt_7f8a9b2c3d4e5f6g",
  "name": "Demo Agent Alpha - Updated",
  "riskTolerance": 7,
  "quotaConfig": {
    "apiCalls": 15000,
    "budgetPerMonth": 10000
  },
  "updatedAt": "2025-12-26T11:00:00Z"
}
```

---

#### DELETE /api/v1/agents/{agentId}
**Description:** Terminate and delete an agent
**Authentication:** Required (JWT)
**Authorization:** Roles: Admin

**Query Parameters:**
- `liquidate` (boolean, optional): Liquidate all positions before deletion (default: true)

**Response (200 OK):**
```json
{
  "agentId": "agt_7f8a9b2c3d4e5f6g",
  "status": "deleted",
  "finalPortfolioValue": 12547.89,
  "liquidationTrades": [
    {
      "symbol": "AAPL",
      "quantity": 50,
      "proceeds": 7512.50
    }
  ],
  "deletedAt": "2025-12-26T11:15:00Z"
}
```

---

### 5.2 Metering API

#### GET /api/v1/agents/{agentId}/usage
**Description:** Retrieve agent's usage and quota information
**Authentication:** Required (JWT)
**Authorization:** Roles: Admin, Trader, Viewer

**Query Parameters:**
- `timeRange` (string, optional): today, week, month (default: month)
- `granularity` (string, optional): hour, day, week (default: day)

**Response (200 OK):**
```json
{
  "agentId": "agt_7f8a9b2c3d4e5f6g",
  "quotaConfig": {
    "apiCalls": 10000,
    "budgetPerMonth": 10000,
    "resetDate": "2026-01-01T00:00:00Z"
  },
  "currentUsage": {
    "apiCalls": 7342,
    "budgetSpent": 6234.56,
    "percentUsed": {
      "apiCalls": 73.42,
      "budget": 62.35
    }
  },
  "projections": {
    "estimatedExhaustion": "2026-12-28T14:30:00Z",
    "recommendedQuota": 12000
  },
  "breakdown": {
    "tradingAPI": 5123,
    "marketData": 1890,
    "analytics": 329
  },
  "history": [
    {
      "timestamp": "2025-12-26T00:00:00Z",
      "apiCalls": 234,
      "budgetSpent": 187.32
    }
  ]
}
```

---

#### POST /api/v1/agents/{agentId}/usage/alerts
**Description:** Configure usage alert thresholds
**Authentication:** Required (JWT)
**Authorization:** Roles: Admin, Trader

**Request Body:**
```json
{
  "alertThresholds": [
    {
      "type": "apiCalls",
      "threshold": 80,
      "channels": ["email", "slack"]
    },
    {
      "type": "budget",
      "threshold": 90,
      "channels": ["email", "slack", "sms"]
    }
  ]
}
```

**Response (200 OK):**
```json
{
  "agentId": "agt_7f8a9b2c3d4e5f6g",
  "alertsConfigured": 2,
  "updatedAt": "2025-12-26T11:30:00Z"
}
```

---

### 5.3 Oversight API

#### POST /api/v1/oversight/requests
**Description:** Create approval request for high-value trade (system-generated)
**Authentication:** Required (Service Token)
**Authorization:** Internal service only

**Request Body:**
```json
{
  "agentId": "agt_7f8a9b2c3d4e5f6g",
  "tradeId": "trd_8g9h0i1j2k3l4m5n",
  "tradeDetails": {
    "symbol": "TSLA",
    "action": "buy",
    "quantity": 500,
    "estimatedValue": 75000.00
  },
  "reason": "Trade value exceeds $50,000 threshold",
  "riskAssessment": {
    "score": 7.2,
    "positionConcentration": 35.5,
    "portfolioImpact": "High"
  },
  "expiresAt": "2025-12-26T12:00:00Z"
}
```

**Response (201 Created):**
```json
{
  "requestId": "ovr_9h0i1j2k3l4m5n6o",
  "status": "pending",
  "approvers": ["@john-doe", "@jane-smith"],
  "slackMessageId": "1234567890.123456",
  "createdAt": "2025-12-26T11:50:00Z",
  "expiresAt": "2025-12-26T12:00:00Z"
}
```

---

#### POST /api/v1/oversight/requests/{requestId}/decision
**Description:** Record approval or rejection decision (triggered by Slack webhook)
**Authentication:** Required (Service Token)
**Authorization:** Internal service only

**Request Body:**
```json
{
  "decision": "approved",
  "approverId": "usr_3e4f5g6h7i8j9k0l",
  "approverName": "John Doe",
  "reason": "Trade aligns with portfolio strategy",
  "timestamp": "2025-12-26T11:52:00Z"
}
```

**Response (200 OK):**
```json
{
  "requestId": "ovr_9h0i1j2k3l4m5n6o",
  "status": "approved",
  "decision": {
    "result": "approved",
    "approver": "John Doe",
    "approvedAt": "2025-12-26T11:52:00Z",
    "approvalLatency": 120
  },
  "tradeExecuted": true,
  "tradeId": "trd_8g9h0i1j2k3l4m5n"
}
```

---

#### GET /api/v1/oversight/analytics
**Description:** Retrieve oversight analytics and performance metrics
**Authentication:** Required (JWT)
**Authorization:** Roles: Admin, Approver

**Query Parameters:**
- `timeRange` (string, optional): 1D, 7D, 30D (default: 30D)

**Response (200 OK):**
```json
{
  "summary": {
    "totalRequests": 127,
    "approved": 108,
    "rejected": 15,
    "expired": 4,
    "approvalRate": 85.04,
    "avgApprovalTime": 23.4
  },
  "trends": [
    {
      "date": "2025-12-26",
      "requests": 5,
      "approved": 4,
      "rejected": 1
    }
  ],
  "approverPerformance": [
    {
      "approver": "John Doe",
      "requests": 45,
      "avgResponseTime": 18.2
    }
  ],
  "recommendations": [
    "Increase approval threshold to $75K to reduce requests by 30%",
    "Automate approvals for trades < $60K with low risk scores"
  ]
}
```

---

### 5.4 WebSocket API

#### WS /api/v1/ws/agents/{agentId}
**Description:** Real-time updates for agent portfolio and trades
**Authentication:** Required (JWT in query param: `?token=<JWT>`)

**Client → Server (Subscribe):**
```json
{
  "type": "subscribe",
  "channels": ["portfolio", "trades", "usage"]
}
```

**Server → Client (Portfolio Update):**
```json
{
  "type": "portfolio.update",
  "agentId": "agt_7f8a9b2c3d4e5f6g",
  "timestamp": "2025-12-26T11:55:00Z",
  "data": {
    "totalValue": 12587.34,
    "dayReturn": 163.00,
    "dayReturnPercent": 1.31
  }
}
```

**Server → Client (Trade Executed):**
```json
{
  "type": "trade.executed",
  "agentId": "agt_7f8a9b2c3d4e5f6g",
  "timestamp": "2025-12-26T11:56:00Z",
  "data": {
    "tradeId": "trd_8g9h0i1j2k3l4m5n",
    "symbol": "AAPL",
    "action": "buy",
    "quantity": 10,
    "executedPrice": 150.45,
    "totalCost": 1505.50
  }
}
```

**Server → Client (Quota Alert):**
```json
{
  "type": "usage.alert",
  "agentId": "agt_7f8a9b2c3d4e5f6g",
  "timestamp": "2025-12-26T11:57:00Z",
  "data": {
    "alertType": "apiCalls",
    "threshold": 80,
    "currentUsage": 8123,
    "quota": 10000
  }
}
```

**Server → Client (Approval Request):**
```json
{
  "type": "oversight.request",
  "agentId": "agt_7f8a9b2c3d4e5f6g",
  "timestamp": "2025-12-26T11:58:00Z",
  "data": {
    "requestId": "ovr_9h0i1j2k3l4m5n6o",
    "tradeDetails": {
      "symbol": "TSLA",
      "action": "buy",
      "quantity": 500,
      "estimatedValue": 75000.00
    },
    "expiresAt": "2025-12-26T12:08:00Z"
  }
}
```

---

## 6. Data Models

### 6.1 Entity Relationship Diagram (Conceptual)

```
┌─────────────┐       ┌──────────────┐       ┌─────────────┐
│    User     │──────<│    Agent     │>──────│  Portfolio  │
└─────────────┘       └──────────────┘       └─────────────┘
                            │   │                    │
                            │   │                    │
                            │   └───────┐            │
                            │           │            │
                      ┌─────▼────┐  ┌──▼────────┐  ┌▼──────────┐
                      │  Trade   │  │QuotaUsage │  │ Position  │
                      └──────────┘  └───────────┘  └───────────┘
                            │
                            │
                      ┌─────▼────────────┐
                      │ApprovalRequest   │
                      └──────────────────┘
```

### 6.2 Core Entities

#### Agent
```yaml
Agent:
  id: uuid (primary key)
  userId: uuid (foreign key → User)
  name: string (max 100 chars, unique per user)
  strategy: enum (balanced, aggressive, conservative, custom)
  status: enum (active, paused, stopped, error)
  riskTolerance: integer (1-10 scale)
  createdAt: timestamp
  updatedAt: timestamp
  deletedAt: timestamp (nullable, soft delete)

  relationships:
    - has_one: Portfolio
    - has_many: Trades
    - has_one: QuotaUsage
    - has_many: ApprovalRequests

  indexes:
    - userId, status
    - createdAt (descending)
```

#### Portfolio
```yaml
Portfolio:
  id: uuid (primary key)
  agentId: uuid (foreign key → Agent, unique)
  totalValue: decimal (precision 18, scale 2)
  cash: decimal (precision 18, scale 2)
  updatedAt: timestamp

  relationships:
    - belongs_to: Agent
    - has_many: Positions
    - has_many: PortfolioSnapshots (historical)

  computed_fields:
    - totalReturn: sum(positions.unrealizedPnL) + sum(trades.realizedPnL)
    - dayReturn: totalValue(today) - totalValue(yesterday)
```

#### Position
```yaml
Position:
  id: uuid (primary key)
  portfolioId: uuid (foreign key → Portfolio)
  symbol: string (max 10 chars, e.g., AAPL)
  quantity: integer (can be negative for shorts)
  averageCost: decimal (precision 10, scale 4)
  currentPrice: decimal (precision 10, scale 4, cached)
  lastPriceUpdate: timestamp

  relationships:
    - belongs_to: Portfolio

  computed_fields:
    - marketValue: quantity * currentPrice
    - unrealizedPnL: (currentPrice - averageCost) * quantity
    - unrealizedPnLPercent: ((currentPrice / averageCost) - 1) * 100

  indexes:
    - portfolioId, symbol (unique together)
```

#### Trade
```yaml
Trade:
  id: uuid (primary key)
  agentId: uuid (foreign key → Agent)
  symbol: string (max 10 chars)
  action: enum (buy, sell)
  quantity: integer (positive)
  orderType: enum (market, limit, stop_loss)
  limitPrice: decimal (nullable)
  status: enum (pending, completed, failed, cancelled)
  submittedAt: timestamp
  executedAt: timestamp (nullable)
  executedPrice: decimal (nullable)
  totalCost: decimal (precision 18, scale 2)
  fees: decimal (precision 10, scale 2)
  manual: boolean (false = autonomous, true = user-initiated)
  reasoning: text (AI decision explanation)

  relationships:
    - belongs_to: Agent
    - has_one: ApprovalRequest (nullable)

  indexes:
    - agentId, submittedAt (descending)
    - status, submittedAt
```

#### QuotaUsage
```yaml
QuotaUsage:
  id: uuid (primary key)
  agentId: uuid (foreign key → Agent, unique)
  apiCallsQuota: integer (default 10000)
  apiCallsUsed: integer (default 0)
  budgetQuota: decimal (precision 18, scale 2, default 10000.00)
  budgetSpent: decimal (precision 18, scale 2, default 0.00)
  resetDate: date (next quota reset, e.g., 1st of month)
  lastUpdated: timestamp

  relationships:
    - belongs_to: Agent
    - has_many: UsageLogs

  computed_fields:
    - apiCallsRemaining: apiCallsQuota - apiCallsUsed
    - budgetRemaining: budgetQuota - budgetSpent
    - percentUsed: (apiCallsUsed / apiCallsQuota) * 100

  indexes:
    - agentId (unique)
```

#### UsageLog
```yaml
UsageLog:
  id: uuid (primary key)
  quotaUsageId: uuid (foreign key → QuotaUsage)
  timestamp: timestamp
  endpoint: string (e.g., /api/trading/execute)
  cost: decimal (precision 10, scale 4, in dollars)
  responseTime: integer (milliseconds)
  statusCode: integer (HTTP status)

  relationships:
    - belongs_to: QuotaUsage

  indexes:
    - quotaUsageId, timestamp (descending)
    - timestamp (partitioned by month)
```

#### ApprovalRequest
```yaml
ApprovalRequest:
  id: uuid (primary key)
  agentId: uuid (foreign key → Agent)
  tradeId: uuid (foreign key → Trade, unique)
  status: enum (pending, approved, rejected, expired)
  reason: text (why approval was triggered)
  riskScore: decimal (precision 3, scale 1, e.g., 7.2)
  approvers: json (array of user IDs)
  slackMessageId: string (nullable)
  createdAt: timestamp
  expiresAt: timestamp
  decidedAt: timestamp (nullable)
  decision: enum (approved, rejected, nullable)
  decidedBy: uuid (foreign key → User, nullable)
  decisionReason: text (nullable)

  relationships:
    - belongs_to: Agent
    - belongs_to: Trade
    - belongs_to: User (decidedBy)

  indexes:
    - agentId, status
    - createdAt (descending)
    - tradeId (unique)
```

#### User
```yaml
User:
  id: uuid (primary key)
  email: string (unique, max 255 chars)
  passwordHash: string (bcrypt)
  fullName: string (max 100 chars)
  role: enum (admin, trader, viewer, approver)
  mfaEnabled: boolean (default false)
  mfaSecret: string (nullable, encrypted)
  createdAt: timestamp
  lastLoginAt: timestamp (nullable)

  relationships:
    - has_many: Agents
    - has_many: ApprovalRequests (as decidedBy)
    - has_many: AuditLogs

  indexes:
    - email (unique)
```

#### AuditLog
```yaml
AuditLog:
  id: uuid (primary key)
  userId: uuid (foreign key → User, nullable for system actions)
  action: string (e.g., agent.create, trade.execute, approval.approve)
  resourceType: string (e.g., agent, trade, user)
  resourceId: uuid
  ipAddress: string
  userAgent: string
  timestamp: timestamp
  details: jsonb (action-specific metadata)

  relationships:
    - belongs_to: User (nullable)

  indexes:
    - userId, timestamp (descending)
    - resourceType, resourceId, timestamp
    - timestamp (partitioned by month)
```

---

## 7. Success Metrics

### 7.1 Demo Performance Metrics

#### Demo Reliability
- **Target:** 99.9% uptime during investor presentation hours (9 AM - 6 PM EST)
- **Measurement:** Uptime monitoring with PagerDuty alerts
- **Success Criteria:** Zero demo failures in investor meetings over 90 days

#### Demo Engagement
- **Target:** 100% of investors request follow-up demo or product trial
- **Measurement:** Post-demo survey and CRM tracking
- **Success Criteria:** Conversion rate from demo to trial ≥ 75%

#### Load Time Performance
- **Target:** Dashboard loads in < 2 seconds on first visit
- **Measurement:** Lighthouse CI and RUM
- **Success Criteria:** P95 load time < 2.5 seconds

---

### 7.2 Metering Value Proposition Metrics

#### Cost Visibility
- **Target:** Investors can explain creto-metering value in < 30 seconds after demo
- **Measurement:** Post-demo comprehension test
- **Success Criteria:** 90% of investors correctly describe quota enforcement

#### Quota Enforcement Demonstration
- **Target:** Trigger quota alert during every demo (controlled)
- **Measurement:** Demo script adherence checklist
- **Success Criteria:** Alert appears within 5 seconds of trigger event

#### Usage Analytics Clarity
- **Target:** Investors understand usage breakdown without explanation
- **Measurement:** Eye-tracking and verbal feedback
- **Success Criteria:** No questions about "what does this chart mean?"

---

### 7.3 Oversight Value Proposition Metrics

#### Approval Workflow Speed
- **Target:** Approval decision within 30 seconds (from trigger to resolution)
- **Measurement:** ApprovalRequest.decidedAt - ApprovalRequest.createdAt
- **Success Criteria:** P95 approval latency < 30 seconds

#### Approval UX Delight
- **Target:** Investors say "wow, that was fast" or equivalent during demo
- **Measurement:** Demo recording transcript analysis
- **Success Criteria:** Positive reaction in 80% of demos

#### Slack Integration Impression
- **Target:** Investors ask "can we integrate this with our tools?" after Slack demo
- **Measurement:** Post-demo question tracking
- **Success Criteria:** 60% of investors inquire about custom integrations

---

### 7.4 Technical Performance Metrics

#### API Latency
- **Target:** P95 latency < 200ms for all GET requests
- **Measurement:** APM dashboard (Datadog)
- **Success Criteria:** No latency regressions in production

#### WebSocket Reliability
- **Target:** < 0.1% WebSocket disconnections during demos
- **Measurement:** Connection error rate monitoring
- **Success Criteria:** Zero visible reconnections during 100+ demos

#### Test Coverage
- **Target:** 90% code coverage (unit + integration + e2e)
- **Measurement:** Jest coverage reports
- **Success Criteria:** Coverage gate in CI/CD blocks < 90% PRs

---

### 7.5 Business Impact Metrics

#### Sales Acceleration
- **Target:** Reduce sales cycle by 20% with compelling demo
- **Measurement:** CRM pipeline velocity analysis
- **Success Criteria:** Average time to close: 45 days → 36 days

#### Product Differentiation
- **Target:** 100% of investors mention "governance" or "cost control" as key differentiator
- **Measurement:** Post-demo survey keywords
- **Success Criteria:** "Governance" appears in 100% of feedback

#### Competitive Advantage
- **Target:** Investors compare favorably to competitors in 90% of demos
- **Measurement:** Win/loss analysis
- **Success Criteria:** "Better governance than [Competitor X]" in 90% of win reasons

---

## 8. Constraints & Assumptions

### 8.1 Technical Constraints

- **Infrastructure:** Must deploy to AWS (company standard)
- **Database:** PostgreSQL 14+ (existing company infrastructure)
- **Authentication:** Integrate with company SSO (Okta)
- **Monitoring:** Use Datadog for APM (company license)
- **Budget:** Development budget $50,000 (contractor hours)

### 8.2 Business Constraints

- **Timeline:** Must be demo-ready in 8 weeks for Q1 2026 investor meetings
- **Team Size:** 3 developers (1 frontend, 1 backend, 1 full-stack)
- **Regulatory:** No real trading (simulated trades only for demo safety)
- **Data:** Use synthetic market data (avoid real-time data provider costs)

### 8.3 Assumptions

- **Investors:** 30-45 minute demo window (keep features focused)
- **Network:** Assume reliable internet (have offline fallback)
- **Approvers:** 1-2 approvers available during demos (rehearsed flow)
- **Market Data:** Pre-recorded data sufficient for demo (no real-time feeds)
- **Scaling:** Single region deployment sufficient for demo (multi-region future)

---

## 9. Out of Scope (Future Phases)

### 9.1 Not Included in v1.0

- Real broker integrations (Alpaca, Interactive Brokers)
- Live market data feeds (Bloomberg, Reuters)
- Multi-currency support (USD only in v1.0)
- Mobile app (responsive web only)
- Advanced analytics (ML-based predictions, sentiment analysis)
- Social trading features (copy trading, leaderboards)
- Tax reporting and capital gains tracking
- Multi-agent collaboration (agent-to-agent trading)
- Custom trading strategies via code editor
- Backtesting framework for strategies

### 9.2 Planned for v2.0 (Post-Demo)

- Production-grade security audit
- Real broker integrations for pilot customers
- Advanced oversight rules (ML-based risk scoring)
- Multi-tenant architecture for customer trials
- White-label capabilities for partners

---

## 10. Acceptance Criteria Summary

### 10.1 Demo Readiness Checklist

- [ ] **Agent Provisioning:** Create agent in < 10 clicks and < 30 seconds
- [ ] **Portfolio Dashboard:** Updates in real-time without refresh (< 5 second latency)
- [ ] **Metering Visibility:** Usage chart clearly shows quota consumption and burn rate
- [ ] **Oversight Workflow:** Approval trigger → Slack notification → Decision → Trade execution in < 30 seconds
- [ ] **Performance:** Dashboard loads in < 2 seconds, no UI lag
- [ ] **Reliability:** Zero crashes or errors in 20 consecutive rehearsal demos
- [ ] **Visual Polish:** Professional design matching company brand guidelines
- [ ] **Demo Mode:** One-click reset for clean demo environment
- [ ] **Documentation:** Demo script, troubleshooting guide, FAQ for presenters

### 10.2 Technical Acceptance Criteria

- [ ] **API Tests:** 90%+ code coverage with passing CI/CD
- [ ] **Load Testing:** 100 concurrent agents, 500 dashboard users, no degradation
- [ ] **Security Scan:** Zero critical vulnerabilities in Snyk/OWASP ZAP
- [ ] **Accessibility:** WCAG 2.1 AA compliance verified with axe DevTools
- [ ] **Browser Compatibility:** Chrome, Safari, Firefox, Edge (latest 2 versions)
- [ ] **Mobile Responsive:** Functional on 375px (iPhone) to 1920px (desktop)

---

## 11. Validation Plan

### 11.1 Specification Review

- [ ] **Product Team:** Validate features align with investor pitch deck
- [ ] **Engineering Team:** Confirm technical feasibility and timelines
- [ ] **Design Team:** Review UI/UX mockups against requirements
- [ ] **Compliance Team:** Approve oversight workflows and audit logging
- [ ] **Sales Team:** Validate demo script matches specification

### 11.2 Prototype Validation

- [ ] **Week 3:** Low-fidelity prototype demo to product owner
- [ ] **Week 5:** High-fidelity UI mockups review with stakeholders
- [ ] **Week 7:** Alpha demo with sales team (feedback collection)
- [ ] **Week 8:** Beta demo with friendly investor (dry run)

### 11.3 Sign-Off

- [ ] **Product Owner:** Specification approved for development
- [ ] **CTO:** Technical architecture approved
- [ ] **CFO:** Budget approved
- [ ] **Head of Sales:** Demo script approved

---

## 12. Glossary

- **Agent:** Autonomous trading software entity with budget and strategy
- **Approval Request:** Human review workflow for high-risk trades
- **Burn-Down Chart:** Visualization of budget spend over time
- **creto-metering:** Quota enforcement and usage tracking system
- **creto-oversight:** Human-in-the-loop approval workflow system
- **Demo Mode:** Synthetic data mode for controlled presentations
- **Portfolio:** Collection of financial positions held by an agent
- **Quota:** Resource limit (API calls, budget) enforced per agent
- **Risk Tolerance:** Agent's willingness to take risk (1-10 scale)
- **Trade:** Buy or sell order for a financial instrument
- **Usage Log:** Granular record of API call with timestamp and cost

---

## 13. Appendices

### 13.1 Reference Documents

- SPARC Methodology Guide: [Internal Wiki Link]
- Company Brand Guidelines: [Figma Link]
- Investor Pitch Deck: [Google Slides Link]
- creto-metering API Documentation: [Confluence Link]
- creto-oversight Design Patterns: [GitHub Wiki Link]

### 13.2 Stakeholder Contact Matrix

| Role               | Name          | Email                | Slack      |
|--------------------|---------------|----------------------|------------|
| Product Owner      | Alice Johnson | alice@company.com    | @alice     |
| Engineering Lead   | Bob Smith     | bob@company.com      | @bob       |
| UX Designer        | Carol Lee     | carol@company.com    | @carol     |
| Head of Sales      | David Kim     | david@company.com    | @david     |
| Compliance Officer | Eve Martinez  | eve@company.com      | @eve       |

---

**END OF SPECIFICATION**

---

## Next Steps

1. **Pseudocode Phase:** Translate requirements into algorithmic designs
2. **Architecture Phase:** Design system components, data flows, and infrastructure
3. **Refinement Phase:** Implement TDD with test-first development
4. **Completion Phase:** Integration testing, performance tuning, demo rehearsal

---

*This specification was generated using the SPARC methodology and represents the complete requirements for the Autonomous Portfolio Manager demo. All stakeholders should review and approve before proceeding to the Pseudocode phase.*