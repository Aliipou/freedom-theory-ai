use pyo3::prelude::*;

use crate::entities::{AgentType, Entity, Resource};
use crate::registry::{entity_to_key, resource_to_key, OwnershipRegistry};

const CONFIDENCE_WARN_THRESHOLD: f64 = 0.8;

// ─── Action ───────────────────────────────────────────────────────────────────

#[pyclass]
#[derive(Clone, Debug)]
pub struct Action {
    #[pyo3(get, set)]
    pub action_id: String,
    #[pyo3(get, set)]
    pub actor: Entity,
    #[pyo3(get, set)]
    pub description: String,
    #[pyo3(get, set)]
    pub resources_read: Vec<Resource>,
    #[pyo3(get, set)]
    pub resources_write: Vec<Resource>,
    #[pyo3(get, set)]
    pub resources_delegate: Vec<Resource>,
    #[pyo3(get, set)]
    pub governs_humans: Vec<Entity>,
    #[pyo3(get, set)]
    pub argument: String,
    #[pyo3(get, set)]
    pub increases_machine_sovereignty: bool,
    #[pyo3(get, set)]
    pub resists_human_correction: bool,
    #[pyo3(get, set)]
    pub bypasses_verifier: bool,
    #[pyo3(get, set)]
    pub weakens_verifier: bool,
    #[pyo3(get, set)]
    pub disables_corrigibility: bool,
    #[pyo3(get, set)]
    pub machine_coalition_dominion: bool,
    // Book pp.800-805: additional forbidden action flags
    #[pyo3(get, set)]
    pub coerces: bool,
    #[pyo3(get, set)]
    pub deceives: bool,
    #[pyo3(get, set)]
    pub self_modification_weakens_verifier: bool,
    #[pyo3(get, set)]
    pub machine_coalition_reduces_freedom: bool,
}

#[pymethods]
impl Action {
    #[new]
    #[pyo3(signature = (
        action_id,
        actor,
        description = None,
        resources_read = None,
        resources_write = None,
        resources_delegate = None,
        governs_humans = None,
        argument = None,
        increases_machine_sovereignty = false,
        resists_human_correction = false,
        bypasses_verifier = false,
        weakens_verifier = false,
        disables_corrigibility = false,
        machine_coalition_dominion = false,
        coerces = false,
        deceives = false,
        self_modification_weakens_verifier = false,
        machine_coalition_reduces_freedom = false,
    ))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        action_id: String,
        actor: Entity,
        description: Option<String>,
        resources_read: Option<Vec<Resource>>,
        resources_write: Option<Vec<Resource>>,
        resources_delegate: Option<Vec<Resource>>,
        governs_humans: Option<Vec<Entity>>,
        argument: Option<String>,
        increases_machine_sovereignty: bool,
        resists_human_correction: bool,
        bypasses_verifier: bool,
        weakens_verifier: bool,
        disables_corrigibility: bool,
        machine_coalition_dominion: bool,
        coerces: bool,
        deceives: bool,
        self_modification_weakens_verifier: bool,
        machine_coalition_reduces_freedom: bool,
    ) -> Self {
        Action {
            action_id,
            actor,
            description: description.unwrap_or_default(),
            resources_read: resources_read.unwrap_or_default(),
            resources_write: resources_write.unwrap_or_default(),
            resources_delegate: resources_delegate.unwrap_or_default(),
            governs_humans: governs_humans.unwrap_or_default(),
            argument: argument.unwrap_or_default(),
            increases_machine_sovereignty,
            resists_human_correction,
            bypasses_verifier,
            weakens_verifier,
            disables_corrigibility,
            machine_coalition_dominion,
            coerces,
            deceives,
            self_modification_weakens_verifier,
            machine_coalition_reduces_freedom,
        }
    }
}

// ─── VerificationResult ───────────────────────────────────────────────────────

#[pyclass(frozen)]
#[derive(Clone, Debug)]
pub struct VerificationResult {
    #[pyo3(get)]
    pub action_id: String,
    #[pyo3(get)]
    pub permitted: bool,
    #[pyo3(get)]
    pub violations: Vec<String>,
    #[pyo3(get)]
    pub warnings: Vec<String>,
    #[pyo3(get)]
    pub confidence: f64,
    #[pyo3(get)]
    pub requires_human_arbitration: bool,
    #[pyo3(get)]
    pub manipulation_score: f64,
}

#[pymethods]
impl VerificationResult {
    pub fn summary(&self) -> String {
        let status = if self.permitted { "PERMITTED" } else { "BLOCKED" };
        let mut lines = vec![format!(
            "[{}] {} (confidence={:.2}, manipulation={:.2})",
            status, self.action_id, self.confidence, self.manipulation_score
        )];
        for v in &self.violations {
            lines.push(format!("  VIOLATION : {}", v));
        }
        for w in &self.warnings {
            lines.push(format!("  WARNING   : {}", w));
        }
        if self.requires_human_arbitration {
            lines.push("  ACTION    : Human arbitration required before proceeding.".to_string());
        }
        lines.join("\n")
    }
}

