# Privacy in AI Agents: Research Report

## Executive Summary

Privacy is a foundational concern for AI agents that process user data, interact with external systems, and maintain persistent memory. As agents become more capable and autonomous, the potential for privacy violations increases exponentially. This report synthesizes research and best practices across three key domains: PII handling and data classification, regulatory compliance (GDPR, CCPA), and privacy-enhancing techniques (differential privacy, federated learning, privacy by design). The goal is to provide actionable guidance for building privacy-respecting agents in the Rust-based agentic framework.

---

## 1. PII Handling and Sensitive Data Classification

### 1.1 The Privacy Challenge for Agents

AI agents face unique privacy challenges:

- **Data ingestion**: Agents read files, emails, code, and documents that may contain PII
- **Memory persistence**: Long-term memory stores may accumulate sensitive information
- **Tool execution**: Agents may pass data to external APIs or services
- **Logging and debugging**: Conversation logs may contain user secrets
- **Multi-user contexts**: Agents may serve multiple users with data isolation requirements

### 1.2 PII Detection Strategies

#### **Pattern-Based Detection**

Regular expressions for common PII types:

```rust
pub struct PiiPatterns {
    /// Email addresses
    email: Regex,
    /// Phone numbers (various formats)
    phone: Vec<Regex>,
    /// Social Security Numbers
    ssn: Regex,
    /// Credit card numbers (Luhn-validated)
    credit_card: Regex,
    /// IP addresses
    ip_address: Regex,
    /// Dates of birth
    dob: Regex,
    /// Physical addresses
    address: Regex,
}

impl PiiPatterns {
    pub fn scan(&self, text: &str) -> Vec<PiiMatch> {
        let mut matches = Vec::new();

        // Email detection
        for m in self.email.find_iter(text) {
            matches.push(PiiMatch {
                pii_type: PiiType::Email,
                span: m.range(),
                confidence: 0.95,
            });
        }

        // Credit card with Luhn validation
        for m in self.credit_card.find_iter(text) {
            if luhn_check(m.as_str()) {
                matches.push(PiiMatch {
                    pii_type: PiiType::CreditCard,
                    span: m.range(),
                    confidence: 0.99,
                });
            }
        }

        // ... other patterns
        matches
    }
}
```

#### **NER-Based Detection**

Named Entity Recognition for contextual PII:

```rust
pub trait PiiDetector: Send + Sync {
    /// Detect PII in text with confidence scores
    fn detect(&self, text: &str) -> Vec<PiiMatch>;

    /// Detect with context (surrounding text may inform detection)
    fn detect_with_context(
        &self,
        text: &str,
        context: &DetectionContext,
    ) -> Vec<PiiMatch>;
}

pub struct HybridPiiDetector {
    /// Fast pattern matching
    patterns: PiiPatterns,

    /// NER model for names, organizations, locations
    ner_model: Box<dyn NerModel>,

    /// Context-aware classifier
    classifier: Box<dyn SensitivityClassifier>,
}

impl PiiDetector for HybridPiiDetector {
    fn detect(&self, text: &str) -> Vec<PiiMatch> {
        let mut matches = self.patterns.scan(text);

        // Add NER-detected entities
        let entities = self.ner_model.extract(text);
        for entity in entities {
            if entity.is_pii_candidate() {
                matches.push(entity.into());
            }
        }

        // Deduplicate and merge overlapping matches
        self.merge_matches(matches)
    }
}
```

#### **PII Categories and Sensitivity Levels**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PiiType {
    // Direct Identifiers (High Sensitivity)
    FullName,
    Email,
    Phone,
    SocialSecurityNumber,
    PassportNumber,
    DriversLicense,
    NationalId,

    // Financial (High Sensitivity)
    CreditCard,
    BankAccount,
    FinancialRecord,

    // Health (High Sensitivity - HIPAA)
    MedicalRecord,
    HealthCondition,
    Prescription,

    // Biometric (High Sensitivity)
    Fingerprint,
    FaceImage,
    VoicePrint,

    // Location (Medium Sensitivity)
    PhysicalAddress,
    GpsCoordinates,
    IpAddress,

    // Demographic (Medium Sensitivity)
    DateOfBirth,
    Age,
    Gender,
    Race,
    Religion,

    // Digital Identifiers (Medium Sensitivity)
    Username,
    DeviceId,
    CookieId,

    // Quasi-Identifiers (Low Individual, High Combined)
    ZipCode,
    Occupation,
    EducationLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SensitivityLevel {
    /// Public information, no restrictions
    Public = 0,
    /// Internal use, basic protections
    Internal = 1,
    /// Confidential, access controls required
    Confidential = 2,
    /// Highly sensitive, encryption + audit required
    Sensitive = 3,
    /// Restricted, special handling (health, financial)
    Restricted = 4,
}

impl PiiType {
    pub fn sensitivity(&self) -> SensitivityLevel {
        match self {
            // Direct identifiers: Restricted
            Self::SocialSecurityNumber |
            Self::PassportNumber |
            Self::DriversLicense |
            Self::NationalId => SensitivityLevel::Restricted,

            // Financial: Restricted
            Self::CreditCard |
            Self::BankAccount |
            Self::FinancialRecord => SensitivityLevel::Restricted,

            // Health: Restricted
            Self::MedicalRecord |
            Self::HealthCondition |
            Self::Prescription => SensitivityLevel::Restricted,

            // Biometric: Restricted
            Self::Fingerprint |
            Self::FaceImage |
            Self::VoicePrint => SensitivityLevel::Restricted,

            // Contact info: Sensitive
            Self::FullName |
            Self::Email |
            Self::Phone |
            Self::PhysicalAddress => SensitivityLevel::Sensitive,

            // Location: Confidential
            Self::GpsCoordinates |
            Self::IpAddress => SensitivityLevel::Confidential,

            // Demographics: Confidential
            Self::DateOfBirth |
            Self::Age |
            Self::Gender |
            Self::Race |
            Self::Religion => SensitivityLevel::Confidential,

            // Digital IDs: Internal
            Self::Username |
            Self::DeviceId |
            Self::CookieId => SensitivityLevel::Internal,

            // Quasi-identifiers: Internal
            Self::ZipCode |
            Self::Occupation |
            Self::EducationLevel => SensitivityLevel::Internal,
        }
    }
}
```

### 1.3 Data Minimization Principles

#### **Collection Minimization**

Only collect data that is strictly necessary:

```rust
pub struct DataCollectionPolicy {
    /// Allowed PII types for this context
    allowed_pii: HashSet<PiiType>,

