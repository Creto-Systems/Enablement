//! State machine for oversight request lifecycle.

use chrono::{DateTime, Utc};
use creto_common::{CretoError, CretoResult, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::request::RequestStatus;

/// State machine managing the lifecycle of an oversight request.
#[derive(Debug, Clone)]
pub struct StateMachine {
    /// Current state.
    current: RequestStatus,
    /// History of state transitions.
    transitions: Vec<StateTransition>,
}

impl StateMachine {
    /// Create a new state machine in the Pending state.
    pub fn new() -> Self {
        Self {
            current: RequestStatus::Pending,
            transitions: Vec::new(),
        }
    }

    /// Create a state machine from an existing state.
    pub fn from_state(state: RequestStatus) -> Self {
        Self {
            current: state,
            transitions: Vec::new(),
        }
    }

    /// Get the current state.
    pub fn current(&self) -> RequestStatus {
        self.current
    }

    /// Get the transition history.
    pub fn history(&self) -> &[StateTransition] {
        &self.transitions
    }

    /// Attempt to transition to a new state.
    pub fn transition(
        &mut self,
        to: RequestStatus,
        actor: Actor,
        reason: Option<String>,
    ) -> CretoResult<()> {
        if !self.can_transition_to(to) {
            return Err(CretoError::InvalidStateTransition {
                from: format!("{:?}", self.current),
                to: format!("{:?}", to),
            });
        }

        let transition = StateTransition {
            id: Uuid::now_v7(),
            from: self.current,
            to,
            actor,
            reason,
            timestamp: Utc::now(),
        };

        self.transitions.push(transition);
        self.current = to;

        Ok(())
    }

    /// Check if a transition to the given state is valid.
    pub fn can_transition_to(&self, to: RequestStatus) -> bool {
        use RequestStatus::*;

        matches!(
            (self.current, to),
            // From Pending
            (Pending, InReview)
                | (Pending, Approved)
                | (Pending, Rejected)
                | (Pending, Escalated)
                | (Pending, TimedOut)
                | (Pending, Cancelled)
                // From InReview
                | (InReview, Approved)
                | (InReview, Rejected)
                | (InReview, Escalated)
                | (InReview, TimedOut)
                | (InReview, Cancelled)
                // From Escalated
                | (Escalated, Approved)
                | (Escalated, Rejected)
                | (Escalated, TimedOut)
                | (Escalated, Cancelled)
        )
    }

    /// Get valid next states from the current state.
    pub fn valid_transitions(&self) -> Vec<RequestStatus> {
        use RequestStatus::*;

        let all_states = [
            Pending, InReview, Approved, Rejected, Escalated, TimedOut, Cancelled,
        ];

        all_states
            .into_iter()
            .filter(|&s| self.can_transition_to(s))
            .collect()
    }

    /// Check if the current state is terminal.
    pub fn is_terminal(&self) -> bool {
        self.current.is_terminal()
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// A recorded state transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Unique transition ID.
    pub id: Uuid,
    /// Previous state.
    pub from: RequestStatus,
    /// New state.
    pub to: RequestStatus,
    /// Who triggered the transition.
    pub actor: Actor,
    /// Optional reason for the transition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// When the transition occurred.
    pub timestamp: DateTime<Utc>,
}

/// Actor who triggered a state transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Actor {
    /// System-initiated transition (e.g., timeout).
    System,
    /// Human reviewer.
    User { user_id: UserId },
    /// Cedar policy auto-decision.
    Policy { policy_id: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions_from_pending() {
        let machine = StateMachine::new();

        assert!(machine.can_transition_to(RequestStatus::InReview));
        assert!(machine.can_transition_to(RequestStatus::Approved));
        assert!(machine.can_transition_to(RequestStatus::Rejected));
        assert!(machine.can_transition_to(RequestStatus::Escalated));
        assert!(machine.can_transition_to(RequestStatus::TimedOut));
        assert!(machine.can_transition_to(RequestStatus::Cancelled));

        // Cannot stay in pending
        assert!(!machine.can_transition_to(RequestStatus::Pending));
    }

    #[test]
    fn test_invalid_transition_from_terminal() {
        let machine = StateMachine::from_state(RequestStatus::Approved);

        // Terminal states have no valid transitions
        assert!(machine.valid_transitions().is_empty());
    }

    #[test]
    fn test_transition_records_history() {
        let mut machine = StateMachine::new();

        machine
            .transition(
                RequestStatus::InReview,
                Actor::User {
                    user_id: UserId::new(),
                },
                Some("Starting review".to_string()),
            )
            .unwrap();

        machine
            .transition(
                RequestStatus::Approved,
                Actor::User {
                    user_id: UserId::new(),
                },
                Some("Looks good".to_string()),
            )
            .unwrap();

        assert_eq!(machine.history().len(), 2);
        assert_eq!(machine.current(), RequestStatus::Approved);
    }

    #[test]
    fn test_invalid_transition_returns_error() {
        let mut machine = StateMachine::from_state(RequestStatus::Approved);

        let result = machine.transition(RequestStatus::Pending, Actor::System, None);

        assert!(result.is_err());
    }
}
