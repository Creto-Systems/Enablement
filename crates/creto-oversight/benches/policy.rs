//! Policy evaluation benchmarks.
//!
//! Verifies the <1ms p99 latency target for policy evaluation and state transitions.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use creto_common::{AgentId, OrganizationId, UserId};
use creto_oversight::{
    ActionType, Approval, ApprovalDecision, PolicyContext, PolicyDecision,
    QuorumCalculator, QuorumConfig, RequestStatus, StateMachine, TrustLevel,
    policy::PolicyEngine, state::Actor,
};
use std::time::Duration;
use uuid::Uuid;

/// Benchmark: Policy Engine Evaluation (<1ms target)
fn bench_policy_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("policy_evaluation");
    group.throughput(Throughput::Elements(1000));
    group.measurement_time(Duration::from_secs(10));

    let engine = PolicyEngine::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Transaction action
    let transaction_action = ActionType::Transaction {
        amount_cents: 500000,
        currency: "USD".to_string(),
    };

    let context = PolicyContext {
        trust_level: TrustLevel::Standard,
        quota_usage_percentage: 50.0,
        delegation_depth: 0,
        ..Default::default()
    };

    group.bench_function("evaluate_transaction_1000", |b| {
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..1000 {
                    let result = black_box(engine.evaluate(&transaction_action, &context).await);
                    black_box(result);
                }
            });
        });
    });

    // Data access action
    let data_access_action = ActionType::DataAccess {
        data_type: "customer_records".to_string(),
        scope: "pii".to_string(),
    };

    group.bench_function("evaluate_data_access_1000", |b| {
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..1000 {
                    let result = black_box(engine.evaluate(&data_access_action, &context).await);
                    black_box(result);
                }
            });
        });
    });

    // Code execution action
    let code_execution_action = ActionType::CodeExecution {
        runtime: "python".to_string(),
        risk_level: "high".to_string(),
    };

    group.bench_function("evaluate_code_execution_1000", |b| {
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..1000 {
                    let result = black_box(engine.evaluate(&code_execution_action, &context).await);
                    black_box(result);
                }
            });
        });
    });

    group.finish();
}

/// Benchmark: State Machine Transitions (<1ms target)
fn bench_state_machine(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_machine");
    group.throughput(Throughput::Elements(100));
    group.measurement_time(Duration::from_secs(10));

    let user_id = UserId::new();

    group.bench_function("transition_pending_to_in_review_100", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut sm = StateMachine::new();
                let result = sm.transition(
                    RequestStatus::InReview,
                    Actor::User { user_id },
                    Some("Starting review".to_string()),
                );
                black_box(result);
            }
        });
    });

    group.bench_function("full_workflow_100", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut sm = StateMachine::new();

                // Pending -> InReview
                let _ = sm.transition(
                    RequestStatus::InReview,
                    Actor::User { user_id },
                    None,
                );

                // InReview -> Approved
                let _ = sm.transition(
                    RequestStatus::Approved,
                    Actor::User { user_id },
                    Some("Approved".to_string()),
                );

                black_box(sm.current());
            }
        });
    });

    group.finish();
}

/// Benchmark: Quorum Calculation
fn bench_quorum_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("quorum_calculation");
    group.throughput(Throughput::Elements(1000));
    group.measurement_time(Duration::from_secs(10));

    let request_id = Uuid::now_v7();

    // Create approvals
    let approvals: Vec<Approval> = (0..5)
        .map(|_| Approval::new(request_id, UserId::new(), ApprovalDecision::Approve))
        .collect();

    // 2-of-5 quorum
    let quorum_config = QuorumConfig::n_of_m(2);
    let calculator = QuorumCalculator::new(quorum_config);

    group.bench_function("evaluate_2_of_5_quorum_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let result = black_box(calculator.evaluate(&approvals));
                black_box(result);
            }
        });
    });

    // 3-of-5 quorum
    let quorum_config_3 = QuorumConfig::n_of_m(3);
    let calculator_3 = QuorumCalculator::new(quorum_config_3);

    group.bench_function("evaluate_3_of_5_quorum_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let result = black_box(calculator_3.evaluate(&approvals));
                black_box(result);
            }
        });
    });

    // Unanimous quorum
    let quorum_config_unanimous = QuorumConfig::unanimous();
    let calculator_unanimous = QuorumCalculator::new(quorum_config_unanimous);

    group.bench_function("evaluate_unanimous_quorum_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let result = black_box(calculator_unanimous.evaluate(&approvals));
                black_box(result);
            }
        });
    });

    group.finish();
}

/// Benchmark: Single Policy Evaluation Latency
fn bench_policy_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("policy_latency");
    group.sample_size(1000);
    group.measurement_time(Duration::from_secs(15));

    let engine = PolicyEngine::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let action = ActionType::Transaction {
        amount_cents: 1500000,
        currency: "USD".to_string(),
    };

    let context = PolicyContext::default();

    group.bench_function("single_evaluation_latency", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(engine.evaluate(&action, &context).await)
            })
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_policy_evaluation,
    bench_state_machine,
    bench_quorum_calculation,
    bench_policy_latency
);
criterion_main!(benches);