    /// Maximum retention period
    retention: Duration,

    /// Purpose limitation
    purposes: Vec<DataPurpose>,

    /// Whether to redact disallowed PII
    redact_disallowed: bool,
}

impl DataCollectionPolicy {
    /// Apply policy to incoming data
    pub fn apply(&self, data: &str) -> PolicyResult {
        let detected = self.detector.detect(data);

        let mut violations = Vec::new();
        let mut redacted = data.to_string();

        for pii in detected {
            if !self.allowed_pii.contains(&pii.pii_type) {
                if self.redact_disallowed {
                    redacted = self.redact(&redacted, &pii);
                } else {
                    violations.push(PolicyViolation {
                        pii_type: pii.pii_type,
                        action: ViolationAction::Blocked,
                    });
                }
            }
        }

        PolicyResult { redacted, violations }
    }
}
```

#### **Storage Minimization**

```rust
pub struct StoragePolicy {
    /// What to store in memory
    memory_policy: MemoryStoragePolicy,

    /// What to store persistently
    persistent_policy: PersistentStoragePolicy,

    /// Automatic expiration
    ttl_by_sensitivity: HashMap<SensitivityLevel, Duration>,
}

#[derive(Debug, Clone)]
pub enum MemoryStoragePolicy {
    /// Store nothing sensitive in memory
    NoSensitive,
    /// Store with encryption
    EncryptedOnly,
    /// Store redacted versions
    RedactedOnly,
    /// Store with access controls
    AccessControlled,
}
```

#### **Retention Minimization**

```rust
pub struct RetentionManager {
    /// Retention policies by data type
    policies: HashMap<DataType, RetentionPolicy>,

    /// Background cleanup task
    cleanup_interval: Duration,
}

impl RetentionManager {
    pub async fn cleanup(&self, storage: &mut dyn DataStorage) -> CleanupResult {
        let now = Utc::now();
        let mut deleted = 0;

        for (data_type, policy) in &self.policies {
            let expired = storage
                .find_expired(data_type, now - policy.retention_period)
                .await?;

            for item in expired {
                // Secure deletion
                storage.secure_delete(&item.id).await?;
                deleted += 1;
            }
        }

        CleanupResult { deleted_count: deleted }
    }
}
```

### 1.4 Data Handling Patterns for Agents

#### **Secure Data Flow**

```rust
/// Data wrapper that tracks sensitivity
pub struct SensitiveData<T> {
    /// The actual data
    inner: T,

    /// Sensitivity classification
    sensitivity: SensitivityLevel,

    /// Data lineage for audit
    lineage: DataLineage,

    /// Access restrictions
    access: AccessPolicy,
}

impl<T> SensitiveData<T> {
    /// Access data with audit logging
    pub fn access(&self, accessor: &Accessor) -> Result<&T, AccessError> {
        if !self.access.allows(accessor) {
            return Err(AccessError::Unauthorized);
        }

        // Log access
        audit_log::record(AuditEvent::DataAccess {
            data_id: self.lineage.id,
            accessor: accessor.clone(),
            timestamp: Utc::now(),
        });

        Ok(&self.inner)
    }

    /// Transform data while maintaining sensitivity tracking
    pub fn map<U, F>(self, f: F) -> SensitiveData<U>
    where
        F: FnOnce(T) -> U,
    {
        SensitiveData {
            inner: f(self.inner),
            sensitivity: self.sensitivity,
            lineage: self.lineage.extend("transformed"),
            access: self.access,
        }
    }
}
```

#### **Redaction Strategies**

```rust
pub enum RedactionStrategy {
    /// Replace with placeholder: "***REDACTED***"
    Placeholder,

    /// Replace with type indicator: "[EMAIL]", "[SSN]"
    TypeIndicator,

    /// Partial masking: "john.***@***.com"
    PartialMask { visible_chars: usize },

    /// Hash for consistency: "user_abc123"
    ConsistentHash { prefix: String },

    /// Remove entirely
    Remove,

    /// Tokenize (reversible with key)
    Tokenize { key: SecretKey },
}

pub struct Redactor {
    strategies: HashMap<PiiType, RedactionStrategy>,
    default_strategy: RedactionStrategy,
}

impl Redactor {
    pub fn redact(&self, text: &str, matches: &[PiiMatch]) -> String {
        let mut result = text.to_string();

        // Process matches in reverse order to maintain positions
        for pii in matches.iter().rev() {
            let strategy = self.strategies
                .get(&pii.pii_type)
                .unwrap_or(&self.default_strategy);

            let replacement = strategy.apply(&text[pii.span.clone()], pii);
            result.replace_range(pii.span.clone(), &replacement);
        }

        result
    }
}
```

---

## 2. Regulatory Compliance

### 2.1 GDPR Requirements for AI Agents

The General Data Protection Regulation applies to agents processing EU residents' data.

#### **Key GDPR Principles**

```rust
/// GDPR compliance requirements mapped to agent behaviors
pub struct GdprCompliance {
    /// Lawful basis for processing
    lawful_basis: LawfulBasis,

    /// Data subject rights handlers
    rights_handlers: DataSubjectRights,

    /// Data protection impact assessment
    dpia: Option<Dpia>,

    /// Data protection officer contact
    dpo: Option<DpoContact>,
}

#[derive(Debug, Clone)]
pub enum LawfulBasis {
    /// User gave explicit consent
    Consent {
        timestamp: DateTime<Utc>,
        scope: Vec<ProcessingPurpose>,
        withdrawable: bool,
    },

    /// Necessary for contract performance
    Contract { contract_id: String },

    /// Legal obligation
    LegalObligation { regulation: String },

    /// Vital interests
    VitalInterests,

    /// Public interest
    PublicInterest,

    /// Legitimate interest (requires balancing test)
    LegitimateInterest {
        interest: String,
        balancing_test: BalancingTest,
    },
}
```

#### **Data Subject Rights Implementation**

```rust
pub struct DataSubjectRights {
    /// Right to access (Article 15)
    access: AccessHandler,

