//! Code execution within sandboxes.

use chrono::{DateTime, Utc};
use creto_common::CretoResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::sandbox::SandboxId;

/// Request to execute code in a sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Unique execution ID.
    pub id: Uuid,

    /// Target sandbox.
    pub sandbox_id: SandboxId,

    /// Code to execute.
    pub code: String,

    /// Entry point function (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_point: Option<String>,

    /// Input data (JSON).
    #[serde(default)]
    pub input: serde_json::Value,

    /// Maximum execution time override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u32>,

    /// Whether to capture stdout/stderr.
    #[serde(default = "default_true")]
    pub capture_output: bool,
}

fn default_true() -> bool {
    true
}

impl ExecutionRequest {
    /// Create a new execution request.
    pub fn new(sandbox_id: SandboxId, code: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            sandbox_id,
            code: code.into(),
            entry_point: None,
            input: serde_json::Value::Null,
            timeout_seconds: None,
            capture_output: true,
        }
    }

    /// Set the entry point.
    pub fn with_entry_point(mut self, entry_point: impl Into<String>) -> Self {
        self.entry_point = Some(entry_point.into());
        self
    }

    /// Set input data.
    pub fn with_input(mut self, input: serde_json::Value) -> Self {
        self.input = input;
        self
    }

    /// Set timeout override.
    pub fn with_timeout(mut self, seconds: u32) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }
}

/// Result of a code execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Execution request ID.
    pub request_id: Uuid,

    /// Execution status.
    pub status: ExecutionStatus,

    /// Output data (JSON).
    #[serde(default)]
    pub output: serde_json::Value,

    /// Captured stdout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,

    /// Captured stderr.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,

    /// Error message if failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ExecutionError>,

    /// Execution timing.
    pub timing: ExecutionTiming,
}

impl ExecutionResult {
    /// Create a successful result.
    pub fn success(request_id: Uuid, output: serde_json::Value, timing: ExecutionTiming) -> Self {
        Self {
            request_id,
            status: ExecutionStatus::Completed,
            output,
            stdout: None,
            stderr: None,
            error: None,
            timing,
        }
    }

    /// Create a failed result.
    pub fn failure(request_id: Uuid, error: ExecutionError, timing: ExecutionTiming) -> Self {
        Self {
            request_id,
            status: ExecutionStatus::Failed,
            output: serde_json::Value::Null,
            stdout: None,
            stderr: None,
            error: Some(error),
            timing,
        }
    }

    /// Check if execution was successful.
    pub fn is_success(&self) -> bool {
        self.status == ExecutionStatus::Completed
    }
}

/// Status of an execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    /// Queued for execution.
    Queued,
    /// Currently running.
    Running,
    /// Completed successfully.
    Completed,
    /// Failed with error.
    Failed,
    /// Timed out.
    TimedOut,
    /// Cancelled by user.
    Cancelled,
}

/// Execution error details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionError {
    /// Error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Stack trace (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
    /// Line number where error occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<u32>,
}

impl ExecutionError {
    /// Create a new execution error.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            stack_trace: None,
            line_number: None,
        }
    }

    /// Create a timeout error.
    pub fn timeout(timeout_seconds: u32) -> Self {
        Self::new(
            "TIMEOUT",
            format!("Execution timed out after {} seconds", timeout_seconds),
        )
    }

    /// Create a sandbox not found error.
    pub fn sandbox_not_found(sandbox_id: &str) -> Self {
        Self::new(
            "SANDBOX_NOT_FOUND",
            format!("Sandbox {} not found or terminated", sandbox_id),
        )
    }
}

/// Execution timing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTiming {
    /// When execution was queued.
    pub queued_at: DateTime<Utc>,
    /// When execution started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    /// When execution completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    /// Total execution duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl ExecutionTiming {
    /// Create new timing (queued now).
    pub fn new() -> Self {
        Self {
            queued_at: Utc::now(),
            started_at: None,
            completed_at: None,
            duration_ms: None,
        }
    }

    /// Mark execution as started.
    pub fn mark_started(&mut self) {
        self.started_at = Some(Utc::now());
    }

    /// Mark execution as completed.
    pub fn mark_completed(&mut self) {
        let now = Utc::now();
        self.completed_at = Some(now);
        if let Some(started) = self.started_at {
            self.duration_ms = Some(now.signed_duration_since(started).num_milliseconds() as u64);
        }
    }
}

impl Default for ExecutionTiming {
    fn default() -> Self {
        Self::new()
    }
}

/// Executor for running code in sandboxes.
pub struct Executor {
    // TODO: Add execution queue, worker pool
    _private: (),
}

impl Executor {
    /// Create a new executor.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Execute a request.
    pub async fn execute(&self, request: ExecutionRequest) -> CretoResult<ExecutionResult> {
        // TODO: Implement actual execution
        // 1. Validate sandbox exists and is ready
        // 2. Acquire sandbox lock
        // 3. Inject input data
        // 4. Execute code with timeout
        // 5. Capture output
        // 6. Release sandbox lock
        // 7. Return result

        let mut timing = ExecutionTiming::new();
        timing.mark_started();

        // Placeholder: Return mock success
        timing.mark_completed();
        Ok(ExecutionResult::success(
            request.id,
            serde_json::json!({"mock": true}),
            timing,
        ))
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::SandboxId;

    #[test]
    fn test_execution_request_builder() {
        let sandbox_id = SandboxId::new();
        let request = ExecutionRequest::new(sandbox_id, "print('hello')")
            .with_entry_point("main")
            .with_timeout(60);

        assert_eq!(request.code, "print('hello')");
        assert_eq!(request.entry_point, Some("main".to_string()));
        assert_eq!(request.timeout_seconds, Some(60));
    }

    #[test]
    fn test_execution_timing() {
        let mut timing = ExecutionTiming::new();
        assert!(timing.started_at.is_none());

        timing.mark_started();
        assert!(timing.started_at.is_some());

        timing.mark_completed();
        assert!(timing.completed_at.is_some());
        assert!(timing.duration_ms.is_some());
    }

    #[test]
    fn test_execution_error() {
        let error = ExecutionError::timeout(300);
        assert_eq!(error.code, "TIMEOUT");
        assert!(error.message.contains("300"));
    }
}
