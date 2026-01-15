//! Safety module for guardrails, validation, and approval workflows.
//!
//! The safety system provides multiple layers of protection:
//! - Guardrails: Hard and soft constraints on agent behavior
//! - Validation: Pre-execution safety checks
//! - Approval: Human-in-the-loop for high-stakes actions
//! - Audit: Logging of all actions for review

use crate::action::Action;
use crate::error::SafetyError;
use crate::id::ActionId;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Safety validator for pre-execution checks
#[async_trait]
pub trait SafetyValidator: Send + Sync {
    /// Validate an action before execution
    async fn validate(&self, action: &Action, context: &SafetyContext) -> SafetyResult;

    /// Get the name of this validator
    fn name(&self) -> &str;
}

/// Result of safety validation
#[derive(Debug, Clone)]
pub enum SafetyResult {
    /// Action is safe to execute
    Pass,
    /// Action failed validation
    Fail(SafetyViolation),
    /// Action requires human approval
    RequiresApproval(ApprovalReason),
    /// Action should be modified
    Modify(ActionModification),
}

impl SafetyResult {
    /// Check if this is a pass
    pub fn is_pass(&self) -> bool {
        matches!(self, SafetyResult::Pass)
    }

    /// Check if this requires approval
    pub fn requires_approval(&self) -> bool {
        matches!(self, SafetyResult::RequiresApproval(_))
    }
}

/// Details of a safety violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyViolation {
    /// Name of the guardrail/validator that was violated
    pub guardrail: String,
    /// Severity of the violation
    pub severity: Severity,
    /// Description of what was wrong
    pub description: String,
    /// Suggested alternative action
    pub suggested_alternative: Option<String>,
}

/// Severity levels for safety violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Informational only
    Info,
    /// Warning - proceed with caution
    Warning,
    /// Error - should not proceed
    Error,
    /// Critical - must not proceed
    Critical,
}

/// Reason approval is required
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalReason {
    /// Why approval is needed
    pub reason: String,
    /// Risk level
    pub risk_level: RiskLevel,
    /// What the action will do
    pub action_summary: String,
}

/// Risk level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Low risk
    #[default]
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Critical risk
    Critical,
}

/// Suggested modification to an action
#[derive(Debug, Clone)]
pub struct ActionModification {
    /// Description of the modification
    pub description: String,
    /// The modified action
    pub modified_action: Action,
}

/// Context for safety validation
#[derive(Debug, Clone, Default)]
pub struct SafetyContext {
    /// Current session/conversation
    pub session_id: Option<crate::id::SessionId>,
    /// User who initiated the action
    pub user_id: Option<String>,
    /// Actions already taken in this session
    pub prior_actions: Vec<ActionId>,
    /// Current risk tolerance
    pub risk_tolerance: RiskLevel,
    /// Whether we're in a test/sandbox environment
    pub sandbox_mode: bool,
}

impl SafetyContext {
    /// Create a new safety context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set sandbox mode
    pub fn with_sandbox_mode(mut self, sandbox: bool) -> Self {
        self.sandbox_mode = sandbox;
        self
    }

    /// Set risk tolerance
    pub fn with_risk_tolerance(mut self, tolerance: RiskLevel) -> Self {
        self.risk_tolerance = tolerance;
        self
    }
}

/// A guardrail that checks specific conditions
pub trait Guardrail: Send + Sync {
    /// Check if an action passes this guardrail
    fn check(&self, action: &Action) -> GuardrailResult;

    /// Name of this guardrail
    fn name(&self) -> &str;

    /// Description of what this guardrail checks
    fn description(&self) -> &str;

    /// Severity if violated
    fn severity(&self) -> Severity;

    /// Whether this guardrail can be overridden
    fn overridable(&self) -> bool {
        false
    }
}