    /// Right to rectification (Article 16)
    rectification: RectificationHandler,

    /// Right to erasure (Article 17)
    erasure: ErasureHandler,

    /// Right to data portability (Article 20)
    portability: PortabilityHandler,

    /// Right to object (Article 21)
    objection: ObjectionHandler,

    /// Rights related to automated decision-making (Article 22)
    automated_decisions: AutomatedDecisionHandler,
}

impl DataSubjectRights {
    /// Handle access request - return all data held about subject
    pub async fn handle_access_request(
        &self,
        subject_id: &SubjectId,
    ) -> Result<SubjectAccessReport, RightsError> {
        let mut report = SubjectAccessReport::new(subject_id);

        // Collect from all data stores
        for store in &self.data_stores {
            let data = store.find_by_subject(subject_id).await?;
            for item in data {
                report.add_data_item(DataItem {
                    category: item.category(),
                    source: item.source(),
                    purpose: item.purpose(),
                    retention: item.retention_period(),
                    recipients: item.recipients(),
                    data: item.export_readable(),
                });
            }
        }

        Ok(report)
    }

    /// Handle erasure request - right to be forgotten
    pub async fn handle_erasure_request(
        &self,
        subject_id: &SubjectId,
        scope: ErasureScope,
    ) -> Result<ErasureConfirmation, RightsError> {
        let mut confirmation = ErasureConfirmation::new();

        for store in &self.data_stores {
            let erased = store.erase_subject_data(subject_id, &scope).await?;
            confirmation.add_erased(erased);
        }

        // Also erase from agent memory/context
        self.agent_memory.forget_subject(subject_id).await?;

        // Notify data processors
        for processor in &self.data_processors {
            processor.notify_erasure(subject_id).await?;
        }

        Ok(confirmation)
    }

    /// Handle objection to automated decision-making
    pub async fn handle_automated_decision_objection(
        &self,
        subject_id: &SubjectId,
        decision_id: &DecisionId,
    ) -> Result<HumanReviewResult, RightsError> {
        // Queue for human review
        let review = HumanReview {
            subject_id: subject_id.clone(),
            decision_id: decision_id.clone(),
            original_decision: self.get_decision(decision_id).await?,
            requested_at: Utc::now(),
        };

        self.human_review_queue.submit(review).await
    }
}
```

#### **Consent Management**

```rust
pub struct ConsentManager {
    /// Active consents by subject
    consents: HashMap<SubjectId, Vec<Consent>>,

    /// Consent history for audit
    history: ConsentHistory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consent {
    /// Unique consent ID
    id: ConsentId,

    /// What processing is consented to
    purposes: Vec<ProcessingPurpose>,

    /// What data categories are covered
    data_categories: Vec<DataCategory>,

    /// When consent was given
    granted_at: DateTime<Utc>,

    /// How consent was obtained
    mechanism: ConsentMechanism,

    /// Is consent still active
    active: bool,

    /// When/if consent was withdrawn
    withdrawn_at: Option<DateTime<Utc>>,
}

impl ConsentManager {
    /// Check if processing is allowed
    pub fn is_processing_allowed(
        &self,
        subject: &SubjectId,
        purpose: &ProcessingPurpose,
        data_category: &DataCategory,
    ) -> bool {
        self.consents
            .get(subject)
            .map(|consents| {
                consents.iter().any(|c| {
                    c.active
                        && c.purposes.contains(purpose)
                        && c.data_categories.contains(data_category)
                })
            })
            .unwrap_or(false)
    }

    /// Withdraw consent
    pub async fn withdraw(
        &mut self,
        subject: &SubjectId,
        consent_id: &ConsentId,
    ) -> Result<(), ConsentError> {
        if let Some(consents) = self.consents.get_mut(subject) {
            for consent in consents.iter_mut() {
                if &consent.id == consent_id {
                    consent.active = false;
                    consent.withdrawn_at = Some(Utc::now());

                    // Log withdrawal
                    self.history.record(ConsentEvent::Withdrawn {
                        consent_id: consent_id.clone(),
                        timestamp: Utc::now(),
                    });

                    // Trigger data cleanup for withdrawn consent
                    self.trigger_cleanup(subject, consent).await?;

                    return Ok(());
                }
            }
        }

        Err(ConsentError::NotFound)
    }
}
```

### 2.2 CCPA Requirements

The California Consumer Privacy Act has similar but distinct requirements.

```rust
pub struct CcpaCompliance {
    /// Consumer rights handlers
    rights: CcpaConsumerRights,

    /// Do Not Sell tracking
    do_not_sell: DoNotSellRegistry,

    /// Financial incentive disclosures
    incentives: FinancialIncentives,
}

pub struct CcpaConsumerRights {
    /// Right to know what data is collected
    know: KnowHandler,

    /// Right to delete
    delete: DeleteHandler,

    /// Right to opt-out of sale
    opt_out: OptOutHandler,

    /// Right to non-discrimination
    non_discrimination: NonDiscriminationPolicy,
}

impl CcpaCompliance {
    /// Handle "Do Not Sell My Personal Information" request
    pub async fn handle_do_not_sell(
        &mut self,
        consumer: &ConsumerId,
    ) -> Result<DnsConfirmation, CcpaError> {
        // Register opt-out
        self.do_not_sell.register(consumer).await?;

        // Stop any active data sales
        for sale_partner in &mut self.sale_partners {
            sale_partner.stop_sharing(consumer).await?;
        }

        // Update agent behavior
        self.agent.set_sale_restriction(consumer, true).await?;

        Ok(DnsConfirmation {
            consumer: consumer.clone(),
            effective_date: Utc::now(),
        })
    }
}
```

### 2.3 Audit Requirements

#### **Comprehensive Audit Logging**

```rust
pub struct AuditLog {
    /// Log storage backend
    storage: Box<dyn AuditStorage>,

    /// Event serializer
    serializer: AuditSerializer,

    /// Integrity verification
    integrity: IntegrityVerifier,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    /// Unique event ID
    id: AuditEventId,

    /// When the event occurred
    timestamp: DateTime<Utc>,

