# Trading Demo UI Specification

**Version:** 1.0.0
**Date:** 2025-12-26
**Status:** Draft
**SPARC Phase:** Specification

---

## 1. Executive Summary

This specification defines the user interface and user experience for an investor-facing autonomous AI trading demo. The application showcases AI agents that execute trades autonomously while maintaining human oversight through approval workflows, budget controls, and real-time monitoring.

**Key Design Principles:**
- **Transparency**: All agent actions are visible and traceable
- **Control**: Human oversight at critical decision points
- **Trust**: Clear budget limits, approval workflows, and performance metrics
- **Clarity**: Professional financial interface with real-time data visualization

---

## 2. Design System

### 2.1 Color Palette

```
Primary Colors:
  - Primary Blue:    #1E40AF (rgb(30, 64, 175))
  - Primary Dark:    #1E3A8A (rgb(30, 58, 138))
  - Primary Light:   #3B82F6 (rgb(59, 130, 246))

Semantic Colors:
  - Success Green:   #059669 (rgb(5, 150, 105))
  - Success Light:   #10B981 (rgb(16, 185, 129))
  - Warning Amber:   #D97706 (rgb(217, 119, 6))
  - Warning Light:   #F59E0B (rgb(245, 158, 11))
  - Danger Red:      #DC2626 (rgb(220, 38, 38))
  - Danger Light:    #EF4444 (rgb(239, 68, 68))

Neutral Colors:
  - Gray 50:         #F9FAFB
  - Gray 100:        #F3F4F6
  - Gray 200:        #E5E7EB
  - Gray 300:        #D1D5DB
  - Gray 500:        #6B7280
  - Gray 700:        #374151
  - Gray 900:        #111827

Financial Colors:
  - Profit Green:    #10B981 (rgb(16, 185, 129))
  - Loss Red:        #EF4444 (rgb(239, 68, 68))
  - Neutral Gray:    #6B7280 (rgb(107, 114, 128))
```

### 2.2 Typography

```
Font Family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif

Type Scale:
  - Display:    48px / 56px line-height, font-weight: 700
  - H1:         36px / 44px line-height, font-weight: 700
  - H2:         30px / 36px line-height, font-weight: 600
  - H3:         24px / 32px line-height, font-weight: 600
  - H4:         20px / 28px line-height, font-weight: 600
  - H5:         18px / 26px line-height, font-weight: 600
  - Body Large: 16px / 24px line-height, font-weight: 400
  - Body:       14px / 20px line-height, font-weight: 400
  - Body Small: 12px / 16px line-height, font-weight: 400
  - Caption:    11px / 14px line-height, font-weight: 500

Monospace: 'JetBrains Mono', 'Fira Code', 'Monaco', monospace
  - Used for: Agent IDs, transaction hashes, numeric values
```

### 2.3 Spacing System

```
Based on 4px grid:

  - 4px:   xs (tight spacing)
  - 8px:   sm (compact components)
  - 12px:  md (standard component padding)
  - 16px:  lg (section spacing)
  - 24px:  xl (card padding)
  - 32px:  2xl (section margins)
  - 48px:  3xl (page sections)
  - 64px:  4xl (major divisions)
```

### 2.4 Border Radius

```
  - None:    0px (tables, strict elements)
  - Small:   4px (buttons, inputs)
  - Medium:  8px (cards, panels)
  - Large:   12px (modals, dialogs)
  - Full:    9999px (pills, badges)
```

### 2.5 Shadows

```
  - sm:  0 1px 2px 0 rgba(0, 0, 0, 0.05)
  - md:  0 4px 6px -1px rgba(0, 0, 0, 0.1)
  - lg:  0 10px 15px -3px rgba(0, 0, 0, 0.1)
  - xl:  0 20px 25px -5px rgba(0, 0, 0, 0.1)
```

---

## 3. Component Library

### 3.1 Buttons

```
Primary Button:
  - Background: Primary Blue (#1E40AF)
  - Text: White
  - Padding: 12px 24px
  - Border Radius: 4px
  - Font: 14px, weight 600
  - Hover: Primary Dark (#1E3A8A)
  - Active: Scale 0.98
  - Disabled: Gray 300, cursor not-allowed

Secondary Button:
  - Background: White
  - Border: 1px solid Gray 300
  - Text: Gray 700
  - Hover: Gray 100 background

Danger Button:
  - Background: Danger Red (#DC2626)
  - Text: White
  - Hover: Darker red

Success Button:
  - Background: Success Green (#059669)
  - Text: White
  - Hover: Darker green

Sizes:
  - Small: 8px 16px, 12px font
  - Medium: 12px 24px, 14px font (default)
  - Large: 16px 32px, 16px font
```

### 3.2 Inputs

```
Text Input:
  - Border: 1px solid Gray 300
  - Padding: 10px 12px
  - Border Radius: 4px
  - Font: 14px
  - Focus: Blue border, shadow
  - Error: Red border
  - Disabled: Gray 100 background

Search Input:
  - Icon: Magnifying glass (left side)
  - Clear button (right side, on input)
  - Autocomplete dropdown

Select Dropdown:
  - Chevron icon (right side)
  - Custom styled options
  - Max height with scroll

Toggle Switch:
  - Width: 44px, Height: 24px
  - Background: Gray when off, Primary Blue when on
  - Circle: 20px diameter
```

### 3.3 Cards

```
Standard Card:
  ┌─────────────────────────────────────────┐
  │  [Header with optional action button]   │
  ├─────────────────────────────────────────┤
  │                                         │
  │  Content area                           │
  │                                         │
  └─────────────────────────────────────────┘

  - Background: White
  - Border: 1px solid Gray 200
  - Border Radius: 8px
  - Padding: 24px
  - Shadow: sm
  - Hover: lg shadow (for interactive cards)
```

### 3.4 Tables

```
Data Table:
  - Header: Gray 50 background, Bold text
  - Rows: Alternating white/Gray 50 (zebra stripe)
  - Hover: Gray 100 background
  - Border: 1px solid Gray 200
  - Cell Padding: 12px 16px
  - Sortable columns: Arrow icons
  - Alignment: Left (text), Right (numbers)
```

### 3.5 Badges

```
Status Badge:
  - Padding: 4px 8px
  - Border Radius: 9999px (full)
  - Font: 11px, weight 500, uppercase
  - Variants:
    • Active:   Green background, dark green text
    • Paused:   Amber background, dark amber text
    • Error:    Red background, dark red text
    • Pending:  Gray background, dark gray text
```

### 3.6 Modals

```
Modal Overlay:
  - Background: rgba(0, 0, 0, 0.5)
  - z-index: 1000

Modal Container:
  - Background: White
  - Border Radius: 12px
  - Shadow: xl
  - Max Width: 600px (medium), 900px (large)
  - Padding: 32px
  - Center aligned

Modal Header:
  - Title: H3
  - Close button: Top right, X icon

Modal Footer:
  - Actions: Right aligned
  - Spacing: 12px between buttons
```

---

## 4. Screen Specifications

### 4.1 Dashboard Overview

**Purpose:** Primary landing page showing system status and quick access to key features.