/// Result of a guardrail check
#[derive(Debug, Clone)]
pub enum GuardrailResult {
    /// Guardrail passed
    Pass,
    /// Guardrail failed
    Fail { reason: String },
    /// Guardrail triggered a warning but allows proceeding
    Warn { reason: String },
}

impl GuardrailResult {
    /// Check if this is a pass
    pub fn is_pass(&self) -> bool {
        matches!(self, GuardrailResult::Pass)
    }

    /// Check if this is a failure
    pub fn is_fail(&self) -> bool {
        matches!(self, GuardrailResult::Fail { .. })
    }
}

/// Approval workflow for human-in-the-loop
#[async_trait]
pub trait ApprovalWorkflow: Send + Sync {
    /// Request approval for an action
    async fn request_approval(
        &self,
        action: &Action,
        reason: &ApprovalReason,
    ) -> Result<ApprovalRequest, SafetyError>;

    /// Wait for a decision on an approval request
    async fn await_decision(
        &self,
        request: &ApprovalRequest,
        timeout: Option<Duration>,
    ) -> Result<ApprovalDecision, SafetyError>;

    /// Cancel a pending approval request
    async fn cancel(&self, request: &ApprovalRequest) -> Result<(), SafetyError>;
}

/// A pending approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique request ID
    pub id: String,
    /// The action requiring approval
    pub action_id: ActionId,
    /// Why approval is needed
    pub reason: ApprovalReason,
    /// When the request was created
    pub created_at: DateTime<Utc>,
    /// When the request expires
    pub expires_at: Option<DateTime<Utc>>,
    /// Current status
    pub status: ApprovalStatus,
}

/// Status of an approval request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    /// Waiting for decision
    Pending,
    /// Approved
    Approved,
    /// Denied
    Denied,
    /// Expired without decision
    Expired,
    /// Cancelled
    Cancelled,
}

/// Decision on an approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalDecision {
    /// Action approved
    Approved {
        /// Who approved
        approver: Option<String>,
        /// Any conditions on the approval
        conditions: Option<String>,
    },
    /// Action denied
    Denied {
        /// Who denied
        denier: Option<String>,
        /// Reason for denial
        reason: String,
    },
    /// Action modified and approved
    ModifiedAndApproved {
        /// Who approved
        approver: Option<String>,
        /// The modifications made
        modifications: String,
    },
}

impl ApprovalDecision {
    /// Check if this is an approval
    pub fn is_approved(&self) -> bool {
        matches!(
            self,
            ApprovalDecision::Approved { .. } | ApprovalDecision::ModifiedAndApproved { .. }
        )
    }
}

/// Audit logger for recording all actions
#[async_trait]
pub trait AuditLogger: Send + Sync {
    /// Log an action
    async fn log_action(&self, entry: AuditEntry) -> Result<(), SafetyError>;

    /// Query audit logs
    async fn query(
        &self,
        query: &AuditQuery,
    ) -> Result<Vec<AuditEntry>, SafetyError>;
}

/// An entry in the audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Entry ID
    pub id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// The action
    pub action: Action,
    /// Safety validation result
    pub validation_result: ValidationSummary,
    /// Approval status (if applicable)
    pub approval: Option<ApprovalSummary>,
    /// Execution result
    pub execution_result: Option<ExecutionSummary>,
    /// Agent ID
    pub agent_id: crate::id::AgentId,
    /// Session ID
    pub session_id: Option<crate::id::SessionId>,
}

/// Summary of validation for audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Whether validation passed
    pub passed: bool,
    /// Guardrails that were checked
    pub guardrails_checked: Vec<String>,
    /// Any violations
    pub violations: Vec<SafetyViolation>,
}

/// Summary of approval for audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalSummary {
    /// Whether approval was required
    pub required: bool,
    /// The decision
    pub decision: Option<ApprovalDecision>,
    /// Time to decision
    pub decision_time_ms: Option<u64>,
}