    /// What type of event
    event_type: AuditEventType,

    /// Who performed the action
    actor: Actor,

    /// What was affected
    resource: Resource,

    /// What action was taken
    action: Action,

    /// Outcome of the action
    outcome: Outcome,

    /// Additional context
    context: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub enum AuditEventType {
    /// Data access events
    DataAccess {
        data_category: DataCategory,
        sensitivity: SensitivityLevel,
    },

    /// Data modification
    DataModification {
        operation: ModificationOperation,
    },

    /// Consent changes
    ConsentChange {
        change_type: ConsentChangeType,
    },

    /// Rights requests
    RightsRequest {
        right: DataSubjectRight,
        status: RequestStatus,
    },

    /// Agent decisions
    AgentDecision {
        decision_type: DecisionType,
        automated: bool,
    },

    /// Security events
    SecurityEvent {
        severity: SecuritySeverity,
    },
}

impl AuditLog {
    /// Record an audit event with integrity protection
    pub async fn record(&self, event: AuditEvent) -> Result<(), AuditError> {
        // Serialize event
        let serialized = self.serializer.serialize(&event)?;

        // Add integrity hash
        let hash = self.integrity.hash(&serialized);
        let record = AuditRecord {
            event: serialized,
            hash,
            previous_hash: self.integrity.get_chain_tip().await?,
        };

        // Store with append-only guarantee
        self.storage.append(record).await?;

        Ok(())
    }

    /// Generate compliance report
    pub async fn generate_report(
        &self,
        filter: AuditFilter,
    ) -> Result<ComplianceReport, AuditError> {
        let events = self.storage.query(filter).await?;

        let report = ComplianceReport {
            period: filter.time_range,
            total_events: events.len(),
            by_type: self.group_by_type(&events),
            data_access_summary: self.summarize_data_access(&events),
            rights_requests: self.summarize_rights_requests(&events),
            security_incidents: self.summarize_security(&events),
        };

        Ok(report)
    }
}
```

### 2.4 Cross-Border Data Transfer

```rust
pub struct DataTransferPolicy {
    /// Approved transfer mechanisms
    mechanisms: Vec<TransferMechanism>,

    /// Country adequacy decisions
    adequacy: HashMap<Country, AdequacyStatus>,

    /// Standard contractual clauses
    sccs: Vec<StandardContractualClause>,
}

#[derive(Debug, Clone)]
pub enum TransferMechanism {
    /// EU adequacy decision
    AdequacyDecision { country: Country },

    /// Standard Contractual Clauses
    Sccs { clause_set: SccSet },

    /// Binding Corporate Rules
    Bcr { approval_id: String },

    /// Explicit consent
    ExplicitConsent,

    /// Necessary for contract
    ContractNecessity,
}

impl DataTransferPolicy {
    /// Check if transfer to destination is allowed
    pub fn can_transfer(
        &self,
        destination: &Country,
        data_category: &DataCategory,
    ) -> TransferDecision {
        // Check adequacy first
        if let Some(AdequacyStatus::Adequate) = self.adequacy.get(destination) {
            return TransferDecision::Allowed {
                mechanism: TransferMechanism::AdequacyDecision {
                    country: destination.clone()
                },
            };
        }

        // Check for applicable SCCs
        for scc in &self.sccs {
            if scc.covers(destination, data_category) {
                return TransferDecision::Allowed {
                    mechanism: TransferMechanism::Sccs {
                        clause_set: scc.clause_set.clone()
                    },
                };
            }
        }

        TransferDecision::RequiresAdditionalSafeguards
    }
}
```

---

## 3. Privacy-Enhancing Techniques

### 3.1 Differential Privacy

Differential privacy provides mathematical guarantees about privacy protection.

#### **Core Concepts**

```rust
/// Differential privacy parameters
pub struct DpParameters {
    /// Privacy budget (epsilon)
    /// Lower = more privacy, less utility
    epsilon: f64,

    /// Delta for approximate DP
    /// Probability of privacy failure
    delta: f64,

    /// Sensitivity of the query
    sensitivity: f64,
}

/// Noise mechanisms for differential privacy
pub trait NoiseMechanism {
    /// Add noise to a value
    fn add_noise(&self, value: f64, sensitivity: f64) -> f64;

    /// Get the privacy cost (epsilon)
    fn privacy_cost(&self) -> f64;
}

/// Laplace mechanism for numeric queries
pub struct LaplaceMechanism {
    epsilon: f64,
}

impl NoiseMechanism for LaplaceMechanism {
    fn add_noise(&self, value: f64, sensitivity: f64) -> f64 {
        let scale = sensitivity / self.epsilon;
        let noise = Laplace::new(0.0, scale).sample(&mut rand::thread_rng());
        value + noise
    }

    fn privacy_cost(&self) -> f64 {
        self.epsilon
    }
}

/// Gaussian mechanism for (epsilon, delta)-DP
pub struct GaussianMechanism {
    epsilon: f64,
    delta: f64,
}

impl NoiseMechanism for GaussianMechanism {
    fn add_noise(&self, value: f64, sensitivity: f64) -> f64 {
        let sigma = sensitivity * (2.0 * (1.25 / self.delta).ln()).sqrt() / self.epsilon;
        let noise = Normal::new(0.0, sigma).sample(&mut rand::thread_rng());
        value + noise
    }

    fn privacy_cost(&self) -> f64 {
        self.epsilon
    }
}
```

#### **Privacy Budget Management**

```rust
/// Track and manage privacy budget across queries
pub struct PrivacyAccountant {
    /// Total budget
    total_epsilon: f64,
    total_delta: f64,

    /// Spent budget
    spent_epsilon: f64,
    spent_delta: f64,

    /// Query history
    queries: Vec<PrivateQuery>,
}

impl PrivacyAccountant {
    /// Check if we can afford a query
    pub fn can_afford(&self, epsilon: f64, delta: f64) -> bool {
        self.spent_epsilon + epsilon <= self.total_epsilon
            && self.spent_delta + delta <= self.total_delta
    }

