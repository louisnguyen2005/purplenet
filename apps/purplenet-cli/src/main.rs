use anyhow::Result;
use clap::{Parser, Subcommand};
use purplenet_core::{
    audit,
    domain::{Approval, Asset, MockEvent, NetworkScope, Run, Scenario},
    runner,
    store::Store,
};
use std::{path::PathBuf, str::FromStr};

#[derive(Parser)]
#[command(
    name = "purplenet",
    version,
    about = "Governed single-network validation (mock stage)"
)]
struct Cli {
    #[arg(long, global = true, default_value = "./state")]
    state: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create state and example files using reserved documentation addresses.
    Init,
    /// Evaluate a proposed mock exercise without creating a run.
    Validate {
        #[arg(long)]
        scope: PathBuf,
        #[arg(long)]
        approval: PathBuf,
        #[arg(long)]
        scenario: PathBuf,
        #[arg(long)]
        actor: String,
    },
    /// Run a policy-approved, data-only mock scenario.
    RunMock {
        #[arg(long)]
        scope: PathBuf,
        #[arg(long)]
        approval: PathBuf,
        #[arg(long)]
        scenario: PathBuf,
        #[arg(long)]
        actor: String,
    },
    /// Display a saved run by ID.
    ShowRun { id: String },
    /// Check the local hash-linked audit sequence.
    VerifyAudit,
}

fn main() {
    if let Err(error) = execute(Cli::parse()) {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Init => {
            let store = Store::init(&cli.state)?;
            let now = runner::now_unix();
            let scope = NetworkScope {
                schema_version: 1,
                id: "example-scope".into(),
                owner: "example-owner".into(),
                environment: "documentation-lab".into(),
                allowed_cidrs: vec![ipnet::IpNet::from_str("192.0.2.0/24")?],
                assets: vec![Asset {
                    id: "example-vm".into(),
                    ip: "192.0.2.10".parse()?,
                }],
                excluded_ips: vec![],
                valid_from_unix: now.saturating_sub(60),
                valid_until_unix: now + 86400,
                max_mock_events: 20,
                allowed_scenarios: vec!["example-observations".into()],
            };
            let scenario = Scenario {
                schema_version: 1,
                id: "example-observations".into(),
                description: "Synthetic events only; no network traffic or system modification"
                    .into(),
                target: "192.0.2.10".parse()?,
                target_asset_id: "example-vm".into(),
                mock_events: vec![
                    MockEvent {
                        id: "sample-file-event".into(),
                        label: "Synthetic file integrity signal".into(),
                    },
                    MockEvent {
                        id: "sample-network-event".into(),
                        label: "Synthetic network sensor signal".into(),
                    },
                ],
            };
            let approval = Approval {
                schema_version: 1,
                id: "example-approval".into(),
                scope_id: scope.id.clone(),
                scope_sha256: scope.digest()?,
                scenario_id: scenario.id.clone(),
                approved_by: "example-approver".into(),
                valid_from_unix: now.saturating_sub(60),
                valid_until_unix: now + 86400,
            };
            let dir = store.root.join("examples");
            store.write(&dir.join("scope.json"), &scope)?;
            store.write(&dir.join("approval.json"), &approval)?;
            store.write(&dir.join("scenario.json"), &scenario)?;
            println!("Initialized mock state at {}", store.root.display());
            println!(
                "Example files expire after one day; they are not real network authorization."
            );
        }
        Command::Validate {
            scope,
            approval,
            scenario,
            actor,
        } => {
            let scope: NetworkScope = Store::read(&scope)?;
            let approval: Approval = Store::read(&approval)?;
            let scenario: Scenario = Store::read(&scenario)?;
            runner::validate(&scope, &approval, &scenario, &actor)?;
            println!("Allowed for mock validation; no action was executed.");
        }
        Command::RunMock {
            scope,
            approval,
            scenario,
            actor,
        } => {
            let store = Store::open(&cli.state)?;
            let scope: NetworkScope = Store::read(&scope)?;
            let approval: Approval = Store::read(&approval)?;
            let scenario: Scenario = Store::read(&scenario)?;
            let run = runner::run_mock(&store, &scope, &approval, &scenario, &actor)?;
            println!(
                "Run {} completed with {} synthetic events",
                run.id,
                run.events.len()
            );
            println!(
                "Saved at {}",
                store
                    .root
                    .join("runs")
                    .join(format!("{}.json", run.id))
                    .display()
            );
        }
        Command::ShowRun { id } => {
            if !purplenet_core::domain::valid_id(&id) {
                anyhow::bail!("invalid run ID");
            }
            let store = Store::open(&cli.state)?;
            let path = store.root.join("runs").join(format!("{id}.json"));
            let run: Run = Store::read(&path)?;
            if run.id != id {
                anyhow::bail!("run ID mismatch");
            }
            println!("{}", serde_json::to_string_pretty(&run)?);
        }
        Command::VerifyAudit => {
            let store = Store::open(&cli.state)?;
            println!("Verified {} audit records", audit::verify(&store)?);
        }
    }
    Ok(())
}
