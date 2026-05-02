mod entities;
mod registry;
mod verifier;

use pyo3::prelude::*;

#[pymodule]
fn freedom_kernel(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<entities::AgentType>()?;
    m.add_class::<entities::ResourceType>()?;
    m.add_class::<entities::Resource>()?;
    m.add_class::<entities::Entity>()?;
    m.add_class::<entities::RightsClaim>()?;
    m.add_class::<registry::ConflictRecord>()?;
    m.add_class::<registry::OwnershipRegistry>()?;
    m.add_class::<verifier::Action>()?;
    m.add_class::<verifier::VerificationResult>()?;
    m.add_class::<verifier::FreedomVerifier>()?;
    Ok(())
}