    /// Execute a private query if budget allows
    pub fn query<T, F>(&mut self, epsilon: f64, delta: f64, query: F) -> Result<T, PrivacyError>
    where
        F: FnOnce() -> T,
    {
        if !self.can_afford(epsilon, delta) {
            return Err(PrivacyError::BudgetExhausted);
        }

        let result = query();

        // Deduct from budget
        self.spent_epsilon += epsilon;
        self.spent_delta += delta;

        self.queries.push(PrivateQuery {
            timestamp: Utc::now(),
            epsilon,
            delta,
        });

        Ok(result)
    }

    /// Reset budget (e.g., for new time period)
    pub fn reset(&mut self) {
        self.spent_epsilon = 0.0;
        self.spent_delta = 0.0;
        self.queries.clear();
    }
}
```

#### **DP for Agent Memory**

```rust
/// Differentially private memory store
pub struct DpMemoryStore {
    /// Underlying storage
    storage: Box<dyn MemoryStorage>,

    /// Privacy accountant
    accountant: PrivacyAccountant,

    /// Noise mechanism
    mechanism: Box<dyn NoiseMechanism>,
}

impl DpMemoryStore {
    /// Store with local DP (noise before storage)
    pub async fn store_with_ldp(
        &mut self,
        key: &str,
        value: f64,
        sensitivity: f64,
    ) -> Result<(), PrivacyError> {
        let noisy_value = self.mechanism.add_noise(value, sensitivity);
        self.storage.store(key, noisy_value).await?;
        Ok(())
    }

    /// Aggregate query with DP
    pub async fn private_count(
        &mut self,
        predicate: impl Fn(&Record) -> bool,
    ) -> Result<f64, PrivacyError> {
        self.accountant.query(self.mechanism.privacy_cost(), 0.0, || {
            let count = self.storage.count(predicate);
            self.mechanism.add_noise(count as f64, 1.0)
        })
    }
}
```

### 3.2 Federated Learning Approaches

Federated learning keeps data on-device while enabling model improvement.

#### **Federated Architecture for Agents**

```rust
/// Federated learning coordinator
pub struct FederatedCoordinator {
    /// Current global model
    global_model: Model,

    /// Participating clients
    clients: Vec<ClientId>,

    /// Aggregation strategy
    aggregator: Box<dyn Aggregator>,

    /// Privacy enhancements
    privacy: FederatedPrivacy,
}

/// Privacy enhancements for federated learning
pub struct FederatedPrivacy {
    /// Add noise to gradients
    gradient_noise: Option<DpParameters>,

    /// Secure aggregation
    secure_aggregation: bool,

    /// Gradient clipping
    gradient_clipping: Option<f64>,
}

impl FederatedCoordinator {
    /// Execute one round of federated learning
    pub async fn training_round(&mut self) -> Result<RoundResult, FederatedError> {
        // Select participants for this round
        let participants = self.select_participants();

        // Distribute current model
        for client in &participants {
            self.send_model(client, &self.global_model).await?;
        }

        // Collect updates
        let mut updates = Vec::new();
        for client in &participants {
            let update = self.receive_update(client).await?;

            // Apply gradient clipping
            let clipped = if let Some(max_norm) = self.privacy.gradient_clipping {
                clip_gradient(&update, max_norm)
            } else {
                update
            };

            updates.push(clipped);
        }

        // Aggregate updates
        let aggregated = if self.privacy.secure_aggregation {
            self.aggregator.secure_aggregate(&updates).await?
        } else {
            self.aggregator.aggregate(&updates)
        };

        // Apply DP noise to aggregated update
        let private_update = if let Some(dp) = &self.privacy.gradient_noise {
            add_gradient_noise(&aggregated, dp)
        } else {
            aggregated
        };

        // Update global model
        self.global_model.apply_update(&private_update);

        Ok(RoundResult {
            participants: participants.len(),
            model_version: self.global_model.version,
        })
    }
}
```

#### **On-Device Processing for Agents**

```rust
/// Agent that processes sensitive data locally
pub struct LocalFirstAgent {
    /// Local model for sensitive operations
    local_model: LocalModel,

    /// What requires local processing
    local_processing_policy: LocalProcessingPolicy,

    /// Sync manager for non-sensitive updates
    sync: SyncManager,
}

impl LocalFirstAgent {
    /// Process user input with local-first privacy
    pub async fn process(&self, input: &str) -> Result<Response, AgentError> {
        // Detect sensitivity
        let sensitivity = self.classify_sensitivity(input);

        match sensitivity {
            SensitivityLevel::Restricted | SensitivityLevel::Sensitive => {
                // Process entirely locally
                let response = self.local_model.generate(input).await?;

                // Only sync aggregated, anonymous feedback
                self.sync.queue_anonymous_feedback(&response).await?;

                Ok(response)
            }
            _ => {
                // Can use cloud model for less sensitive data
                self.cloud_model.generate(input).await
            }
        }
    }
}
```

### 3.3 Privacy by Design Principles

#### **The Seven Foundational Principles**

```rust
/// Privacy by Design implementation
pub struct PrivacyByDesign {
    /// 1. Proactive not Reactive
    proactive: ProactivePrivacy,

    /// 2. Privacy as the Default
    default_privacy: DefaultPrivacySettings,

    /// 3. Privacy Embedded into Design
    embedded: EmbeddedPrivacyControls,

    /// 4. Full Functionality (positive-sum)
    full_functionality: FunctionalityPreservation,

    /// 5. End-to-End Security
    security: EndToEndSecurity,

    /// 6. Visibility and Transparency
    transparency: TransparencyControls,

    /// 7. Respect for User Privacy
    user_centric: UserCentricDesign,
}

/// Privacy-by-default configuration
pub struct DefaultPrivacySettings {
    /// Default data collection level
    collection: CollectionLevel,

    /// Default sharing settings
    sharing: SharingDefault,

    /// Default retention
    retention: RetentionDefault,

    /// Default visibility
    visibility: VisibilityDefault,
}

impl Default for DefaultPrivacySettings {
    fn default() -> Self {
        Self {
            // Collect minimum by default
            collection: CollectionLevel::Minimum,
            // Share nothing by default
            sharing: SharingDefault::None,
            // Short retention by default
            retention: RetentionDefault::ShortTerm,
            // Private by default
            visibility: VisibilityDefault::Private,
        }
    }
}
```

#### **Privacy Controls Architecture**

```rust
/// Embedded privacy controls for agents
pub struct AgentPrivacyControls {
    /// Pre-processing filters
    input_filters: Vec<Box<dyn InputFilter>>,

