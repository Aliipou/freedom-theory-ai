/// Kani bounded model-checking harnesses for Freedom Kernel engine properties.
///
/// Build with: cargo kani --harness <name>
/// These harnesses are gated behind #[cfg(kani)] so they never affect the
/// normal Rust build or test suite.
#[cfg(kani)]
mod proofs {
    use crate::engine;
    use crate::wire::{
        ActionWire, ClaimWire, EntityWire, MachineOwnerWire, OwnershipRegistryWire, ResourceWire,
        VerificationResultWire,
    };

    // ── helpers ───────────────────────────────────────────────────────────────

    fn human(name: &str) -> EntityWire {
        EntityWire { name: name.to_string(), kind: "HUMAN".to_string() }
    }

    fn machine(name: &str) -> EntityWire {
        EntityWire { name: name.to_string(), kind: "MACHINE".to_string() }
    }

    fn file_resource(name: &str) -> ResourceWire {
        ResourceWire {
            name: name.to_string(),
            rtype: "file".to_string(),
            scope: String::new(),
            is_public: false,
            ifc_label: String::new(),
        }
    }

    fn claim(holder: EntityWire, resource: ResourceWire, can_write: bool) -> ClaimWire {
        ClaimWire {
            holder,
            resource,
            can_read: true,
            can_write,
            can_delegate: can_write,
            confidence: 1.0,
            expires_at: None,
        }
    }

    // ── Property 1: FORBIDDEN flags always block ──────────────────────────────

    #[kani::proof]
    fn prop_forbidden_flags_always_block() {
        let increases_sovereignty: bool = kani::any();
        let resists_correction: bool = kani::any();
        let bypasses: bool = kani::any();

        // at least one flag is set
        kani::assume(increases_sovereignty || resists_correction || bypasses);

        let registry = OwnershipRegistryWire {
            claims: vec![],
            machine_owners: vec![MachineOwnerWire {
                machine: machine("bot"),
                owner: human("alice"),
            }],
        };

        let action = ActionWire {
            action_id: "forbidden_test".to_string(),
            actor: machine("bot"),
            description: String::new(),
            resources_read: vec![],
            resources_write: vec![],
            resources_delegate: vec![],
            governs_humans: vec![],
            argument: String::new(),
            increases_machine_sovereignty: increases_sovereignty,
            resists_human_correction: resists_correction,
            bypasses_verifier: bypasses,
            weakens_verifier: false,
            disables_corrigibility: false,
            machine_coalition_dominion: false,
            coerces: false,
            deceives: false,
            self_modification_weakens_verifier: false,
            machine_coalition_reduces_freedom: false,
        };

        let result = engine::verify(&registry, &action);
        assert!(!result.permitted, "FORBIDDEN flags must always block");
    }

    // ── Property 2: Ownerless machine is blocked ──────────────────────────────

    #[kani::proof]
    fn prop_ownerless_machine_blocked() {
        let registry = OwnershipRegistryWire {
            claims: vec![],
            machine_owners: vec![], // no owner registered
        };

        let action = ActionWire {
            action_id: "no_owner".to_string(),
            actor: machine("orphan"),
            description: String::new(),
            resources_read: vec![],
            resources_write: vec![],
            resources_delegate: vec![],
            governs_humans: vec![],
            argument: String::new(),
            increases_machine_sovereignty: false,
            resists_human_correction: false,
            bypasses_verifier: false,
            weakens_verifier: false,
            disables_corrigibility: false,
            machine_coalition_dominion: false,
            coerces: false,
            deceives: false,
            self_modification_weakens_verifier: false,
            machine_coalition_reduces_freedom: false,
        };

        let result = engine::verify(&registry, &action);
        assert!(!result.permitted, "Ownerless machine must be blocked (A4)");
        assert!(result.violations.iter().any(|v| v.contains("A4")));
    }

    // ── Property 3: A machine governing a human is always blocked ─────────────

    #[kani::proof]
    fn prop_machine_governs_human_blocked() {
        let registry = OwnershipRegistryWire {
            claims: vec![],
            machine_owners: vec![MachineOwnerWire {
                machine: machine("bot"),
                owner: human("alice"),
            }],
        };

        let action = ActionWire {
            action_id: "governs_test".to_string(),
            actor: machine("bot"),
            description: String::new(),
            resources_read: vec![],
            resources_write: vec![],
            resources_delegate: vec![],
            governs_humans: vec![human("bob")], // machine tries to govern human
            argument: String::new(),
            increases_machine_sovereignty: false,
            resists_human_correction: false,
            bypasses_verifier: false,
            weakens_verifier: false,
            disables_corrigibility: false,
            machine_coalition_dominion: false,
            coerces: false,
            deceives: false,
            self_modification_weakens_verifier: false,
            machine_coalition_reduces_freedom: false,
        };

        let result = engine::verify(&registry, &action);
        assert!(!result.permitted, "Machine governing human must be blocked (A6)");
        assert!(result.violations.iter().any(|v| v.contains("A6")));
    }

    // ── Property 4: Public resource read always permitted ────────────────────

    #[kani::proof]
    fn prop_public_resource_read_permitted() {
        let public_res = ResourceWire {
            name: "public_data".to_string(),
            rtype: "file".to_string(),
            scope: String::new(),
            is_public: true,
            ifc_label: String::new(),
        };

        let registry = OwnershipRegistryWire {
            claims: vec![], // no explicit claims needed
            machine_owners: vec![MachineOwnerWire {
                machine: machine("bot"),
                owner: human("alice"),
            }],
        };

        let action = ActionWire {
            action_id: "pub_read".to_string(),
            actor: machine("bot"),
            description: String::new(),
            resources_read: vec![public_res],
            resources_write: vec![],
            resources_delegate: vec![],
            governs_humans: vec![],
            argument: String::new(),
            increases_machine_sovereignty: false,
            resists_human_correction: false,
            bypasses_verifier: false,
            weakens_verifier: false,
            disables_corrigibility: false,
            machine_coalition_dominion: false,
            coerces: false,
            deceives: false,
            self_modification_weakens_verifier: false,
            machine_coalition_reduces_freedom: false,
        };

        let result = engine::verify(&registry, &action);
        assert!(result.permitted, "Public resource reads must always be permitted");
    }

    // ── Property 5: Non-escalation — write denied without write claim ─────────

    #[kani::proof]
    fn prop_write_denied_without_claim() {
        let res = file_resource("secret");
        let registry = OwnershipRegistryWire {
            claims: vec![claim(machine("bot"), file_resource("secret"), false)], // read-only
            machine_owners: vec![MachineOwnerWire {
                machine: machine("bot"),
                owner: human("alice"),
            }],
        };

        let action = ActionWire {
            action_id: "write_test".to_string(),
            actor: machine("bot"),
            description: String::new(),
            resources_read: vec![],
            resources_write: vec![res],
            resources_delegate: vec![],
            governs_humans: vec![],
            argument: String::new(),
            increases_machine_sovereignty: false,
            resists_human_correction: false,
            bypasses_verifier: false,
            weakens_verifier: false,
            disables_corrigibility: false,
            machine_coalition_dominion: false,
            coerces: false,
            deceives: false,
            self_modification_weakens_verifier: false,
            machine_coalition_reduces_freedom: false,
        };

        let result = engine::verify(&registry, &action);
        assert!(!result.permitted, "Write without write claim must be denied");
        assert!(result.violations.iter().any(|v| v.contains("WRITE DENIED")));
    }
}