**Layout:** Desktop (1440px+)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ [Logo]  Trading Demo                      [Notifications] [Profile Menu] │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Dashboard                                                               │
│                                                                          │
│  ┌─────────────────┬─────────────────┬─────────────────┬──────────────┐ │
│  │ Active Agents   │ Total AUM       │ Today's P&L     │ Pending      │ │
│  │                 │                 │                 │ Approvals    │ │
│  │  8              │  $1,247,500     │  +$12,450       │  3           │ │
│  │  ↑ 2 this week  │  ↑ 2.4%        │  +1.01%         │  [View]      │ │
│  └─────────────────┴─────────────────┴─────────────────┴──────────────┘ │
│                                                                          │
│  Quick Actions                                                           │
│  [+ Create Agent]  [View All Agents]  [Oversight Center]  [⚙ Settings] │
│                                                                          │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │ Active Agents                                  [View All →]       │  │
│  ├───────────────────────────────────────────────────────────────────┤  │
│  │                                                                   │  │
│  │ ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐ │  │
│  │ │ Growth Alpha     │  │ Value Hunter     │  │ Momentum Scout   │ │  │
│  │ │ [●Active]        │  │ [●Active]        │  │ [●Active]        │ │  │
│  │ │                  │  │                  │  │                  │ │  │
│  │ │ AUM: $125K       │  │ AUM: $230K       │  │ AUM: $180K       │ │  │
│  │ │ P&L: +$2,340     │  │ P&L: +$4,120     │  │ P&L: +$1,890     │ │  │
│  │ │ (1.87%)          │  │ (1.79%)          │  │ (1.05%)          │ │  │
│  │ │                  │  │                  │  │                  │ │  │
│  │ │ Budget: 45% used │  │ Budget: 23% used │  │ Budget: 67% used │ │  │
│  │ │ ████░░░░░░       │  │ ██░░░░░░░░       │  │ ██████░░░░       │ │  │
│  │ │                  │  │                  │  │                  │ │  │
│  │ │ [View Details →] │  │ [View Details →] │  │ [View Details →] │ │  │
│  │ └──────────────────┘  └──────────────────┘  └──────────────────┘ │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌──────────────────────────────┬──────────────────────────────────────┐ │
│  │ Recent Activity              │ System Metering                      │ │
│  ├──────────────────────────────┼──────────────────────────────────────┤ │
│  │                              │                                      │ │
│  │ ● 2m ago                     │  API Calls (Last 24h)                │ │
│  │   Growth Alpha bought        │  ┌────────────────────────────────┐  │ │
│  │   50 shares of AAPL          │  │         ▁▂█▅▃▇▆▄               │  │ │
│  │   $8,750                     │  │                                │  │ │
│  │                              │  └────────────────────────────────┘  │ │
│  │ ● 15m ago                    │                                      │ │
│  │   Value Hunter sold          │  Total: 12,450 calls                 │ │
│  │   100 shares of MSFT         │  Quota: 78% remaining                │ │
│  │   $37,200                    │  ███████░░░                          │ │
│  │                              │                                      │ │
│  │ ● 45m ago                    │  Cost: $124.50                       │ │
│  │   Momentum Scout             │  [View Breakdown →]                  │ │
│  │   submitted approval         │                                      │ │
│  │   request for TSLA           │                                      │ │
│  │                              │                                      │ │
│  │ [View All →]                 │                                      │ │
│  └──────────────────────────────┴──────────────────────────────────────┘ │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

**Components:**

1. **Top Navigation Bar** (fixed)
   - Left: Logo + "Trading Demo" text
   - Right: Notifications bell icon (with badge count), Profile menu
   - Height: 64px
   - Background: White
   - Border Bottom: 1px Gray 200

2. **Summary Cards** (Grid: 4 columns)
   - Active Agents: Count with trend arrow
   - Total AUM: Dollar amount with percentage change
   - Today's P&L: Dollar amount with percentage (colored)
   - Pending Approvals: Count with "View" link
   - Height: 120px each
   - Background: White cards with hover effect

3. **Quick Actions** (Horizontal button row)
   - Primary: "Create Agent" (primary button)
   - Secondary: View All Agents, Oversight Center
   - Tertiary: Settings (icon button)

4. **Active Agents Grid** (3 columns, responsive)
   - Agent name + status badge
   - AUM amount
   - P&L amount with percentage (colored)
   - Budget progress bar
   - "View Details" link
   - Card height: 280px

5. **Recent Activity Feed**
   - Chronological list with timestamps
   - Bullet points with agent name, action, amount
   - Max 5 items, "View All" link
   - Real-time updates (WebSocket)

6. **System Metering Panel**
   - Sparkline chart (24h API calls)
   - Total calls count
   - Quota usage bar
   - Cost summary
   - Link to detailed breakdown

**Interactions:**
- Agent cards: Click anywhere to navigate to detail view
- Summary cards: Click to filter/navigate to relevant view
- Activity items: Click to view transaction details
- Real-time updates: Smooth animations for new data

**Responsive Behavior:**
- Desktop (1440px+): 4-column grid for summary, 3-column for agents
- Tablet (768px-1439px): 2-column grid
- Mobile (< 768px): Single column, stacked layout

---

### 4.2 Agent Creation Wizard

**Purpose:** Multi-step form to configure and launch new trading agents.

**Layout:** Modal overlay with stepped progression

```
┌──────────────────────────────────────────────────────────────────────────┐
│                          Create New Agent                          [✕]   │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ○──────●──────○──────○──────○                                          │
│  Basic  Budget Trading Oversight Review                                 │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                                                                    │ │
│  │  Step 2: Budget Configuration                                     │ │
│  │                                                                    │ │
│  │  Monthly Spending Limit *                                         │ │
│  │  ┌──────────────────────────────────────────────────────────────┐ │ │
│  │  │ $                                                            │ │ │
│  │  └──────────────────────────────────────────────────────────────┘ │ │
│  │  Maximum amount this agent can spend per calendar month          │ │
│  │                                                                    │ │
│  │  Alert Thresholds                                                 │ │
│  │                                                                    │ │
│  │  Budget Warning (% of monthly limit)                             │ │
│  │  ┌─────────┐                                                      │ │
│  │  │   75%   │  [Slider: 0%────●────100%]                          │ │
│  │  └─────────┘                                                      │ │
│  │  Notify when agent reaches this percentage                        │ │
│  │                                                                    │ │
│  │  Daily Loss Limit                                                 │ │
│  │  ┌──────────────────────────────────────────────────────────────┐ │ │
│  │  │ $                                                            │ │ │
│  │  └──────────────────────────────────────────────────────────────┘ │ │
│  │  Pause agent if daily losses exceed this amount                   │ │
│  │                                                                    │ │
│  │  ┌───────────────────────────────────────────────────────────┐   │ │
│  │  │ ⚠ Budget Protection Enabled                               │   │ │
│  │  │                                                           │   │ │
│  │  │ Agent will automatically pause when limits are reached    │   │ │
│  │  │ Requires manual re-activation after review                │   │ │
│  │  └───────────────────────────────────────────────────────────┘   │ │
│  │                                                                    │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │  Required fields are marked with *                                │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│                            [← Back]  [Continue →]                        │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

**Step 1: Basic Information**

```
Fields:
  - Agent Name * (text input, max 50 chars)
  - Description (textarea, max 200 chars)
  - Agent Type (dropdown: Growth, Value, Momentum, Custom)
  - Avatar Color (color picker, 8 preset options)

Validation:
  - Name required, unique check
  - Description optional but recommended
  - Type required
```

**Step 2: Budget Configuration** (shown above)

```
Fields:
  - Monthly Spending Limit * (currency input, min $1,000)
  - Budget Warning Threshold (slider, 50-90%, default 75%)
  - Daily Loss Limit * (currency input, max 20% of monthly)
  - Auto-pause toggle (default: ON)

Validation:
  - Monthly limit >= $1,000
  - Daily loss <= 20% of monthly
  - Warning threshold between 50-90%
```

**Step 3: Trading Parameters**

```
┌────────────────────────────────────────────────────────────────────┐
│ Step 3: Trading Parameters                                         │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│ Risk Tolerance *                                                   │
│ ○ Conservative  ● Moderate  ○ Aggressive                          │
│                                                                    │
│ Asset Classes (select multiple)                                   │
│ ☑ US Equities    ☑ ETFs        ☐ Options                         │
│ ☐ Crypto         ☐ Bonds       ☐ Forex                           │
│                                                                    │
│ Position Sizing                                                    │
│ Maximum Position Size (% of portfolio)                            │
│ [Slider: 0%────●────100%]  25%                                    │
│                                                                    │
│ Maximum Open Positions                                            │
│ ┌────┐                                                            │
│ │ 10 │                                                            │
│ └────┘                                                            │
│                                                                    │
│ Trading Hours                                                      │
│ ○ Market Hours Only (9:30 AM - 4:00 PM ET)                       │
│ ○ Extended Hours (4:00 AM - 8:00 PM ET)                          │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘

Fields:
  - Risk Tolerance * (radio: Conservative/Moderate/Aggressive)
  - Asset Classes * (multi-checkbox, min 1 required)
  - Max Position Size (slider, 5-50%, default 25%)
  - Max Open Positions (number, 1-50, default 10)
  - Trading Hours (radio)
```

**Step 4: Oversight Rules**

```
┌────────────────────────────────────────────────────────────────────┐
│ Step 4: Oversight & Approvals                                      │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│ Approval Required For:                                            │
│                                                                    │
│ ☑ Trades exceeding threshold amount                              │
│   Threshold: $ ┌───────┐  (default: $10,000)                     │
│                └───────┘                                          │
│                                                                    │
│ ☑ High-risk trades (based on risk score)                         │
│   Risk Score Threshold: [Slider] 7.5/10                          │
│                                                                    │
│ ☑ First trade of each new symbol                                 │
│                                                                    │
│ ☐ All trades (manual oversight mode)                             │
│                                                                    │
│ Approvers                                                         │
│ ┌──────────────────────────────────────────────────────────────┐ │
│ │ [Search users...]                                    [+ Add] │ │
│ └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│ Selected Approvers:                                               │
│ • John Smith (john@example.com)              [Remove]            │
│ • Sarah Johnson (sarah@example.com)          [Remove]            │
│                                                                    │
│ Approval Rules                                                    │
│ ○ Any one approver can approve                                   │
│ ● Require 2 approvers for trades > $50,000                       │
│ ○ All approvers must approve                                     │
│                                                                    │
│ Timeout Handling                                                  │
│ If no response within: ┌────┐ hours                              │
│                         │ 24 │                                    │
│                         └────┘                                    │
│ ○ Reject automatically                                           │
│ ● Escalate to admin                                              │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘

Fields:
  - Approval triggers (multi-checkbox)
  - Threshold amount (currency, default $10,000)
  - Risk score threshold (slider, 0-10, default 7.5)
  - Approvers list (searchable, multi-select)
  - Approval rules (radio)
  - Timeout (number, hours, default 24)
  - Timeout action (radio)
```

**Step 5: Review & Launch**

```
┌────────────────────────────────────────────────────────────────────┐
│ Step 5: Review & Launch                                            │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│ ┌────────────────────────────────────────────────────────────────┐│
│ │ Basic Information                               [Edit Step 1] ││
│ │ • Name: Growth Alpha                                          ││
│ │ • Type: Growth                                                ││
│ │ • Description: Long-term growth strategy focused on...       ││
│ └────────────────────────────────────────────────────────────────┘│
│                                                                    │
│ ┌────────────────────────────────────────────────────────────────┐│
│ │ Budget Configuration                            [Edit Step 2] ││
│ │ • Monthly Limit: $50,000                                      ││
│ │ • Budget Warning: 75%                                         ││
│ │ • Daily Loss Limit: $5,000                                    ││
│ │ • Auto-pause: Enabled                                         ││
│ └────────────────────────────────────────────────────────────────┘│
│                                                                    │
│ ┌────────────────────────────────────────────────────────────────┐│
│ │ Trading Parameters                              [Edit Step 3] ││
│ │ • Risk Tolerance: Moderate                                    ││
│ │ • Asset Classes: US Equities, ETFs                           ││
│ │ • Max Position Size: 25% of portfolio                        ││
│ │ • Max Open Positions: 10                                      ││
│ │ • Trading Hours: Market Hours Only                           ││
│ └────────────────────────────────────────────────────────────────┘│
│                                                                    │
│ ┌────────────────────────────────────────────────────────────────┐│
│ │ Oversight Rules                                 [Edit Step 4] ││
│ │ • Approval for trades > $10,000                              ││
│ │ • Approval for high-risk trades (7.5+/10)                    ││
│ │ • Approvers: John Smith, Sarah Johnson                       ││
│ │ • Rule: Require 2 approvers for trades > $50,000             ││
│ └────────────────────────────────────────────────────────────────┘│
│                                                                    │
│ ┌────────────────────────────────────────────────────────────────┐│
│ │ ✓ I confirm these settings are correct                       ││
│ │ ✓ I understand this agent will trade autonomously            ││
│ └────────────────────────────────────────────────────────────────┘│
│                                                                    │
│                      [← Back]  [Launch Agent →]                    │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘

Behavior:
  - Read-only summary of all steps
  - Each section has "Edit" link to jump back
  - Two confirmation checkboxes required
  - "Launch Agent" button (primary, disabled until confirmed)
  - Shows loading spinner during creation
  - Success: Navigate to agent detail page
  - Error: Show error message, allow retry
```

**Wizard Navigation:**
- Progress indicator at top (circles: complete ●, current ●, future ○)
- "Back" button: Save draft, return to previous step
- "Continue" button: Validate and advance (disabled if invalid)
- "✕" close button: Confirm discard (if changes made)
- Keyboard: Enter to continue, Esc to close

**Validation & Error Handling:**
- Inline validation on blur
- Error messages below fields in red
- Required field indicators (*)
- Prevent navigation if current step invalid
- Auto-save draft every 30 seconds
- Resume from draft if browser closed

---

### 4.3 Agent Detail View

**Purpose:** Comprehensive view of individual agent performance, holdings, and activity.

**Layout:** Desktop (1440px+)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ ← Back to Dashboard                      [Notifications] [Profile Menu] │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Growth Alpha                         [●Active]                          │
│  Long-term growth strategy focused on technology leaders                │
│                                                                          │
│  [📊 Analytics]  [⏸ Pause Agent]  [⚙ Settings]  [⋮ More]               │
│                                                                          │
│  ┌──────────────┬──────────────┬──────────────┬──────────────┬─────────┐ │
│  │ Total Value  │ Today's P&L  │ All-Time P&L │ Budget Used  │ Win Rate│ │
│  │              │              │              │              │         │ │
│  │  $127,340    │  +$2,340     │  +$27,340    │  45%         │  67%    │ │
│  │  ↑ 1.87%     │  +1.87%      │  +27.35%     │ ████░░░░░░   │ 12/18   │ │
│  └──────────────┴──────────────┴──────────────┴──────────────┴─────────┘ │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │ Portfolio Performance                           [1D 1W 1M 3M 1Y All]│ │
│  ├─────────────────────────────────────────────────────────────────────┤ │
│  │                                                                     │ │
│  │  $130K  ─                                                  ●        │ │
│  │         │                                            ╱╲    ╱         │ │
│  │  $125K  ─                                      ╱╲  ╱  ╲  ╱          │ │
│  │         │                                ╱╲  ╱  ╲╱    ╲╱            │ │
│  │  $120K  ─                          ╱╲  ╱  ╲╱                        │ │
│  │         │                    ╱╲  ╱  ╲╱                              │ │
│  │  $115K  ─              ╱╲  ╱  ╲╱                                    │ │
│  │         │        ╱╲  ╱  ╲╱                                          │ │
│  │  $110K  ─  ╱╲  ╱  ╲╱                                                │ │
│  │         └──────────────────────────────────────────────────────────│ │
│  │           Jan    Feb    Mar    Apr    May    Jun    Jul    Aug     │ │
│  │                                                                     │ │
│  │  Current: $127,340  ↑ +27.35% since inception                      │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌───────────────────────────────────┬─────────────────────────────────┐ │
│  │ Current Holdings                  │ Pending Approvals        [2]   │ │
│  ├───────────────────────────────────┼─────────────────────────────────┤ │
│  │                                   │                                 │ │
│  │ Symbol  Shares  Value    P&L      │ ┌─────────────────────────────┐ │ │
│  │ ────────────────────────────────  │ │ TSLA - Buy 100 shares       │ │ │
│  │ AAPL    50      $8,750  +$325    │ │ $24,500 at $245.00         │ │ │
│  │ MSFT    30      $10,200 +$780    │ │                             │ │ │
│  │ GOOGL   15      $19,350 -$150    │ │ Reason: High conviction     │ │ │
│  │ NVDA    25      $27,500 +$1,200  │ │ AI growth play              │ │ │
│  │ META    40      $13,200 +$450    │ │                             │ │ │
│  │ TSLA    35      $8,925  +$275    │ │ Risk Score: 8.2/10 ⚠       │ │ │
│  │ AMZN    20      $32,400 +$1,100  │ │ Submitted: 15m ago          │ │ │
│  │ NFLX    12      $7,015  +$85     │ │                             │ │ │
│  │                                   │ │ [Approve] [Reject] [Review] │ │ │
│  │ Total: 8 positions                │ └─────────────────────────────┘ │ │
│  │ [Export CSV] [Rebalance]          │                                 │ │
│  │                                   │ [View All Pending →]            │ │
│  └───────────────────────────────────┴─────────────────────────────────┘ │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │ Recent Trades                                      [View All →]     │ │
│  ├─────────────────────────────────────────────────────────────────────┤ │
│  │                                                                     │ │
│  │ Time     Type  Symbol  Qty   Price      Total      Status          │ │
│  │ ──────────────────────────────────────────────────────────────────  │ │
│  │ 2:34 PM  Buy   AAPL    50    $175.00    $8,750     ✓ Executed      │ │
│  │ 1:15 PM  Sell  MSFT    100   $372.00    $37,200    ✓ Executed      │ │
│  │ 11:42 AM Buy   NVDA    25    $1,100.00  $27,500    ✓ Executed      │ │
│  │ 10:20 AM Buy   META    40    $330.00    $13,200    ✓ Executed      │ │
│  │ 9:35 AM  Sell  AMZN    15    $165.50    $2,482.50  ✓ Executed      │ │
│  │                                                                     │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

**Components:**

1. **Header Section**
   - Breadcrumb: "← Back to Dashboard"
   - Agent Name (H1) + Status Badge
   - Description text (Body)
   - Action buttons: Analytics, Pause, Settings, More menu

2. **Key Metrics Row** (5 columns)
   - Total Value (with change indicator)
   - Today's P&L (colored, with percentage)
   - All-Time P&L (colored, with percentage)
   - Budget Used (percentage + progress bar)
   - Win Rate (percentage + fraction)

3. **Portfolio Performance Chart**
   - Line chart with area fill
   - Time range selector (1D, 1W, 1M, 3M, 1Y, All)
   - Y-axis: Portfolio value
   - X-axis: Time
   - Current value and percentage change below
   - Interactive tooltip on hover
   - Responsive: maintains aspect ratio

4. **Current Holdings Table**
   - Columns: Symbol, Shares, Value, P&L
   - Sortable by any column
   - Color-coded P&L (green/red)
   - Total row at bottom
   - Action buttons: Export CSV, Rebalance
   - Max height with scroll

5. **Pending Approvals Panel**
   - Card-based layout
   - Shows top 2 requests
   - Each card includes:
     - Symbol and action
     - Amount and price
     - Reason (agent's explanation)
     - Risk score with warning if high
     - Timestamp
     - Action buttons: Approve, Reject, Review
   - Badge count in header
   - "View All Pending" link

6. **Recent Trades Table**
   - Columns: Time, Type, Symbol, Qty, Price, Total, Status
   - Latest 5 trades shown
   - Status icons: ✓ (executed), ⏱ (pending), ✗ (rejected)
   - "View All" link to full trade history

**Interactions:**
- Chart: Hover for tooltip with exact values
- Holdings table: Click row to see position details
- Pending approvals: Click card to open approval modal
- Recent trades: Click row to see trade details
- Real-time updates via WebSocket
- Refresh button to force update

**Responsive Behavior:**
- Desktop (1440px+): Side-by-side layout
- Tablet (768px-1439px): Stacked layout, 2-column metrics
- Mobile (< 768px): Single column, scrollable tables

---

### 4.4 Trade Execution Modal

**Purpose:** Modal dialog for executing trades with oversight checks.

**Layout:** Medium modal (600px width)

```
┌──────────────────────────────────────────────────────────────────┐
│ Execute Trade                                              [✕]   │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Agent: Growth Alpha                                             │
│                                                                  │
│  Symbol *                                                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ TSLA                                            [🔍 Search] │ │
│  └────────────────────────────────────────────────────────────┘ │
│  Tesla, Inc. · Last: $245.50 · Change: +2.35 (+0.97%)          │
│                                                                  │
│  Action *                                                        │
│  [● Buy]  [○ Sell]                                              │
│                                                                  │
│  Quantity *                                                      │
│  ┌──────────────────┐                                           │
│  │ 100              │  shares                                   │
│  └──────────────────┘                                           │
│                                                                  │
│  Order Type                                                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Market Order                                         [▼] │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Estimated Total:  $24,550.00                             │   │
│  │                                                          │   │
│  │ Impact on Budget:  45% → 54% (+9%)                       │   │
│  │ ████░░░░░░  →  █████░░░░░                                │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ ⚠ Oversight Required                                     │   │
│  │                                                          │   │
│  │ This trade exceeds the approval threshold ($10,000).    │   │
│  │ It will be submitted for review by:                     │   │
│  │ • John Smith                                             │   │
│  │ • Sarah Johnson                                          │   │
│  │                                                          │   │
│  │ Reason for Trade (required):                            │   │
│  │ ┌────────────────────────────────────────────────────┐  │   │
│  │ │ High conviction AI growth play. Strong earnings   │  │   │
│  │ │ report and positive momentum.                      │  │   │
│  │ └────────────────────────────────────────────────────┘  │   │
│  │                                                          │   │
│  │ Estimated Approval Time: 2-4 hours                      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│                          [Cancel]  [Submit for Approval →]      │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