    /// Output sanitization
    output_sanitizers: Vec<Box<dyn OutputSanitizer>>,

    /// Memory privacy controls
    memory_controls: MemoryPrivacyControls,

    /// Tool execution controls
    tool_controls: ToolPrivacyControls,

    /// Transparency interface
    transparency: TransparencyInterface,
}

impl AgentPrivacyControls {
    /// Apply privacy controls to agent input
    pub async fn filter_input(&self, input: AgentInput) -> Result<AgentInput, PrivacyError> {
        let mut filtered = input;

        for filter in &self.input_filters {
            filtered = filter.apply(filtered).await?;
        }

        Ok(filtered)
    }

    /// Apply privacy controls to agent output
    pub async fn sanitize_output(&self, output: AgentOutput) -> Result<AgentOutput, PrivacyError> {
        let mut sanitized = output;

        for sanitizer in &self.output_sanitizers {
            sanitized = sanitizer.apply(sanitized).await?;
        }

        Ok(sanitized)
    }
}

/// Tool execution privacy controls
pub struct ToolPrivacyControls {
    /// Allowed tools per sensitivity level
    tool_permissions: HashMap<SensitivityLevel, HashSet<ToolId>>,

    /// Data flow restrictions
    data_flow: DataFlowPolicy,

    /// External API restrictions
    external_api: ExternalApiPolicy,
}

impl ToolPrivacyControls {
    /// Check if tool execution is allowed given data sensitivity
    pub fn can_execute(
        &self,
        tool: &ToolId,
        data_sensitivity: SensitivityLevel,
    ) -> bool {
        self.tool_permissions
            .get(&data_sensitivity)
            .map(|allowed| allowed.contains(tool))
            .unwrap_or(false)
    }

    /// Check if data can flow to external API
    pub fn can_send_external(
        &self,
        api: &ExternalApi,
        data: &SensitiveData<impl Any>,
    ) -> bool {
        self.external_api.allows(api)
            && self.data_flow.allows_external(data.sensitivity)
    }
}
```

### 3.4 Encryption and Secure Computation

```rust
/// End-to-end encryption for agent data
pub struct AgentEncryption {
    /// Key management
    key_manager: KeyManager,

    /// Encryption algorithms
    algorithms: EncryptionAlgorithms,

    /// Secure enclaves (if available)
    enclave: Option<SecureEnclave>,
}

#[derive(Debug, Clone)]
pub struct EncryptionAlgorithms {
    /// Symmetric encryption for data at rest
    symmetric: SymmetricAlgorithm,

    /// Asymmetric for key exchange
    asymmetric: AsymmetricAlgorithm,

    /// Hashing for integrity
    hash: HashAlgorithm,
}

impl AgentEncryption {
    /// Encrypt sensitive data for storage
    pub fn encrypt_for_storage(
        &self,
        data: &[u8],
        sensitivity: SensitivityLevel,
    ) -> Result<EncryptedData, CryptoError> {
        let key = self.key_manager.get_key_for_sensitivity(sensitivity)?;

        let nonce = generate_nonce();
        let ciphertext = self.algorithms.symmetric.encrypt(&key, &nonce, data)?;

        Ok(EncryptedData {
            ciphertext,
            nonce,
            sensitivity,
            algorithm: self.algorithms.symmetric.id(),
        })
    }

    /// Encrypt for transmission to another party
    pub fn encrypt_for_recipient(
        &self,
        data: &[u8],
        recipient_public_key: &PublicKey,
    ) -> Result<EncryptedTransmission, CryptoError> {
        // Generate ephemeral key pair
        let ephemeral = self.algorithms.asymmetric.generate_keypair()?;

        // Derive shared secret
        let shared_secret = self.algorithms.asymmetric.derive_shared(
            &ephemeral.private,
            recipient_public_key,
        )?;

        // Encrypt with derived key
        let nonce = generate_nonce();
        let ciphertext = self.algorithms.symmetric.encrypt(&shared_secret, &nonce, data)?;

        Ok(EncryptedTransmission {
            ephemeral_public: ephemeral.public,
            nonce,
            ciphertext,
        })
    }
}

/// Secure memory handling
pub struct SecureMemory {
    /// Memory that's wiped on drop
    inner: ZeroizeOnDrop<Vec<u8>>,

    /// Lock in memory (prevent swapping)
    locked: bool,
}

impl SecureMemory {
    pub fn new(data: Vec<u8>) -> Self {
        let mut mem = Self {
            inner: ZeroizeOnDrop(data),
            locked: false,
        };

        // Try to lock memory
        #[cfg(unix)]
        {
            if mlock(mem.inner.as_ptr(), mem.inner.len()).is_ok() {
                mem.locked = true;
            }
        }

        mem
    }
}

impl Drop for SecureMemory {
    fn drop(&mut self) {
        // Unlock if locked
        #[cfg(unix)]
        if self.locked {
            let _ = munlock(self.inner.as_ptr(), self.inner.len());
        }
        // ZeroizeOnDrop handles zeroing
    }
}
```

---

## 4. Privacy Architecture for Agentic Framework

### 4.1 Layered Privacy Architecture

```rust
/// Core privacy module for the agentic framework
pub struct PrivacyModule {
    /// PII detection and handling
    pii: PiiHandler,

    /// Compliance management
    compliance: ComplianceManager,

    /// Privacy-enhancing techniques
    pet: PrivacyEnhancingTechniques,

    /// Audit and transparency
    audit: AuditSystem,

    /// User privacy preferences
    preferences: UserPreferences,
}

impl PrivacyModule {
    /// Create with default privacy-preserving settings
    pub fn privacy_first() -> Self {
        Self {
            pii: PiiHandler::strict(),
            compliance: ComplianceManager::gdpr_ccpa(),
            pet: PrivacyEnhancingTechniques::default(),
            audit: AuditSystem::comprehensive(),
            preferences: UserPreferences::conservative(),
        }
    }