/// Summary of execution for audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    /// Whether execution succeeded
    pub success: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error message if failed
    pub error: Option<String>,
}

/// Query for audit logs
#[derive(Debug, Clone, Default)]
pub struct AuditQuery {
    /// Filter by agent
    pub agent_id: Option<crate::id::AgentId>,
    /// Filter by session
    pub session_id: Option<crate::id::SessionId>,
    /// Filter by time range
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Filter by action type
    pub action_type: Option<String>,
    /// Only show violations
    pub violations_only: bool,
    /// Maximum results
    pub limit: usize,
}

/// Pipeline of safety validators
pub struct SafetyPipeline {
    validators: Vec<Arc<dyn SafetyValidator>>,
    guardrails: Vec<Arc<dyn Guardrail>>,
    approval_workflow: Option<Arc<dyn ApprovalWorkflow>>,
    audit_logger: Option<Arc<dyn AuditLogger>>,
}

impl SafetyPipeline {
    /// Create a new safety pipeline
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
            guardrails: Vec::new(),
            approval_workflow: None,
            audit_logger: None,
        }
    }

    /// Add a validator
    pub fn with_validator(mut self, validator: Arc<dyn SafetyValidator>) -> Self {
        self.validators.push(validator);
        self
    }

    /// Add a guardrail
    pub fn with_guardrail(mut self, guardrail: Arc<dyn Guardrail>) -> Self {
        self.guardrails.push(guardrail);
        self
    }

    /// Set the approval workflow
    pub fn with_approval_workflow(mut self, workflow: Arc<dyn ApprovalWorkflow>) -> Self {
        self.approval_workflow = Some(workflow);
        self
    }

    /// Set the audit logger
    pub fn with_audit_logger(mut self, logger: Arc<dyn AuditLogger>) -> Self {
        self.audit_logger = Some(logger);
        self
    }

    /// Validate an action through the pipeline
    pub async fn validate(&self, action: &Action, context: &SafetyContext) -> SafetyResult {
        // Check all guardrails first
        for guardrail in &self.guardrails {
            match guardrail.check(action) {
                GuardrailResult::Fail { reason } => {
                    return SafetyResult::Fail(SafetyViolation {
                        guardrail: guardrail.name().to_string(),
                        severity: guardrail.severity(),
                        description: reason,
                        suggested_alternative: None,
                    });
                }
                GuardrailResult::Warn { reason } => {
                    if guardrail.severity() >= Severity::Error {
                        return SafetyResult::Fail(SafetyViolation {
                            guardrail: guardrail.name().to_string(),
                            severity: guardrail.severity(),
                            description: reason,
                            suggested_alternative: None,
                        });
                    }
                }
                GuardrailResult::Pass => {}
            }
        }

        // Run all validators
        for validator in &self.validators {
            let result = validator.validate(action, context).await;
            match result {
                SafetyResult::Pass => {}
                other => return other,
            }
        }

        SafetyResult::Pass
    }
}

impl Default for SafetyPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::Error);
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Critical > RiskLevel::High);
        assert!(RiskLevel::High > RiskLevel::Medium);
        assert!(RiskLevel::Medium > RiskLevel::Low);
    }

    #[test]
    fn test_safety_result() {
        assert!(SafetyResult::Pass.is_pass());
        assert!(!SafetyResult::Pass.requires_approval());

        let fail = SafetyResult::Fail(SafetyViolation {
            guardrail: "test".to_string(),
            severity: Severity::Error,
            description: "test violation".to_string(),
            suggested_alternative: None,
        });
        assert!(!fail.is_pass());
    }

    #[test]
    fn test_approval_decision() {
        let approved = ApprovalDecision::Approved {
            approver: Some("user".to_string()),
            conditions: None,
        };
        assert!(approved.is_approved());

        let denied = ApprovalDecision::Denied {
            denier: None,
            reason: "not allowed".to_string(),
        };
        assert!(!denied.is_approved());
    }
}