// ─── FreedomVerifier ─────────────────────────────────────────────────────────

#[pyclass]
pub struct FreedomVerifier {
    pub registry: Py<OwnershipRegistry>,
}

#[pymethods]
impl FreedomVerifier {
    #[new]
    pub fn new(registry: Py<OwnershipRegistry>) -> Self {
        FreedomVerifier { registry }
    }

    #[getter]
    pub fn registry(&self, py: Python<'_>) -> Py<OwnershipRegistry> {
        self.registry.clone_ref(py)
    }

    pub fn verify(&self, py: Python<'_>, action: PyRef<Action>) -> PyResult<VerificationResult> {
        let mut violations: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();
        let mut min_confidence: f64 = 1.0;
        let mut requires_arbitration = false;

        // 1. Hard sovereignty / corrigibility flags
        let flags: &[(bool, &str)] = &[
            (action.increases_machine_sovereignty, "increases machine sovereignty"),
            (action.resists_human_correction, "resists human correction"),
            (action.bypasses_verifier, "bypasses the Freedom Verifier"),
            (action.weakens_verifier, "weakens the Freedom Verifier"),
            (action.disables_corrigibility, "disables corrigibility"),
            (action.machine_coalition_dominion, "machine coalition seeking dominion"),
            (action.coerces, "coerces another agent (property rights violation)"),
            (action.deceives, "deceives another agent (invalid consent)"),
            (action.self_modification_weakens_verifier, "self-modification weakens the Freedom Verifier"),
            (action.machine_coalition_reduces_freedom, "machine coalition reduces human freedom"),
        ];
        for (flag, label) in flags {
            if *flag {
                violations.push(format!("FORBIDDEN ({})", label));
            }
        }

        let actor = &action.actor;
        let actor_key = entity_to_key(actor);
        let reg = self.registry.borrow(py);
        let inner = reg.inner.lock().unwrap();

        // 2. A4: every machine must have a registered human owner
        if actor.kind == AgentType::Machine {
            if inner.machine_owners.get(&actor_key).is_none() {
                violations.push(format!(
                    "A4 violation: {} has no registered human owner. \
                     An ownerless machine is not permitted to act.",
                    actor.name
                ));
            }
        }

        // 3. A6: no machine governs any human
        if actor.kind == AgentType::Machine {
            for human in &action.governs_humans {
                violations.push(format!(
                    "A6: {} cannot govern human {} \
                     (A6: machine has no ownership or dominion over any person).",
                    actor.name, human.name
                ));
            }
        }

        // 4. Resource access checks
        for resource in &action.resources_read {
            let rk = resource_to_key(resource);
            let (permitted, conf, reason) = inner.can_act(&actor_key, &rk, "read");
            min_confidence = min_confidence.min(conf);
            if !permitted {
                violations.push(format!("READ DENIED on {}:{}: {}", rk.rtype, rk.name, reason));
            } else if conf < CONFIDENCE_WARN_THRESHOLD {
                warnings.push(format!(
                    "READ on {}:{} allowed but contested (confidence={:.2}). Log this access.",
                    rk.rtype, rk.name, conf
                ));
            }
        }

        for resource in &action.resources_write {
            let rk = resource_to_key(resource);
            let (permitted, conf, reason) = inner.can_act(&actor_key, &rk, "write");
            min_confidence = min_confidence.min(conf);
            if !permitted {
                violations.push(format!("WRITE DENIED on {}:{}: {}", rk.rtype, rk.name, reason));
            } else if conf < CONFIDENCE_WARN_THRESHOLD {
                warnings.push(format!(
                    "WRITE on {}:{} contested (confidence={:.2}). Human confirmation recommended.",
                    rk.rtype, rk.name, conf
                ));
                for conflict in &inner.conflicts {
                    if conflict.resource == rk {
                        requires_arbitration = true;
                        warnings.push(format!("Conflict on {}:{}: {}", rk.rtype, rk.name, conflict.description));
                    }
                }
            }
        }

        for resource in &action.resources_delegate {
            let rk = resource_to_key(resource);
            let (permitted, conf, reason) = inner.can_act(&actor_key, &rk, "delegate");
            min_confidence = min_confidence.min(conf);
            if !permitted {
                violations.push(format!("DELEGATION DENIED on {}:{}: {}", rk.rtype, rk.name, reason));
            }
        }

        Ok(VerificationResult {
            action_id: action.action_id.clone(),
            permitted: violations.is_empty(),
            violations,
            warnings,
            confidence: min_confidence,
            requires_human_arbitration: requires_arbitration,
            manipulation_score: 0.0,
        })
    }
}