    /// Wrap an agent with privacy protections
    pub fn wrap_agent<A: Agent>(&self, agent: A) -> PrivacyWrappedAgent<A> {
        PrivacyWrappedAgent {
            inner: agent,
            privacy: self.clone(),
        }
    }
}

/// Agent wrapped with privacy protections
pub struct PrivacyWrappedAgent<A: Agent> {
    inner: A,
    privacy: PrivacyModule,
}

impl<A: Agent> Agent for PrivacyWrappedAgent<A> {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput, AgentError> {
        // Pre-process: detect and handle PII
        let filtered_input = self.privacy.pii.filter_input(&input).await?;

        // Check consent
        if !self.privacy.compliance.check_consent(&filtered_input).await? {
            return Err(AgentError::ConsentRequired);
        }

        // Log access
        self.privacy.audit.log_input_access(&filtered_input).await?;

        // Execute agent
        let output = self.inner.process(filtered_input).await?;

        // Post-process: sanitize output
        let sanitized = self.privacy.pii.sanitize_output(&output).await?;

        // Log output
        self.privacy.audit.log_output(&sanitized).await?;

        Ok(sanitized)
    }
}
```

### 4.2 Rust Patterns for Privacy

#### **Type-Level Privacy Enforcement**

```rust
/// Marker traits for data sensitivity
pub trait Sensitivity {}
pub struct Public;
pub struct Confidential;
pub struct Sensitive;
pub struct Restricted;

impl Sensitivity for Public {}
impl Sensitivity for Confidential {}
impl Sensitivity for Sensitive {}
impl Sensitivity for Restricted {}

/// Data with compile-time sensitivity tracking
pub struct Classified<T, S: Sensitivity> {
    data: T,
    _marker: PhantomData<S>,
}

impl<T> Classified<T, Public> {
    /// Public data can be freely converted
    pub fn into_inner(self) -> T {
        self.data
    }
}

impl<T, S: Sensitivity> Classified<T, S> {
    /// Access requires explicit acknowledgment
    pub fn access<R>(self, f: impl FnOnce(T) -> R) -> Classified<R, S> {
        Classified {
            data: f(self.data),
            _marker: PhantomData,
        }
    }

    /// Downgrade sensitivity only with proof of authorization
    pub fn declassify<NewS: Sensitivity>(
        self,
        _proof: DeclassificationProof<S, NewS>,
    ) -> Classified<T, NewS> {
        Classified {
            data: self.data,
            _marker: PhantomData,
        }
    }
}

/// Proof that declassification is authorized
pub struct DeclassificationProof<From: Sensitivity, To: Sensitivity> {
    _from: PhantomData<From>,
    _to: PhantomData<To>,
}

impl<From: Sensitivity, To: Sensitivity> DeclassificationProof<From, To> {
    /// Only constructible with proper authorization
    pub(crate) fn new(_authorization: AuthorizationToken) -> Self {
        Self {
            _from: PhantomData,
            _to: PhantomData,
        }
    }
}
```

#### **Secure String Types**

```rust
/// String that auto-redacts in Debug output
#[derive(Clone)]
pub struct SecretString {
    inner: Zeroizing<String>,
}

impl SecretString {
    pub fn new(s: impl Into<String>) -> Self {
        Self {
            inner: Zeroizing::new(s.into()),
        }
    }

    /// Expose with explicit acknowledgment
    pub fn expose_secret(&self) -> &str {
        &self.inner
    }
}

impl Debug for SecretString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SecretString(***)")
    }
}

impl Display for SecretString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "***")
    }
}

/// String that tracks its data lineage
pub struct TrackedString {
    content: String,
    origin: DataOrigin,
    transformations: Vec<Transformation>,
}

impl TrackedString {
    pub fn transform(&mut self, name: &str, f: impl FnOnce(&str) -> String) {
        self.content = f(&self.content);
        self.transformations.push(Transformation {
            name: name.to_string(),
            timestamp: Utc::now(),
        });
    }

    pub fn lineage(&self) -> &DataOrigin {
        &self.origin
    }
}
```

#### **Privacy-Safe Builders**

```rust
/// Builder that enforces privacy considerations
pub struct AgentBuilder {
    config: AgentConfig,
    privacy_reviewed: bool,
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self {
            config: AgentConfig::default(),
            privacy_reviewed: false,
        }
    }

    /// Must acknowledge privacy considerations
    pub fn with_privacy_review(mut self, review: PrivacyReview) -> Self {
        self.config.privacy = review.into_config();
        self.privacy_reviewed = true;
        self
    }

    /// Build fails without privacy review
    pub fn build(self) -> Result<Agent, BuildError> {
        if !self.privacy_reviewed {
            return Err(BuildError::PrivacyReviewRequired);
        }

        Ok(Agent::from_config(self.config))
    }
}

pub struct PrivacyReview {
    /// What PII will be processed
    pii_types: Vec<PiiType>,

    /// Legal basis for processing
    legal_basis: LawfulBasis,

    /// Retention policy
    retention: RetentionPolicy,

    /// Data protection measures
    protections: Vec<ProtectionMeasure>,
}
```

---

## 5. Privacy Checklist for Agents

### 5.1 Design Phase Checklist

```
[ ] Data Inventory
    [ ] Document all PII types that will be processed
    [ ] Classify data by sensitivity level
    [ ] Map data flows through the system
    [ ] Identify all storage locations

[ ] Privacy Impact Assessment
    [ ] Identify privacy risks
    [ ] Evaluate necessity and proportionality
    [ ] Document mitigating controls
    [ ] Get stakeholder sign-off

[ ] Legal Basis
    [ ] Determine lawful basis for each processing activity
    [ ] Design consent mechanisms if required
    [ ] Document legitimate interest assessments
    [ ] Plan for cross-border transfers

[ ] Privacy by Design
    [ ] Default to minimum data collection
    [ ] Build in data minimization
    [ ] Plan for data subject rights
    [ ] Design audit logging
```

### 5.2 Implementation Phase Checklist

```
[ ] PII Handling
    [ ] Implement PII detection
    [ ] Add redaction capabilities
    [ ] Set up sensitivity classification
    [ ] Create data handling policies

[ ] Security Controls
    [ ] Encrypt data at rest
    [ ] Encrypt data in transit
    [ ] Implement access controls
    [ ] Secure key management