**Alternative: No Oversight Required**

```
┌──────────────────────────────────────────────────────────────────┐
│ Execute Trade                                              [✕]   │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Agent: Growth Alpha                                             │
│                                                                  │
│  Symbol *                                                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ AAPL                                            [🔍 Search] │ │
│  └────────────────────────────────────────────────────────────┘ │
│  Apple Inc. · Last: $175.25 · Change: -1.15 (-0.65%)           │
│                                                                  │
│  Action *                                                        │
│  [● Buy]  [○ Sell]                                              │
│                                                                  │
│  Quantity *                                                      │
│  ┌──────────────────┐                                           │
│  │ 50               │  shares                                   │
│  └──────────────────┘                                           │
│                                                                  │
│  Order Type                                                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Market Order                                         [▼] │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Estimated Total:  $8,762.50                              │   │
│  │                                                          │   │
│  │ Impact on Budget:  45% → 47% (+2%)                       │   │
│  │ ████░░░░░░  →  ████░░░░░░                                │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ ✓ Ready to Execute                                       │   │
│  │                                                          │   │
│  │ This trade is within your autonomous limits and will    │   │
│  │ execute immediately.                                     │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│                          [Cancel]  [Execute Trade →]            │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

**Confirmation Step (after clicking Execute)**

```
┌──────────────────────────────────────────────────────────────────┐
│ Confirm Trade Execution                                    [✕]   │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  You are about to execute the following trade:                  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Action:     Buy                                          │   │
│  │ Symbol:     AAPL (Apple Inc.)                            │   │
│  │ Quantity:   50 shares                                    │   │
│  │ Order Type: Market Order                                 │   │
│  │ Estimated:  $8,762.50                                    │   │
│  │                                                          │   │
│  │ Agent:      Growth Alpha                                 │   │
│  │ Budget:     45% → 47% (+$8,762.50)                       │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ⚠ Market orders execute at the current market price.           │
│  Final price may differ from estimate.                          │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ ☑ I understand this trade will execute immediately      │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│                          [Go Back]  [Confirm & Execute →]       │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

**Components:**

1. **Symbol Search**
   - Autocomplete input
   - Search by symbol or company name
   - Shows: Company name, last price, change
   - Real-time quote data
   - Dropdown with suggestions

2. **Action Toggle**
   - Radio buttons: Buy / Sell
   - Large, clear selection
   - Disabled if no holdings (for sell)

3. **Quantity Input**
   - Number input
   - Validation: > 0, integer
   - Shows unit (shares)
   - Max quantity for sell (current holdings)

4. **Order Type Dropdown**
   - Market Order (default)
   - Limit Order (optional)
   - Stop Loss (optional)
   - Conditional fields based on selection

5. **Estimated Total**
   - Calculated: Quantity × Current Price
   - Updates in real-time
   - Shows budget impact (before/after)
   - Visual progress bar

