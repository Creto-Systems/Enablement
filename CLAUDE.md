# Claude Code Configuration - Enablement

## 🎯 CRITICAL: SDD-FIRST METHODOLOGY

**This project follows a Software Design Document (SDD) first approach.**

### Core Principle
> **Design before code. Document before implement. Specify before build.**

No code should be written until the corresponding SDD section is complete and approved.

---

## 📋 SDD Document Structure

All design documents live in `/docs/sdd/`:

```
docs/sdd/
├── 00-overview.md           # Executive summary, vision, scope
├── 01-requirements.md       # Functional & non-functional requirements
├── 02-architecture.md       # System architecture & component design
├── 03-data-design.md        # Data models, schemas, storage strategy
├── 04-api-design.md         # API contracts, endpoints, interfaces
├── 05-security-design.md    # Security model, auth, encryption
├── 06-integration-design.md # External systems, third-party services
├── 07-deployment-design.md  # Infrastructure, CI/CD, environments
├── 08-testing-strategy.md   # Test plans, coverage requirements
└── 09-implementation-plan.md # Phased rollout, milestones, timeline
```

---

## 🔄 SDD Development Workflow

### Phase 1: Discovery & Requirements
1. Define problem statement and target users
2. Document functional requirements (user stories, use cases)
3. Document non-functional requirements (performance, scale, security)
4. Identify constraints and dependencies

### Phase 2: Architecture & Design
1. Design system architecture (components, boundaries, interactions)
2. Define data models and storage strategy
3. Specify API contracts and interfaces
4. Document security model and threat analysis

### Phase 3: Implementation Planning
1. Break down into implementation phases
2. Define milestones and success criteria
3. Establish testing strategy
4. Plan deployment and rollout

### Phase 4: Implementation (Code)
- **Only after SDD approval**
- Code follows the documented design
- Deviations require SDD updates first

---

## 📝 SDD Section Templates

### Each SDD Section Must Include:
- **Purpose**: Why this section exists
- **Scope**: What it covers and doesn't cover
- **Decisions**: Key design decisions with rationale
- **Diagrams**: Visual representations where applicable
- **Open Questions**: Unresolved items needing discussion
- **Revision History**: Track changes and approvals

---

## ⚠️ Rules of Engagement

### DO:
- Start every feature with an SDD section
- Document decisions and their rationale
- Update SDDs when requirements change
- Use diagrams (Mermaid, ASCII) for clarity
- Keep each document focused and concise

### DON'T:
- Write code before the design is documented
- Skip sections because "it's obvious"
- Let SDDs become stale
- Create implementation without traceability to SDD

---

## 🗂️ Project Structure

```
Enablement/
├── CLAUDE.md              # This file - project instructions
├── README.md              # Project overview (generated from SDD)
├── docs/
│   ├── sdd/               # Software Design Documents
│   ├── decisions/         # Architecture Decision Records (ADRs)
│   └── diagrams/          # Source files for diagrams
├── src/                   # Source code (after SDD approval)
├── tests/                 # Test files
└── config/                # Configuration files
```

---

## 🚀 Getting Started

When starting work on Enablement:

1. **Review existing SDDs** in `/docs/sdd/`
2. **Identify the section** relevant to your task
3. **Update or create SDD** before any implementation
4. **Get alignment** on design decisions
5. **Then implement** following the documented design

---

## 📊 SDD Status Tracking

Use frontmatter in each SDD file:

```yaml
---
status: draft | review | approved | implemented
author: [name]
created: YYYY-MM-DD
updated: YYYY-MM-DD
reviewers: [names]
---
```

---

## 🔗 Related Resources

- Architecture Decision Records: `/docs/decisions/`
- Diagrams source: `/docs/diagrams/`
- Implementation tracking: GitHub Issues/Projects

---

**Remember: The SDD is the source of truth. Code is the implementation of that truth.**