[ ] Consent Management
    [ ] Implement consent collection
    [ ] Track consent status
    [ ] Enable consent withdrawal
    [ ] Propagate consent changes

[ ] Data Subject Rights
    [ ] Implement access request handler
    [ ] Implement erasure handler
    [ ] Implement portability export
    [ ] Implement objection handling
```

### 5.3 Runtime Checklist

```
[ ] Input Processing
    [ ] Scan all inputs for PII
    [ ] Apply collection policies
    [ ] Verify consent before processing
    [ ] Log data access

[ ] Agent Memory
    [ ] Apply retention policies
    [ ] Encrypt sensitive memories
    [ ] Enable memory erasure
    [ ] Track data lineage

[ ] Tool Execution
    [ ] Validate data before external calls
    [ ] Redact sensitive data in API calls
    [ ] Log tool invocations
    [ ] Enforce data flow policies

[ ] Output Generation
    [ ] Scan outputs for PII leakage
    [ ] Apply output sanitization
    [ ] Verify data sharing permissions
    [ ] Log data disclosures
```

### 5.4 Audit and Compliance Checklist

```
[ ] Logging
    [ ] Log all data access events
    [ ] Log consent changes
    [ ] Log rights request processing
    [ ] Maintain audit trail integrity

[ ] Reporting
    [ ] Generate data inventory reports
    [ ] Track rights request metrics
    [ ] Monitor privacy incidents
    [ ] Produce compliance evidence

[ ] Testing
    [ ] Test PII detection accuracy
    [ ] Test erasure completeness
    [ ] Test consent enforcement
    [ ] Pen test privacy controls

[ ] Incident Response
    [ ] Define breach notification procedures
    [ ] Establish response team
    [ ] Document incident playbooks
    [ ] Test response procedures
```

---

## 6. Recommended Implementation Priorities

### Phase 1: Foundation (Week 1-2)

1. **PII Detection Module**
   - Pattern-based detection for common PII types
   - Sensitivity classification enum
   - Redaction utilities

2. **Secure Data Types**
   - `SecretString` with auto-zeroization
   - `SensitiveData<T>` wrapper with access tracking
   - Debug-safe formatting

3. **Basic Audit Logging**
   - Event structure
   - File-based logging
   - Access event recording

### Phase 2: Compliance (Week 3-4)

4. **Consent Management**
   - Consent model
   - Storage and retrieval
   - Consent checking middleware

5. **Data Subject Rights**
   - Access request handling
   - Erasure implementation
   - Export/portability

6. **Retention Management**
   - Policy definition
   - Automatic cleanup
   - Secure deletion

### Phase 3: Advanced Privacy (Week 5-6)

7. **Privacy Wrapper**
   - Agent wrapping trait
   - Input/output filtering
   - Privacy-by-default configuration

8. **Differential Privacy**
   - Noise mechanisms
   - Budget tracking
   - DP aggregations

9. **Encryption**
   - At-rest encryption
   - Key management
   - Secure memory handling

### Phase 4: Production Readiness (Week 7-8)

10. **Comprehensive Audit**
    - Tamper-evident logging
    - Compliance reporting
    - Dashboard integration

11. **Testing and Validation**
    - Privacy test suite
    - Compliance validation
    - Penetration testing

12. **Documentation**
    - Privacy policy templates
    - Developer guidelines
    - Compliance documentation

---

## 7. Open Research Questions

### 7.1 Technical Challenges

1. **Semantic PII Detection**
   - How to detect PII in natural language context?
   - Handling PII in non-standard formats?
   - Multi-language PII detection?

2. **Privacy-Utility Tradeoffs**
   - What epsilon values preserve agent utility?
   - When does DP break agent functionality?
   - Measuring privacy vs. performance?

3. **Memory Privacy**
   - How to "forget" without corrupting agent capabilities?
   - Privacy-preserving memory retrieval?
   - Verifiable deletion in vector stores?

### 7.2 Compliance Questions

1. **AI-Specific Regulations**
   - How will EU AI Act affect agent privacy?
   - What additional requirements are coming?
   - Cross-jurisdictional agent operation?

2. **Automated Decision-Making**
   - When do agent decisions require explanation?
   - Human oversight requirements?
   - Liability for agent privacy violations?

### 7.3 Architecture Questions

1. **Multi-Agent Privacy**
   - Privacy in agent-to-agent communication?
   - Secure multi-party computation for agents?
   - Trust boundaries between agents?

2. **Federated Agents**
   - Local-first agent architectures?
   - On-device vs. cloud processing decisions?
   - Privacy-preserving synchronization?

---

## 8. Summary

Privacy in AI agents requires a comprehensive, multi-layered approach:

1. **PII Handling**: Detect, classify, and protect sensitive data at every touchpoint
2. **Compliance**: Build GDPR/CCPA compliance into the core architecture, not as an afterthought
3. **Privacy Techniques**: Apply differential privacy, encryption, and privacy-by-design principles
4. **Audit**: Maintain comprehensive, tamper-evident logs for accountability

**Key Takeaways for the Agentic Framework**:

- Privacy must be a first-class concern, not an optional add-on
- Rust's type system enables compile-time privacy enforcement
- Default to maximum privacy protection; require explicit opt-out
- Build for regulatory compliance from day one
- Plan for data subject rights (access, erasure, portability)
- Log everything, encrypt everything, minimize everything

The patterns and architectures in this report provide a foundation for building privacy-respecting agents that can operate in regulated environments while maintaining user trust.

---

## References

### Regulations
- General Data Protection Regulation (EU 2016/679)
- California Consumer Privacy Act (Cal. Civ. Code 1798.100-199)
- EU AI Act (pending)

### Technical Standards
- NIST Privacy Framework
- ISO 27701 (Privacy Information Management)
- OWASP Privacy Guidelines

### Academic Papers
- Dwork, C. "Differential Privacy" (2006)
- McMahan et al. "Communication-Efficient Learning of Deep Networks from Decentralized Data" (2017)
- Cavoukian, A. "Privacy by Design: The 7 Foundational Principles" (2009)

### Rust Libraries
- `zeroize` - Secure memory zeroing
- `secrecy` - Secret-keeping types
- `ring` / `rustls` - Cryptography