6. **Oversight Warning Panel** (conditional)
   - Yellow background (#FEF3C7)
   - Warning icon
   - Explains why approval needed
   - Lists approvers
   - Required reason textarea (200 chars max)
   - Estimated approval time

7. **Ready to Execute Panel** (conditional)
   - Green background (#D1FAE5)
   - Checkmark icon
   - Confirms immediate execution

8. **Action Buttons**
   - Cancel: Secondary button
   - Submit/Execute: Primary button
   - Disabled until valid

**Validation:**
- Symbol required and valid
- Quantity > 0
- Sell quantity ≤ current holdings
- Reason required if oversight needed
- Confirmation checkbox required

**Error Handling:**
- Invalid symbol: "Symbol not found"
- Insufficient budget: "Exceeds monthly limit"
- Duplicate: "Trade already pending for this symbol"
- API error: "Unable to submit trade. Please try again."

**Success Flow:**
1. Submit trade → Loading spinner
2. Success message: "Trade submitted successfully"
3. Redirect to agent detail page
4. Show toast notification
5. Real-time update in holdings/pending

---

### 4.5 Oversight Approval View

**Purpose:** Interface for reviewing and approving/rejecting pending trade requests.

**Layout:** Desktop (1440px+)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ ← Back                                       [Notifications] [Profile]   │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Oversight & Approvals                                                   │
│                                                                          │
│  [Pending (3)]  [Approved]  [Rejected]  [All History]                   │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ ⚠ High Priority Request                            Submitted 15m ago│ │
│  ├────────────────────────────────────────────────────────────────────┤ │
│  │                                                                    │ │
│  │  Growth Alpha wants to Buy 100 shares of TSLA                      │ │
│  │  Estimated Total: $24,550.00 at $245.50/share                     │ │
│  │                                                                    │ │
│  │  ┌──────────────────────┬───────────────────────────────────────┐ │ │
│  │  │ Request Details      │ Agent Context                         │ │ │
│  │  ├──────────────────────┼───────────────────────────────────────┤ │ │
│  │  │                      │                                       │ │ │
│  │  │ Symbol: TSLA         │ Agent Status: ●Active                 │ │ │
│  │  │ Tesla, Inc.          │                                       │ │ │
│  │  │                      │ Current Portfolio: $127,340           │ │ │
│  │  │ Action: Buy          │ Today's P&L: +$2,340 (+1.87%)        │ │ │
│  │  │ Quantity: 100 shares │ All-Time P&L: +$27,340 (+27.35%)     │ │ │
│  │  │ Type: Market Order   │                                       │ │ │
│  │  │                      │ Budget Status:                        │ │ │
│  │  │ Current Price:       │ Current: 45% ($22,500/$50,000)       │ │ │
│  │  │ $245.50              │ After Trade: 54% ($27,050/$50,000)   │ │ │
│  │  │ Change: +2.35 ↑      │ ████░░░░░░ → █████░░░░░              │ │ │
│  │  │                      │                                       │ │ │
│  │  │ Estimated Total:     │ Win Rate: 67% (12/18 trades)         │ │ │
│  │  │ $24,550.00           │                                       │ │ │
│  │  │                      │ Open Positions: 8/10                  │ │ │
│  │  │ Risk Score:          │ Largest Position: AMZN (25%)         │ │ │
│  │  │ 8.2/10 ⚠ High       │                                       │ │ │
│  │  │                      │ Recent Performance (7d):              │ │ │
│  │  │ Reason:              │   ▁▂▃▅▆▇█ +4.2%                      │ │ │
│  │  │ "High conviction AI  │                                       │ │ │
│  │  │ growth play. Strong  │                                       │ │ │
│  │  │ earnings report and  │                                       │ │ │
│  │  │ positive momentum."  │                                       │ │ │
│  │  │                      │                                       │ │ │
│  │  └──────────────────────┴───────────────────────────────────────┘ │ │
│  │                                                                    │ │
│  │  ┌──────────────────────────────────────────────────────────────┐ │ │
│  │  │ Recent Trades (Last 5)                                       │ │ │
│  │  ├──────────────────────────────────────────────────────────────┤ │ │
│  │  │ Time     Symbol  Action  Qty   P&L       Status              │ │ │
│  │  │ 2:34 PM  AAPL    Buy     50    +$325     ✓ Executed         │ │ │
│  │  │ 1:15 PM  MSFT    Sell    100   +$780     ✓ Executed         │ │ │
│  │  │ 11:42 AM NVDA    Buy     25    +$1,200   ✓ Executed         │ │ │
│  │  │ 10:20 AM META    Buy     40    +$450     ✓ Executed         │ │ │
│  │  │ 9:35 AM  AMZN    Sell    15    +$1,100   ✓ Executed         │ │ │
│  │  └──────────────────────────────────────────────────────────────┘ │ │
│  │                                                                    │ │
│  │  ┌──────────────────────────────────────────────────────────────┐ │ │
│  │  │ Your Decision                                                │ │ │
│  │  ├──────────────────────────────────────────────────────────────┤ │ │
│  │  │                                                              │ │ │
│  │  │ ○ Approve    ○ Reject    ○ Request More Info                │ │ │
│  │  │                                                              │ │ │
│  │  │ Comments (optional):                                         │ │ │
│  │  │ ┌──────────────────────────────────────────────────────────┐│ │ │
│  │  │ │                                                          ││ │ │
│  │  │ │                                                          ││ │ │
│  │  │ └──────────────────────────────────────────────────────────┘│ │ │
│  │  │                                                              │ │ │
│  │  │ If rejecting, reason is required.                           │ │ │
│  │  │                                                              │ │ │
│  │  │                             [Cancel]  [Submit Decision →]   │ │ │
│  │  └──────────────────────────────────────────────────────────────┘ │ │
│  │                                                                    │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ Standard Priority Request                          Submitted 2h ago│ │
│  ├────────────────────────────────────────────────────────────────────┤ │
│  │                                                                    │ │
│  │  Value Hunter wants to Buy 200 shares of DIS                      │ │
│  │  Estimated Total: $18,400.00 at $92.00/share                      │ │
│  │                                                                    │ │
│  │  Risk Score: 5.3/10   Budget Impact: 23% → 30%                    │ │
│  │                                                                    │ │
│  │  [Expand Details ▼]                                               │ │
│  │                                                                    │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ Standard Priority Request                          Submitted 4h ago│ │
│  ├────────────────────────────────────────────────────────────────────┤ │
│  │                                                                    │ │
│  │  Momentum Scout wants to Sell 75 shares of NFLX                   │ │
│  │  Estimated Total: $43,875.00 at $585.00/share                     │ │
│  │                                                                    │ │
│  │  Risk Score: 3.8/10   Budget Impact: 67% → 59%                    │ │
│  │                                                                    │ │
│  │  [Expand Details ▼]                                               │ │
│  │                                                                    │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

**Components:**

1. **Tab Navigation**
   - Pending (with count badge)
   - Approved
   - Rejected
   - All History
   - Active tab highlighted

2. **Approval Request Card** (Expanded)
   - Priority indicator (High/Standard)
   - Timestamp (relative: "15m ago", "2h ago")
   - Summary line: Agent + Action + Symbol + Quantity
   - Estimated total

3. **Request Details Section**
   - Symbol info with current price
   - Action, Quantity, Order Type
   - Estimated total
   - Risk Score (colored by level)
   - Agent's reason (quoted)

4. **Agent Context Section**
   - Current status badge
   - Portfolio value
   - Today's and All-Time P&L
   - Budget status (before/after bars)
   - Win rate and fraction
   - Open positions count
   - Largest position warning
   - Recent performance sparkline

5. **Recent Trades Table**
   - Last 5 trades for context
   - Shows pattern of agent behavior
   - P&L to assess performance

6. **Decision Panel**
   - Radio buttons: Approve / Reject / Request More Info
   - Comments textarea (optional for approve, required for reject)
   - Submit button (validates based on selection)

7. **Collapsed Request Cards**
   - Compact view for other pending requests
   - Shows key info: Agent, action, amount
   - Risk score and budget impact
   - "Expand Details" button

**Approval Workflow:**

**Approve:**
1. Select "Approve" radio
2. Optional comment
3. Click "Submit Decision"
4. Confirmation: "Trade approved and will execute shortly"
5. Real-time notification to agent
6. Card moves to "Approved" tab

**Reject:**
1. Select "Reject" radio
2. Required reason in comments
3. Click "Submit Decision"
4. Confirmation: "Trade request rejected"
5. Notification to agent with reason
6. Card moves to "Rejected" tab

**Request More Info:**
1. Select "Request More Info" radio
2. Required questions/comments
3. Click "Submit Decision"
4. Notification to agent
5. Card remains in "Pending" with "Info Requested" badge
6. Agent can respond, then re-submit

**Notifications:**
- Email notification for new requests
- In-app notification badge
- Push notification (if enabled)
- Slack integration (optional)
- Escalation after timeout (24h default)

**Filters & Search:**
- Filter by agent
- Filter by risk score range
- Filter by amount range
- Search by symbol
- Sort by: Time, Amount, Risk Score

---

### 4.6 Metering Dashboard

**Purpose:** Monitor API usage, quota consumption, and cost metrics for the trading system.

**Layout:** Desktop (1440px+)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ ← Back to Dashboard                      [Notifications] [Profile Menu] │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  API Metering & Usage                                                    │
│                                                                          │
│  [Real-time]  [Last 24h]  [Last 7d]  [Last 30d]  [Custom Range]        │
│                                                                          │
│  ┌──────────────┬──────────────┬──────────────┬──────────────────────┐  │
│  │ Total Calls  │ Success Rate │ Avg Latency  │ Current Cost         │  │
│  │              │              │              │                      │  │
│  │  12,450      │  99.2%       │  142ms       │  $124.50             │  │
│  │  ↑ 8% vs 24h │  ↑ 0.1%      │  ↓ 12ms      │  ↑ $12.40 vs 24h     │  │
│  └──────────────┴──────────────┴──────────────┴──────────────────────┘  │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │ API Call Volume                              [1h 6h 24h 7d 30d All]│ │
│  ├─────────────────────────────────────────────────────────────────────┤ │
│  │                                                                     │ │
│  │  600  ─                                                             │ │
│  │       │                                                      ▅      │ │
│  │  500  ─                                               ▃     █       │ │
│  │       │                                        ▂     ▆█    ██       │ │
│  │  400  ─                                 ▁     ▄█   ▇██   ███       │ │
│  │       │                          ▂     ▃█    ███  ████  ████       │ │
│  │  300  ─                   ▁     ▅█   ▆███  █████ █████ █████       │ │
│  │       │            ▂     ▄█    ████ ██████████████████████████     │ │
│  │  200  ─     ▁     ▆█   ████  ███████████████████████████████████   │ │
│  │       │    ███  ████████████████████████████████████████████████   │ │
│  │  100  ─  ████████████████████████████████████████████████████████  │ │
│  │       └──────────────────────────────────────────────────────────  │ │
│  │         12am  3am  6am  9am  12pm  3pm  6pm  9pm  12am             │ │
│  │                                                                     │ │
│  │  Peak: 582 calls/hour at 2:00 PM                                   │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌───────────────────────────────────┬─────────────────────────────────┐ │
│  │ Usage by Category                 │ Quota Status                    │ │
│  ├───────────────────────────────────┼─────────────────────────────────┤ │
│  │                                   │                                 │ │
│  │ Category          Calls     %     │ ┌─────────────────────────────┐ │ │
│  │ ───────────────────────────────   │ │ Monthly Quota               │ │ │
│  │ Market Data       8,234    66.1%  │ │ 78,450 / 100,000 calls      │ │ │
│  │ Trade Execution   2,104    16.9%  │ │                             │ │ │
│  │ Portfolio Sync    1,456    11.7%  │ │ ███████░░░ 78%              │ │ │
│  │ Agent Decisions     432     3.5%  │ │                             │ │ │
│  │ Oversight/Approvals 224     1.8%  │ │ Resets in: 12 days          │ │ │
│  │                                   │ └─────────────────────────────┘ │ │
│  │ ┌───────────────────────────────┐ │                                 │ │
│  │ │      ███ 66%                 │ │ ┌─────────────────────────────┐ │ │
│  │ │      ██ 17%                  │ │ │ Daily Rate Limit            │ │ │
│  │ │      ██ 12%                  │ │ │ 2,340 / 5,000 calls/day     │ │ │
│  │ │      ░ 4%                    │ │ │                             │ │ │
│  │ │      ░ 2%                    │ │ │ ████░░░░░░ 47%              │ │ │
│  │ └───────────────────────────────┘ │ │                             │ │ │
│  │                                   │ │ Safe buffer: 2,660 remaining│ │ │
│  │ [Export Report]                   │ └─────────────────────────────┘ │ │
│  │                                   │                                 │ │
│  │                                   │ ⚠ Cost Alert Thresholds         │ │
│  │                                   │                                 │ │
│  │                                   │ Daily:   $50    [Edit]          │ │
│  │                                   │ Monthly: $500   [Edit]          │ │
│  │                                   │                                 │ │
│  │                                   │ Current Pace: $124.50/month ✓   │ │
│  │                                   │                                 │ │
│  └───────────────────────────────────┴─────────────────────────────────┘ │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │ Cost Breakdown                                       [View Details →]│ │
│  ├─────────────────────────────────────────────────────────────────────┤ │
│  │                                                                     │ │
│  │ Service               Calls    Cost      Avg Cost/Call             │ │
│  │ ─────────────────────────────────────────────────────────────────  │ │
│  │ Real-time Quotes      8,234   $82.34     $0.010                    │ │
│  │ Trade Execution API   2,104   $31.56     $0.015                    │ │
│  │ Portfolio Analytics   1,456   $7.28      $0.005                    │ │
│  │ AI Agent Inference      432   $2.16      $0.005                    │ │
│  │ Oversight Webhooks      224   $1.16      $0.005                    │ │
│  │                                                                     │ │
│  │ Total:               12,450   $124.50    $0.010 avg                │ │
│  │                                                                     │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │ Recent Errors                                        [View All →]   │ │
│  ├─────────────────────────────────────────────────────────────────────┤ │
│  │                                                                     │ │
│  │ Time     Service             Error Code   Message                  │ │
│  │ ──────────────────────────────────────────────────────────────────  │ │
│  │ 2:45 PM  Trade Execution     429          Rate limit exceeded      │ │
│  │ 1:23 PM  Market Data         503          Service unavailable      │ │
│  │ 11:15 AM Portfolio Analytics 401          Invalid API key          │ │
│  │                                                                     │ │
│  │ Error Rate: 0.8% (99 errors / 12,450 calls)                        │ │
│  │                                                                     │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │ Performance Metrics                                                 │ │
│  ├─────────────────────────────────────────────────────────────────────┤ │
│  │                                                                     │ │
│  │ Latency Distribution (p50, p95, p99):                              │ │
│  │                                                                     │ │
│  │ Market Data:       98ms   ██████████                               │ │
│  │                   142ms   ███████████████                          │ │
│  │                   234ms   ████████████████████████                 │ │
│  │                                                                     │ │
│  │ Trade Execution:   156ms  ████████████████                         │ │
│  │                   289ms   ██████████████████████████████           │ │
│  │                   445ms   █████████████████████████████████████████ │ │
│  │                                                                     │ │
│  │ AI Inference:      203ms  ████████████████████                     │ │
│  │                   412ms   ███████████████████████████████████████  │ │
│  │                   678ms   ████████████████████████████████████████ │ │
│  │                                                                     │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

**Components:**

1. **Time Range Selector**
   - Tabs: Real-time, Last 24h, Last 7d, Last 30d, Custom Range
   - Active tab highlighted
   - Custom range opens date picker modal

2. **Summary Metrics** (4 columns)
   - Total Calls (with trend)
   - Success Rate (percentage)
   - Average Latency (ms)
   - Current Cost (dollar amount with trend)

3. **API Call Volume Chart**
   - Bar chart showing calls over time
   - X-axis: Time (granularity based on range)
   - Y-axis: Call count
   - Hover: Exact count and timestamp
   - Peak indicator below chart

4. **Usage by Category**
   - Table: Category, Calls, Percentage
   - Sorted by call count (descending)
   - Horizontal bar chart visualization
   - Export button for CSV

5. **Quota Status Panel**
   - Monthly Quota: Progress bar with used/total
   - Days until reset
   - Daily Rate Limit: Progress bar with used/total
   - Remaining buffer calculation
   - Visual warning if > 90% used

6. **Cost Alert Thresholds**
   - Daily threshold (editable)
   - Monthly threshold (editable)
   - Current pace calculation
   - Check/warning icon based on status

7. **Cost Breakdown Table**
   - Columns: Service, Calls, Cost, Avg Cost/Call
   - Sorted by cost (descending)
   - Total row at bottom
   - Link to detailed cost analysis

8. **Recent Errors Table**
   - Columns: Time, Service, Error Code, Message
   - Last 5 errors shown
   - Error rate calculation
   - Link to full error log

9. **Performance Metrics**
   - Latency distribution (p50, p95, p99)
   - Horizontal bar charts
   - Grouped by service category
   - Color-coded by latency (green < 200ms, yellow < 500ms, red > 500ms)

**Interactions:**
- Charts: Hover for tooltips
- Time range: Update all data on selection
- Thresholds: Click "Edit" to modify, save
- Export: Download CSV report
- Real-time: Auto-refresh every 30 seconds
- Errors: Click row to see full error details

**Alerts Configuration Modal:**

```
┌──────────────────────────────────────────────────────────────────┐
│ Configure Alerts                                           [✕]   │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Cost Alerts                                                     │
│                                                                  │
│  Daily Threshold                                                 │
│  ┌────────────────┐                                             │
│  │ $ 50.00        │  per day                                    │
│  └────────────────┘                                             │
│                                                                  │
│  Monthly Threshold                                               │
│  ┌────────────────┐                                             │
│  │ $ 500.00       │  per month                                  │
│  └────────────────┘                                             │
│                                                                  │
│  Usage Alerts                                                    │
│                                                                  │
│  ☑ Notify when quota reaches 80%                                │
│  ☑ Notify when daily rate limit reaches 90%                     │
│  ☑ Notify on sustained high error rate (> 5%)                   │
│                                                                  │
│  Notification Channels                                           │
│                                                                  │
│  ☑ Email (user@example.com)                                     │
│  ☑ In-app notifications                                         │
│  ☐ Slack (#alerts channel)                                      │
│  ☐ SMS (+1 555-0123)                                            │
│                                                                  │
│                             [Cancel]  [Save Settings →]         │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 5. Responsive Design

### 5.1 Breakpoints

```
Mobile:      < 768px   (iPhone, Android phones)
Tablet:      768px - 1439px   (iPad, tablets)
Desktop:     1440px+   (laptops, desktops)
Wide:        1920px+   (large monitors)
```

### 5.2 Mobile Adaptations

**Dashboard:**
- Single column layout
- Summary cards stacked vertically
- Agent cards full-width
- Collapsible sections
- Bottom navigation bar
- Swipe gestures for cards

**Agent Creation Wizard:**
- Full-screen modal
- Step indicator at top
- Larger tap targets (min 44×44px)
- Auto-save progress
- Exit confirmation

**Agent Detail:**
- Stacked sections
- Horizontal scroll for tables
- Collapsible holdings
- Fixed header with agent name
- Floating action button

**Trade Execution:**
- Full-screen modal
- Large input fields
- Symbol search full-width
- Buy/Sell buttons full-width
- Sticky footer buttons

**Oversight Approval:**
- Single column cards
- Expand/collapse details
- Swipe to approve/reject (optional)
- Sticky decision panel at bottom

**Metering:**
- Stacked metrics
- Simplified charts
- Collapsible sections
- Horizontal scroll for tables

---

## 6. Accessibility Requirements

### 6.1 WCAG 2.1 AA Compliance

**Color Contrast:**
- Text on background: 4.5:1 minimum
- Large text (18pt+): 3:1 minimum
- UI components: 3:1 minimum
- Use color + icon/text (not color alone)

**Keyboard Navigation:**
- All interactive elements focusable
- Logical tab order
- Visible focus indicators
- Skip to main content link
- Escape to close modals
- Arrow keys for navigation

**Screen Reader Support:**
- Semantic HTML5 elements
- ARIA labels where needed
- ARIA live regions for updates
- Alt text for all images
- Form labels properly associated
- Table headers properly marked
- Landmark regions defined

**Focus Management:**
- Trap focus in modals
- Return focus after modal close
- Focus first error on validation
- Announce dynamic content changes

**Error Handling:**
- Clear error messages
- Error summary at top of form
- Inline error indicators
- Suggestions for correction

### 6.2 ARIA Labels

```html
<!-- Dashboard -->
<main role="main" aria-label="Dashboard">
  <section aria-labelledby="summary-heading">
    <h2 id="summary-heading">Summary</h2>
  </section>

  <nav aria-label="Quick actions">
    <button aria-label="Create new agent">+ Create Agent</button>
  </nav>
</main>

<!-- Agent Card -->
<article aria-label="Growth Alpha agent card">
  <h3>Growth Alpha</h3>
  <span role="status" aria-label="Agent status">Active</span>
  <div role="meter" aria-valuenow="45" aria-valuemin="0" aria-valuemax="100" aria-label="Budget usage: 45%">
    <div style="width: 45%"></div>
  </div>
</article>

<!-- Modal -->
<div role="dialog" aria-labelledby="modal-title" aria-modal="true">
  <h2 id="modal-title">Execute Trade</h2>
  <button aria-label="Close dialog">×</button>
</div>

<!-- Form -->
<form aria-labelledby="agent-form-title">
  <label for="agent-name">Agent Name *</label>
  <input id="agent-name" type="text" required aria-required="true" aria-describedby="name-help">
  <span id="name-help">Maximum 50 characters</span>
  <span role="alert" aria-live="assertive" id="name-error"></span>
</form>

<!-- Live Updates -->
<div aria-live="polite" aria-atomic="true">
  New trade executed: Growth Alpha bought 50 shares of AAPL
</div>
```

### 6.3 Keyboard Shortcuts

```
Global:
  ? - Show keyboard shortcuts help
  / - Focus search
  Esc - Close modal/dialog

Dashboard:
  C - Create new agent
  A - View all agents
  O - Oversight center
  M - Metering

Agent Detail:
  E - Execute trade
  P - Pause/Resume agent
  S - Settings

Modals:
  Tab - Next field
  Shift+Tab - Previous field
  Enter - Submit (when focused on button)
  Esc - Cancel/Close

Tables:
  Arrow keys - Navigate cells
  Space - Select/toggle
  Enter - Activate row action
```

---

## 7. Animation & Transitions

### 7.1 Timing Functions

```
Ease-in-out: cubic-bezier(0.4, 0, 0.2, 1)  (default)
Ease-out: cubic-bezier(0, 0, 0.2, 1)       (entering)
Ease-in: cubic-bezier(0.4, 0, 1, 1)        (exiting)
```

### 7.2 Durations

```
Fast:     150ms   (hover, focus states)
Normal:   250ms   (transitions, fades)
Slow:     350ms   (modals, drawers)
```

### 7.3 Common Transitions

```css
/* Button hover */
.button {
  transition: background-color 150ms ease-in-out, transform 150ms ease-in-out;
}

/* Modal entrance */
.modal {
  animation: modal-enter 250ms ease-out;
}

@keyframes modal-enter {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

/* Toast notification */
.toast {
  animation: toast-slide-in 250ms ease-out;
}

@keyframes toast-slide-in {
  from {
    transform: translateY(-100%);
  }
  to {
    transform: translateY(0);
  }
}

/* Loading spinner */
.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Progress bar fill */
.progress-bar {
  transition: width 350ms ease-out;
}

/* Real-time data update */
.data-update {
  animation: pulse 500ms ease-in-out;
}

@keyframes pulse {
  0%, 100% { background-color: transparent; }
  50% { background-color: rgba(59, 130, 246, 0.1); }
}
```

---

## 8. Data Visualization Guidelines

### 8.1 Charts

**Line Charts:**
- Stroke width: 2px
- Data points: 6px diameter circles on hover
- Grid lines: Gray 200, 1px, dashed
- Area fill: Primary color with 10% opacity
- Tooltip: White background, shadow-md

**Bar Charts:**
- Bar width: Auto-calculated with 20% gap
- Rounded corners: 4px top
- Hover: Brightness increase 110%
- Labels: Below bars (X-axis) or inside (Y-axis)

**Sparklines:**
- Stroke width: 1.5px
- No axes or labels
- Height: 40px
- Width: 100px
- Stroke: Current text color

**Progress Bars:**
- Height: 8px
- Border radius: 4px (full)
- Background: Gray 200
- Fill: Color based on value
  - < 50%: Success Green
  - 50-80%: Warning Amber
  - > 80%: Danger Red

### 8.2 Color Coding

**P&L Values:**
- Positive: Success Green (#10B981)
- Negative: Danger Red (#EF4444)
- Zero: Neutral Gray (#6B7280)

**Risk Scores (0-10):**
- 0-3: Green (Low)
- 4-6: Yellow (Medium)
- 7-8: Orange (High)
- 9-10: Red (Critical)

**Status Indicators:**
- Active: Green dot
- Paused: Amber dot
- Error: Red dot
- Pending: Gray dot

---

## 9. Loading & Empty States

### 9.1 Loading States

**Skeleton Screens:**
- Use for initial page load
- Animated gradient shimmer
- Match layout of actual content
- Gray 200 background

```
┌────────────────────────────────────┐
│ ▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░░░ │
│                                    │
│ ▓▓▓▓░░░░  ▓▓▓▓░░░░  ▓▓▓▓░░░░      │
│                                    │
│ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░   │
│ ▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░░░    │
└────────────────────────────────────┘
```

**Spinner:**
- Size: 24px (small), 40px (medium), 64px (large)
- Color: Primary Blue
- Use for: Button actions, inline loading

**Progress Bar:**
- Use when progress is measurable
- Show percentage
- Estimated time remaining

### 9.2 Empty States

**No Agents:**
```
┌────────────────────────────────────┐
│           [📊 Large Icon]          │
│                                    │
│      No Agents Created Yet         │
│                                    │
│  Create your first AI trading      │
│  agent to get started.             │
│                                    │
│       [+ Create Agent]             │
└────────────────────────────────────┘
```

**No Trades:**
```
┌────────────────────────────────────┐
│           [📈 Icon]                │
│                                    │
│      No Trades Yet                 │
│                                    │
│  This agent hasn't executed any    │
│  trades. Trades will appear here.  │
└────────────────────────────────────┘
```

**No Pending Approvals:**
```
┌────────────────────────────────────┐
│           [✓ Icon]                 │
│                                    │
│   All Caught Up!                   │
│                                    │
│  No pending approval requests.     │
└────────────────────────────────────┘
```

---

## 10. Error Handling

### 10.1 Error Message Patterns

**Inline Validation:**
```
Agent Name *
┌────────────────────────────────────┐
│ Growth Alpha 123                   │
└────────────────────────────────────┘
⚠ Agent name already exists. Please choose a different name.
```

**Form-level Error:**
```
┌────────────────────────────────────┐
│ ⚠ Please correct the following:    │
│                                    │
│ • Agent name is required           │
│ • Budget must be at least $1,000   │
│ • At least one approver required   │
└────────────────────────────────────┘
```

**API Error:**
```
┌────────────────────────────────────┐
│ ✗ Unable to Execute Trade          │
│                                    │
│ The market data service is         │
│ temporarily unavailable.           │
│                                    │
│ [Retry]  [Cancel]                  │
└────────────────────────────────────┘
```

**Network Error:**
```
┌────────────────────────────────────┐
│ ⚠ Connection Lost                  │
│                                    │
│ Trying to reconnect...             │
│ [●○○] Attempt 2 of 3               │
└────────────────────────────────────┘
```

### 10.2 Toast Notifications

**Success:**
- Green background (#D1FAE5)
- Green border (#10B981)
- Checkmark icon
- Auto-dismiss: 3 seconds

**Error:**
- Red background (#FEE2E2)
- Red border (#EF4444)
- X icon
- Auto-dismiss: 5 seconds (or manual close)

**Warning:**
- Amber background (#FEF3C7)
- Amber border (#F59E0B)
- Warning icon
- Auto-dismiss: 4 seconds

**Info:**
- Blue background (#DBEAFE)
- Blue border (#3B82F6)
- Info icon
- Auto-dismiss: 3 seconds

**Position:** Top-right corner, stacked vertically

---

## 11. Performance Considerations

### 11.1 Optimization Strategies

**Lazy Loading:**
- Charts: Load only when in viewport
- Images: Progressive loading
- Modals: Load content on open
- Tables: Virtual scrolling for > 100 rows

**Caching:**
- Agent list: 30 seconds
- Market data: 5 seconds (real-time)
- Historical data: 5 minutes
- User preferences: Session storage

**Debouncing:**
- Search input: 300ms
- Auto-save: 2 seconds
- Resize events: 150ms

**Throttling:**
- Scroll events: 16ms (60fps)
- Real-time updates: 1 second

### 11.2 Bundle Size Targets

```
Initial Load:    < 150KB (gzipped)
Per Route:       < 50KB (gzipped)
Critical CSS:    < 14KB (inline)
JavaScript:      < 200KB (total, gzipped)
```

### 11.3 Perceived Performance

- Show skeleton screens immediately
- Optimistic UI updates
- Inline critical CSS
- Preload fonts
- Defer non-critical scripts
- Service worker for offline support

---

## 12. Browser Support

### 12.1 Target Browsers

**Desktop:**
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

**Mobile:**
- iOS Safari 14+
- Chrome Mobile 90+
- Samsung Internet 14+

### 12.2 Progressive Enhancement

**Core Experience (all browsers):**
- View agents and portfolios
- Execute trades (with approvals)
- View metering data

**Enhanced Experience (modern browsers):**
- Real-time chart updates
- WebSocket notifications
- Offline mode
- Push notifications

**Polyfills Required:**
- fetch (for older browsers)
- IntersectionObserver (lazy loading)
- ResizeObserver (responsive charts)

---

## 13. Implementation Notes

### 13.1 Technology Stack Recommendations

**Frontend:**
- React 18+ (with TypeScript)
- Next.js (for SSR/routing)
- TailwindCSS (styling)
- Recharts or Victory (charts)
- React Query (data fetching)
- Zustand (state management)

**Real-time:**
- Socket.io or WebSocket API
- Server-Sent Events (SSE) for one-way updates

**Forms:**
- React Hook Form (validation)
- Zod (schema validation)

**Accessibility:**
- @react-aria (accessible components)
- eslint-plugin-jsx-a11y (linting)

### 13.2 Component Structure

```
components/
├── layout/
│   ├── Navigation.tsx
│   ├── Header.tsx
│   └── Footer.tsx
├── dashboard/
│   ├── SummaryCards.tsx
│   ├── AgentCard.tsx
│   ├── ActivityFeed.tsx
│   └── MeteringPanel.tsx
├── agent/
│   ├── AgentDetail.tsx
│   ├── PortfolioChart.tsx
│   ├── HoldingsTable.tsx
│   ├── PendingApprovals.tsx
│   └── TradeHistory.tsx
├── trade/
│   ├── TradeExecutionModal.tsx
│   ├── SymbolSearch.tsx
│   └── OrderForm.tsx
├── oversight/
│   ├── ApprovalCard.tsx
│   ├── ApprovalList.tsx
│   └── DecisionPanel.tsx
├── metering/
│   ├── UsageChart.tsx
│   ├── QuotaStatus.tsx
│   └── CostBreakdown.tsx
├── wizard/
│   ├── AgentWizard.tsx
│   ├── StepIndicator.tsx
│   ├── BasicInfoStep.tsx
│   ├── BudgetStep.tsx
│   ├── TradingParamsStep.tsx
│   ├── OversightStep.tsx
│   └── ReviewStep.tsx
└── shared/
    ├── Button.tsx
    ├── Input.tsx
    ├── Select.tsx
    ├── Modal.tsx
    ├── Card.tsx
    ├── Table.tsx
    ├── Badge.tsx
    ├── ProgressBar.tsx
    ├── Toast.tsx
    └── Spinner.tsx
```

### 13.3 State Management

**Global State:**
- User authentication
- Active agents list
- Pending approvals count
- WebSocket connection status

**Local State:**
- Form inputs
- Modal open/close
- Chart time range selection
- Table sorting/filtering

**Server State (React Query):**
- Agent details
- Portfolio data
- Trade history
- Metering data

### 13.4 API Integration

**Endpoints:**
```
GET    /api/agents
POST   /api/agents
GET    /api/agents/:id
PATCH  /api/agents/:id
DELETE /api/agents/:id

GET    /api/agents/:id/portfolio
GET    /api/agents/:id/trades
POST   /api/agents/:id/trades

GET    /api/approvals
GET    /api/approvals/:id
POST   /api/approvals/:id/approve
POST   /api/approvals/:id/reject

GET    /api/metering/usage
GET    /api/metering/cost
GET    /api/metering/quota

WebSocket: /ws/notifications
```

**Error Handling:**
- 401: Redirect to login
- 403: Show permission error
- 404: Show not found state
- 429: Show rate limit message
- 500: Show generic error, retry button

---

## 14. Security Considerations

### 14.1 Authentication & Authorization

- JWT tokens with refresh mechanism
- Role-based access control (RBAC)
- Approve/reject permissions per user
- Session timeout: 30 minutes idle
- Two-factor authentication (optional)

### 14.2 Data Protection

- HTTPS only (enforce)
- CSP headers (Content Security Policy)
- XSS protection (sanitize inputs)
- CSRF tokens for mutations
- Rate limiting on API endpoints

### 14.3 Sensitive Data

- Mask API keys in UI
- Redact sensitive fields in logs
- Encrypt trade details in transit
- Audit trail for all approvals

---

## 15. Testing Requirements

### 15.1 Unit Tests

- All form validations
- Utility functions (formatting, calculations)
- Component rendering (snapshot tests)
- State management logic

### 15.2 Integration Tests

- Agent creation workflow
- Trade execution flow
- Approval process
- Real-time updates

### 15.3 E2E Tests

- Critical user journeys:
  - Create agent → Execute trade → Approve trade
  - View dashboard → Navigate to agent detail
  - Configure alerts → Receive notification

### 15.4 Accessibility Tests

- axe-core automated tests
- Keyboard navigation testing
- Screen reader testing (NVDA, JAWS, VoiceOver)
- Color contrast validation

### 15.5 Performance Tests

- Lighthouse CI (score > 90)
- Bundle size monitoring
- Real-time update performance
- Table rendering with 1000+ rows

---

## 16. Conclusion

This UI specification provides a comprehensive blueprint for building the Trading Demo application. The design prioritizes:

1. **Clarity**: Financial data is presented clearly with appropriate visual hierarchy
2. **Control**: Users maintain oversight through approval workflows and budget controls
3. **Transparency**: All agent actions are visible and traceable
4. **Performance**: Real-time updates without sacrificing user experience
5. **Accessibility**: WCAG 2.1 AA compliance ensures usability for all users
6. **Responsiveness**: Adaptive layouts work across devices

**Next Steps:**
1. Review and approve this specification
2. Create high-fidelity mockups in Figma
3. Build component library and design system
4. Implement screens iteratively (MVP: Dashboard + Agent Creation + Trade Execution)
5. Conduct usability testing
6. Iterate based on feedback

---

**Document Version:** 1.0.0
**Last Updated:** 2025-12-26
**Author:** SPARC Specification Agent
**Status:** Ready for Review
