# Creto Enablement Layer 🚀

> **Orchestration and governance for AI agents at enterprise scale**
>
> Built on the Sovereign platform, providing metering, oversight, runtime, and messaging for agentic operations.

**[Quick Start](#-quick-start)** | **[Documentation](docs/sdd/)** | **[Demos](demos/)**

---

## 🎯 Overview

The **Enablement Layer** provides four core products for managing autonomous AI agents:

```
┌─────────────────────────────────────────────────────────────────┐
│                    ENABLEMENT LAYER (This Repo)                 │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌─────────┐ │
│  │   Metering   │ │   Oversight  │ │   Runtime    │ │Messaging│ │
│  │  (billing)   │ │    (HITL)    │ │  (sandbox)   │ │  (E2E)  │ │
│  └──────────────┘ └──────────────┘ └──────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              ↓ built on
┌─────────────────────────────────────────────────────────────────┐
│                SOVEREIGN PLATFORM (External)                    │
│   Quantum-resistant crypto • Consensus • Authorization • Audit  │
└─────────────────────────────────────────────────────────────────┘
```

| Product | Description | OSS Pattern |
|---------|-------------|-------------|
| **creto-metering** | Usage-based billing and quota enforcement | Lago |
| **creto-oversight** | Human-in-the-loop approval workflows | HumanLayer |
| **creto-runtime** | Sandboxed agent execution | Agent Sandbox |
| **creto-messaging** | Secure agent-to-agent communication | Signal Protocol |

---

## 🚀 Quick Start

### Prerequisites
- **Rust 1.75+** ([rustup.rs](https://rustup.rs/))
- **Node.js 18+** (for demos)

### Build

```bash
# Clone the repository
git clone https://github.com/Creto-Systems/Enablement.git
cd Enablement

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace
```

### Run Demos

```bash
# Trading Demo (localhost:3000)
cd demos/trading-demo && npm install && npm run dev

# Travel Demo (localhost:5173)
cd demos/travel-demo && npm install && npm run dev

# Healthcare Demo (localhost:5175)
cd demos/healthcare-demo && npm install && npm run client:dev

# PSA Demo (localhost:5174)
cd demos/psa-demo && npm install && npm run client:dev
```

---

## 📦 Project Structure

```
creto-enablement/
├── crates/
│   ├── creto-metering/      # Usage-based billing
│   ├── creto-oversight/     # Human-in-the-loop
│   ├── creto-runtime/       # Sandboxed execution
│   ├── creto-messaging/     # Secure messaging
│   └── creto-common/        # Shared types
├── demos/
│   ├── trading-demo/        # Financial agent oversight
│   ├── travel-demo/         # Travel booking agent
│   ├── healthcare-demo/     # Healthcare data access
│   └── psa-demo/            # Professional services
├── docs/
│   └── sdd/                 # Software Design Documents
└── tests/                   # Integration tests
```

---

## 🔧 Development

```bash
# Build specific crate
cargo build -p creto-metering

# Run tests for a crate
cargo test -p creto-oversight

# Check all code
cargo clippy --workspace

# Format code
cargo fmt --all
```

---

## 📚 Documentation

- **[Software Design Documents](docs/sdd/)** - Architecture and specifications
- **[Product SDDs](docs/sdd/products/)** - Per-product design docs
- **[Decisions](docs/decisions/)** - Architecture Decision Records

---

## 🔗 Related Projects

- **[Sovereign](https://github.com/Creto-Systems/Sovereign)** - Quantum-resistant security platform
- **Creto AuthZ Engine** - Authorization service (in development)

---

## 📄 License

Apache 2.0 - See [LICENSE](LICENSE) for details.

---

**Creto Systems** - Trusted Vigilance for the Agentic Enterprise

