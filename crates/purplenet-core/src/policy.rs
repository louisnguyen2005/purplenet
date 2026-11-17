use crate::domain::{Approval, NetworkScope, Scenario};
use anyhow::{bail, Result};

pub fn authorize(
    scope: &NetworkScope,
    approval: &Approval,
    scenario: &Scenario,
    actor: &str,
    now: u64,
) -> Result<()> {
    scope.validate()?;
    approval.validate()?;
    scenario.validate()?;
    if actor.trim().is_empty() || actor == approval.approved_by {
        bail!("operator must be named and distinct from approver");
    }
    if scope.id != approval.scope_id || scope.digest()? != approval.scope_sha256 {
        bail!("approval is not bound to this exact scope");
    }
    if scenario.id != approval.scenario_id || !scope.allowed_scenarios.contains(&scenario.id) {
        bail!("scenario not approved by scope and approval");
    }
    if now < scope.valid_from_unix || now >= scope.valid_until_unix {
        bail!("scope window is not active");
    }
    if now < approval.valid_from_unix || now >= approval.valid_until_unix {
        bail!("approval window is not active");
    }
    if scenario.mock_events.len() > scope.max_mock_events as usize {
        bail!("scenario exceeds scope event limit");
    }
    if scope.excluded_ips.contains(&scenario.target) {
        bail!("target is excluded");
    }
    if !scope
        .allowed_cidrs
        .iter()
        .any(|cidr| cidr.contains(&scenario.target))
    {
        bail!("target is outside approved CIDRs");
    }
    if !scope
        .assets
        .iter()
        .any(|asset| asset.id == scenario.target_asset_id && asset.ip == scenario.target)
    {
        bail!("target is not an exact named asset in the scope");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Asset, MockEvent};

    fn fixture() -> (NetworkScope, Approval, Scenario) {
        let scope = NetworkScope {
            schema_version: 1,
            id: "lab".into(),
            owner: "owner".into(),
            environment: "lab".into(),
            allowed_cidrs: vec!["192.0.2.0/24".parse().unwrap()],
            assets: vec![Asset {
                id: "test-vm".into(),
                ip: "192.0.2.10".parse().unwrap(),
            }],
            excluded_ips: vec![],
            valid_from_unix: 100,
            valid_until_unix: 200,
            max_mock_events: 2,
            allowed_scenarios: vec!["demo".into()],
        };
        let scenario = Scenario {
            schema_version: 1,
            id: "demo".into(),
            description: "mock".into(),
            target: "192.0.2.10".parse().unwrap(),
            target_asset_id: "test-vm".into(),
            mock_events: vec![MockEvent {
                id: "one".into(),
                label: "synthetic".into(),
            }],
        };
        let approval = Approval {
            schema_version: 1,
            id: "a1".into(),
            scope_id: scope.id.clone(),
            scope_sha256: scope.digest().unwrap(),
            scenario_id: scenario.id.clone(),
            approved_by: "approver".into(),
            valid_from_unix: 100,
            valid_until_unix: 200,
        };
        (scope, approval, scenario)
    }

    #[test]
    fn allows_exact_approved_mock() {
        let (scope, approval, scenario) = fixture();
        assert!(authorize(&scope, &approval, &scenario, "operator", 150).is_ok());
    }

    #[test]
    fn refuses_out_of_scope_target_and_stale_digest() {
        let (scope, approval, mut scenario) = fixture();
        scenario.target = "198.51.100.10".parse().unwrap();
        assert!(authorize(&scope, &approval, &scenario, "operator", 150).is_err());
        let (scope, approval, mut scenario) = fixture();
        scenario.target = "192.0.2.11".parse().unwrap();
        assert!(authorize(&scope, &approval, &scenario, "operator", 150).is_err());
        let (mut scope, approval, scenario) = fixture();
        scope.owner = "changed-owner".into();
        assert!(authorize(&scope, &approval, &scenario, "operator", 150).is_err());
    }

    #[test]
    fn refuses_expired_and_self_approval() {
        let (scope, approval, scenario) = fixture();
        assert!(authorize(&scope, &approval, &scenario, "operator", 200).is_err());
        assert!(authorize(&scope, &approval, &scenario, "approver", 150).is_err());
    }
}
