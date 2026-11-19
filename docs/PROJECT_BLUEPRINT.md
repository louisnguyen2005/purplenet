# Build PurpleNet for Safe Network Validation

**PurpleNet should be a governed validation platform for one explicitly authorized network:** it records scope and approvals, coordinates safe and repeatable tests, correlates existing sensor evidence, and measures detection, response, cleanup, and recovery. It should not become a general-purpose intrusion tool, malware framework, or remote shell. The practical value is the workflow between mature emulation tools and mature sensors: an operator can answer which approved scenario ran, which assets were in scope, what each sensor observed, where visibility was missing, and whether the network returned to a healthy state. Use NIST CSF 2.0 to organize outcomes, NIST SP 800-61 Rev. 3 for incident response and recovery, and NIST SSDF for PurpleNet's own development; these are guidance frameworks, not automatic certifications or universal legal requirements. ([NIST CSF 2.0](https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=957258), [NIST SP 800-61 Rev. 3](https://csrc.nist.gov/pubs/sp/800/61/r3/final), [NIST SSDF](https://csrc.nist.gov/pubs/sp/800/218/final))

**Blueprint date: 2026-11-17.** Current framework versions, package advisories, laws, and connector APIs must be checked again before each implementation or production decision. “Must” below means a PurpleNet product requirement unless the text explicitly attributes it to a standard or law; the cited NIST/CISA frameworks are generally voluntary guidance unless adopted by a contract, regulation, or organization policy.

## 1. Product scope and measurable needs

### Purpose and intended users

PurpleNet serves a network owner who wants to validate defensive visibility without assembling a manual chain of spreadsheets, scenario tools, log exports, and cleanup notes. Its users are the network owner, an exercise planner, an operator, a detection engineer, an approver, an auditor, and a recovery lead. A small deployment can combine roles, but the system should still record who approved and who ran each exercise.

A normal exercise begins with a signed or reviewed authorization record, an explicit network scope, a selected scenario pack and test window. PurpleNet confirms that sensors are healthy and that the selected assets are in scope, records pre-run health, then launches either a dry-run/mock exercise or a separately approved safe validation. It correlates events from existing tools and produces an evidence-backed report. It then schedules cleanup, checks service and sensor health, and requires a human to close or escalate the run. CALDERA already provides adversary-emulation operations and a Mock plugin for simulated agents; Atomic Red Team already organizes tests around ATT&CK and documents prerequisites, timeouts and cleanup. PurpleNet should orchestrate governance, evidence, comparison and recovery around those systems rather than reimplement their catalogs or execute arbitrary commands. ([CALDERA plugin library](https://caldera.readthedocs.io/en/latest/Plugin-library.html), [CALDERA operation results](https://caldera.readthedocs.io/en/latest/Operation-Results.html), [Atomic Red Team local tests](https://www.atomicredteam.io/docs/invoke-atomicredteam/execute-tests-locally))

### What the first release will and will not do

The first release is for **one network and one administration domain**, initially a disposable Linux lab with a small number of virtual machines and one network sensor. It will import or query asset and telemetry data, manage scope and approvals, run mock or non-destructive test plans, correlate evidence, score coverage, and verify cleanup. It will not exploit vulnerabilities, bypass endpoint protection, hide activity, steal credentials, perform lateral movement, accept arbitrary shell commands, or provide an unrestricted remote-control channel. Production use should begin read-only and add any state-changing test only through an independently reviewed integration with explicit operator approval.

The product should answer five questions for every run: Was authorization valid? Did every action stay inside the approved scope and window? Which expected events did each sensor observe? Did the exercise affect service health? Did cleanup and recovery succeed? These questions map naturally to NIST CSF 2.0’s Govern, Identify, Protect, Detect, Respond and Recover outcomes; mapping to CSF is a planning aid, not a claim of certification. ([NIST CSF 2.0](https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=957258))

### Requirements

| Area | PurpleNet requirement | Evidence of completion |
|---|---|---|
| Scope | Every run binds to an owner, exact network/asset scope, exclusions, approval reference, start/end time, scenario revision and rate/resource limits. Expired or ambiguous scope fails closed. | Negative cases prove out-of-scope, expired and malformed scopes are rejected before any connector is invoked. |
| Identity | Authenticated users have least-privilege roles: planner, approver, operator, auditor and administrator. MFA/OIDC is required before multi-user or production-like deployments. | Role matrix and access audit show each role can perform only its assigned actions. CISA recommends MFA for administrators and strong audit logging; role separation is PurpleNet’s design decision. ([CISA Secure by Design](https://www.cisa.gov/resources-tools/resources/secure-by-design)) |
| Scenario | Scenario definitions are versioned data with declared risk, target types, prerequisites, timeout, expected telemetry, cleanup and rollback. No user-supplied shell or arbitrary code is part of the scenario format. | Schema validation, static review, signatures or reviewed checksums, and a scenario manifest show what can execute. |
| Orchestration | Runs have durable states, idempotent stop/cleanup, global emergency stop, bounded queues and controller-lease expiry. Restarting the controller never resumes an operation silently. | Audit trail proves state transitions and safe interruption through simulated restart and connector loss. |
| Telemetry | Connectors preserve original source IDs and timestamps, normalize a common event envelope, report gaps and parser errors, and avoid treating missing logs as “no detection.” | Replaying known fixtures yields expected correlation; sensor outages appear as degraded/unknown coverage. |
| Evidence | Every result is linked to run ID, approval, scope revision, scenario version, connector version and framework mapping version. Exported evidence includes hashes and access history. | A report can be regenerated from stored event IDs and records its provenance. |
| Privacy | Collect the minimum event fields needed; default to metadata rather than packet payloads; redact secrets; define retention, access and export controls before production deployment. | Data inventory, retention configuration, redaction review and deletion/restore rehearsal. NIST privacy guidance treats aggregated logs as a privacy risk and recommends minimization, retention limits and access controls. ([NIST privacy risk assessment](https://www.nist.gov/itl/applied-cybersecurity/privacy-engineering/collaboration-space/privacy-risk-assessment)) |
| Reliability | Runs stop safely on scope change, loss of approval, sensor health failure, unexpected service degradation, quota breach or a real incident signal. Recovery is a separate state and cannot be auto-closed. | Fault-injection results and recovery record demonstrate stop, cleanup, service validation and escalation. |
| AI | Any AI feature is advisory and read-only. It cannot modify scope, approve an exercise, invoke a connector or launch a scenario. | Permission tests show no model response can bypass the deterministic policy engine. |
| Supply chain | Dependencies, builds and releases are reviewable; a release has an SBOM, signature/hash and provenance appropriate to the available build environment. | Release artifact, lockfile, SBOM, provenance and vulnerability disposition record. ([NIST SSDF](https://csrc.nist.gov/pubs/sp/800/218/final), [SLSA 1.2](https://slsa.dev/spec/v1.2/)) |

### Standards and framework mapping

Use the frameworks as separate lenses. NIST CSF 2.0 organizes outcomes; NIST SP 800-61 Rev. 3 informs preparation, detection, response and recovery; NIST SSDF informs how PurpleNet itself is designed and released; NIST AI RMF and its Generative AI Profile inform optional AI features; MITRE ATT&CK and ATLAS supply versioned threat and detection references. None of these alone certifies PurpleNet or replaces legal, sectoral, contractual or privacy review. ([NIST CSF 2.0](https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=957258), [NIST SP 800-61 Rev. 3](https://csrc.nist.gov/pubs/sp/800/61/r3/final), [NIST AI RMF](https://www.nist.gov/itl/ai-risk-management-framework), [NIST GenAI Profile](https://www.nist.gov/publications/artificial-intelligence-risk-management-framework-generative-artificial-intelligence), [MITRE ATLAS](https://atlas.mitre.org/))

As of this blueprint date, MITRE ATT&CK v20.1 is listed as current. ATT&CK’s legacy Data Sources catalog was deprecated in v18; new mappings should use Detection Strategies and Analytics, while retaining old mappings only where needed for compatibility. Pin framework versions on every scenario and report so a future update cannot silently rewrite historical results. ([ATT&CK versions](https://attack.mitre.org/resources/versions/), [Detection Strategies](https://attack.mitre.org/detectionstrategies/), [Analytics](https://attack.mitre.org/analytics/), [Data Sources notice](https://attack.mitre.org/datasources/))

## 2. Architecture and implementation plan

### Recommended architecture

Start as a modular monolith, not a fleet of microservices. A single Rust control-plane service with a local web UI, PostgreSQL for transactional records and a separate encrypted evidence store is easier to operate and audit. Keep connectors in separate processes or restricted tasks with scoped credentials, so a malformed event or compromised connector cannot directly approve or schedule a test.

```text
                    ┌─────────────────────────┐
                    │ Web UI / CLI / API      │
                    └────────────┬────────────┘
                                 │ authenticated requests
                    ┌────────────▼────────────┐
                    │ Policy + Scope Engine   │
                    │ approvals / run state   │
                    └───────┬──────────┬──────┘
                            │          │
                    ┌───────▼───┐  ┌───▼────────────┐
                    │ Scenario  │  │ Audit + Evidence│
                    │ Scheduler │  │ Store / Reports │
                    └─────┬─────┘  └───▲────────────┘
                          │ approved     │ normalized events
                 ┌────────▼────────┐    │
                 │ Safe test adapter│    │
                 │ mock / approved  │    │
                 └─────────────────┘    │
                                  ┌─────┴─────────────────┐
                                  │ Connector processes   │
                                  │ Zeek / Suricata /      │
                                  │ osquery / OTLP / SIEM  │
                                  └───────────────────────┘
```

Suggested Rust boundaries are `scope` for allowlists and approval validation; `identity` for OIDC/MFA and roles; `scenario` for versioned metadata; `scheduler` for state transitions and quotas; `connectors` for ingestion and mature tool adapters; `normalize` for event envelopes; `correlate` for expected-versus-observed matching; `evidence` for retention and integrity; `coverage` for framework mapping and metrics; `report` for exports; and `audit` for append-only operator records. Use traits around connectors and storage backends, but avoid abstracting every function before the first integration works.

A practical first stack is Tokio, Axum, Serde, SQLx and PostgreSQL, with a file-backed encrypted evidence directory for a single-node lab. The exact crate versions must be checked at implementation time for maintenance, security advisories, license and compiler compatibility; the research pass did not benchmark libraries. The API should bind to localhost in development and require TLS or a trusted reverse proxy plus authenticated identity in a multi-user deployment. Do not place secrets or raw packet data in ordinary application logs.

### Data model and event flow

Version the following records from the first commit: `NetworkScope` (owner, allowed assets/CIDRs, exclusions, environment, change window, expiry, rate/volume limits); `Approval` (approvers, authorization reference, scenario categories, signed/recorded time); `Scenario` (ID, semantic version, risk tier, allowed adapter, prerequisites, expected observations, cleanup and rollback metadata, framework mappings); `Run` (run ID, scope and scenario revisions, actor, approval, state, timestamps, stop reason); `Connector` (source identity, version, health, credential reference); `Event` (source timestamp, ingestion timestamp, host/network identity, event type, fields, provenance and redaction flags); `Observation` (expected item, matched events, confidence and analyst disposition); `EvidenceBundle` (hash, storage reference, retention deadline, access record); and `Incident` (trigger, containment, recovery, owner and closure approval).

A run should move through `Draft → Approved → Ready → Running → StopRequested → Stopped → Cleanup → Verified → Closed`, with `Failed` and `RecoveryRequired` as durable exceptional states. State transitions must be transactional and audited. Before `Running`, validate current scope, approval expiry, connector health, storage capacity and any required recovery point. During execution, enforce timeout and quotas and write a heartbeat. When the run ends, correlate observations, run cleanup, independently validate health, then require a human to close it. If the controller restarts, recover the stored state as stopped or recovery-required; never silently continue.

Connector events must retain both source time and ingestion time. Preserve source IDs and parser versions, attach confidence and data quality, and distinguish “not observed” from “sensor unavailable,” “parsing failed,” “outside retention,” or “mapping absent.” OpenTelemetry provides shared patterns for traces, metrics and logs; Zeek and Suricata can provide network security events; osquery can provide endpoint telemetry. PurpleNet should ingest from them instead of becoming another packet sensor or endpoint-query engine. ([OpenTelemetry](https://opentelemetry.io/docs/), [Zeek documentation](https://docs.zeek.org/en/current/), [Suricata documentation](https://docs.suricata.io/en/latest/), [osquery documentation](https://osquery.io/))

### Integration boundaries and AI features

For the first exercise, use synthetic fixtures or CALDERA Mock mode. For later approved validation, PurpleNet should call mature tools only through explicit adapters and only after scope and human approval checks; it should not interpret imported ATT&CK or STIX/TAXII content as executable instructions. Atomic Red Team’s prerequisite, timeout and cleanup metadata is a useful format to study, but imported tests still require review and policy checks. STIX/TAXII is suitable for threat-intelligence exchange; it should not become the scenario execution language. ([CALDERA Mock plugin](https://caldera.readthedocs.io/en/latest/Plugin-library.html), [Atomic Red Team continuous testing](https://www.atomicredteam.io/docs/invoke-atomicredteam/continuous-atomic-testing), [OASIS CTI documentation](https://oasis-open.github.io/cti-documentation/))

AI can later summarize evidence, group duplicate alerts, suggest missing telemetry, or draft a human-readable report. Treat every imported log, ticket, alert, threat-intelligence object and document as untrusted input because it can contain prompt injection or poisoned content. Keep model output in a suggestion field; pass any proposal through the same schema validation and deterministic authorization engine as user input. The model must not set targets, modify scope, authorize a run, invoke shell or connector tools, or retrieve secrets. Record model/provider/version, prompt-template version, retrieved evidence IDs and analyst approval. Provide a kill switch and a deterministic no-AI workflow. NIST AI RMF is lifecycle guidance and OWASP’s 2026 LLM/agent guidance identifies risks such as goal hijack, tool misuse, identity/privilege abuse and context poisoning; neither replaces product-specific threat modeling. ([NIST AI RMF](https://www.nist.gov/itl/ai-risk-management-framework), [NIST GenAI Profile](https://www.nist.gov/publications/artificial-intelligence-risk-management-framework-generative-artificial-intelligence), [OWASP 2026 LLM Top 10](https://genai.owasp.org/2026/09/01/owasp-genai-security-project-unveils-2026-top-10-for-llm-applications-new-agent-control-standard-and-sponsors-as-community-tops-30000-members/), [OWASP Agentic Top 10](https://genai.owasp.org/2025/12/09/owasp-top-10-for-agentic-applications-the-benchmark-for-agentic-security-in-the-age-of-autonomous-ai/))

### Suggested repository structure

```text
purplenet/
  apps/
    purplenetd/              # API, scheduler, lifecycle
    purplenet-cli/           # admin and operator CLI
    purplenet-ui/            # optional web UI after CLI/API stabilize
  crates/
    domain/                  # scope, approval, run and event types
    policy/                  # deterministic authorization
    scenarios/               # schema, validation, signed pack metadata
    connectors/              # source adapters and health checks
    normalize/               # canonical event envelope
    correlate/               # expected vs observed matching
    evidence/                # storage, hashing, retention
    reporting/               # JSON/Markdown exports
    audit/                   # append-only action records
  scenario-packs/            # data-only, reviewed, version-pinned packs
  migrations/                # database schema changes
  deploy/                    # local lab compose and sample VM topology
  docs/
    architecture.md
    threat-model.md
    operations.md
    incident-playbooks.md
    privacy.md
    update-policy.md
  .github/workflows/         # CI and signed release pipeline
```

## 3. Network governance and phased build

### Lab and scope model

Start in a disposable virtual network with one controller, two or three test guests, a sensor and a deliberately small sample service. Disable forwarding to other LANs and the Internet by default, avoid shared folders and real credentials, restrict management access to the host-only/admin path, and snapshot guests before each run. A libvirt isolated network can keep guest traffic away from other LAN machines when forwarding is omitted, but the host remains connected to guests; QEMU also documents VM escape as a host threat. Isolation reduces risk but does not make the hypervisor immune to escape. ([libvirt isolated network configuration](https://libvirt.org/formatnetwork.html), [QEMU security documentation](https://www.qemu.org/docs/master/system/security.html))

Represent scope in a signed or reviewed manifest. Include environment ID, owner, permitted assets or exact ranges, exclusions, approved scenario IDs, test window with timezone, maximum event/request rates, storage ceiling, emergency contact and expiry. Revalidate the manifest at preparation and immediately before each state-changing adapter call. Any route, ownership, maintenance-window or asset change invalidates the authorization until a human approves the new scope. The controller should make the “out-of-scope” state impossible to override with a scenario field or AI suggestion.

### Phased implementation

| Phase | Build | Deliverable and exit condition |
|---|---|---|
| 0. Charter and threat model | Document one network, asset owners, scenarios allowed, data fields, trust boundaries, failure triggers and recovery path. Choose a hypervisor and a demo sensor. | Owner-approved scope, architecture diagram, initial risk register and incident contact. No live action yet. |
| 1. Domain and policy core | Rust workspace, versioned schemas, scope parser, role model, run state machine, audit event model. | Unit-level policy design proves malformed, expired and out-of-scope requests fail closed. |
| 2. Mock end-to-end workflow | Scenario catalog, synthetic events or CALDERA Mock adapter, run lifecycle, event normalization and report. | One mock exercise creates a complete evidence bundle and is reproducible from a clean lab snapshot. |
| 3. First sensor connector | Read-only import from one source, initially Zeek, Suricata, osquery or an OTLP receiver. Add connector health and gap reporting. | Fixtures correlate to expected observations; connector outage appears as “unknown/degraded,” never as a clean result. |
| 4. Scope and approvals | UI/CLI approval workflow, expiry, operator/approver separation, test window, per-run cap, stop switch and signed scenario pack metadata. | Negative scope and revocation scenarios prevent execution and create audit records. |
| 5. Cleanup and recovery | Idempotent cleanup, pre/post service health checks, recovery-required state, snapshot/runbook integration and evidence preservation. | Interrupted run can be stopped, cleaned or restored, independently verified and closed by an authorized person. |
| 6. Coverage and privacy | ATT&CK v20.1 mappings, detection strategy/analytic links, false-positive control run, redaction and retention. | Report distinguishes detection, missing telemetry, parser failure, out-of-window data and unsupported mapping. |
| 7. Hardening and release | SSDF controls, CI, SBOM, dependency review, signed artifacts, backup/restore rehearsal, operator docs and support policy. | Release gate met; a reviewer can reproduce the demo and restore its state from documented materials. |
| 8. Optional AI assistant | Read-only report summarization and evidence search with provenance, output validation and human approval. | AI can never change scope or run actions; deterministic non-AI flow remains functional. |

Phases are ordered by dependency: policy and audit precede adapters; mock integration precedes live validation; recovery precedes production-like use; AI comes last. This sequencing is a design recommendation, not a prescribed NIST implementation order.

### API and scenario contract

Expose a small API: `POST /scopes` and `GET /scopes/{id}` for reviewed scope records; `POST /scenarios/{id}/validate`; `POST /runs` to request a run; `POST /runs/{id}/approve`; `POST /runs/{id}/stop`; `GET /runs/{id}`; `GET /runs/{id}/events`; and `POST /connectors/{id}/ingest` for authenticated event ingestion. Every write includes an idempotency key and actor identity. Reject duplicate or stale requests safely. Keep the adapter protocol typed and narrow: `prepare`, `start_approved_test`, `collect_status`, `stop`, `cleanup`, and `health`; never expose a generic “execute command” API.

A scenario pack should be data, not arbitrary executable code. Each scenario declares an ID and version, purpose, risk tier, supported assets, prerequisites, expected observation IDs, time/resource limits, stop conditions, cleanup expectations, framework mapping with version, reviewer and signature/checksum. An adapter maps only approved scenario types to mature test tools or synthetic fixtures. Store the exact pack digest in the run so results remain explainable after scenario updates.

## 4. Threat model, incidents, privacy, and reliability

### Trust boundaries and AI-era threats

Protect against a compromised operator account, malicious or malformed scenario pack, compromised connector, event spoofing, controller compromise, database/evidence tampering, dependency/build compromise, scope drift, prompt injection in imported data, overbroad AI agency, collector outage and VM/hypervisor compromise. Assets include the scope manifest, credentials, controller state, network services, evidence, user identity, connector secrets, and availability of the lab.

The most important control is not a warning banner; it is deterministic enforcement at the API, scheduler and adapter boundaries. The policy engine checks the actor, role, approval, scope, window, asset identity, scenario risk and quotas on every state-changing operation. A connector receives only the credential and network reachability needed to ingest its own source. A scenario pack is signed/versioned, and a stale or unknown signature blocks execution. Use short-lived credentials where supported, and revoke them on stop or incident. Store control-plane credentials separately from evidence encryption keys.

| Threat or failure | Early signal | Prevention | Response and recovery |
|---|---|---|---|
| Scope escape or stale authorization | Target not in manifest, approval expired, routing or owner changed | Fail-closed scope checks at prepare and dispatch; explicit exclusions; short approval window | Global stop; invalidate run; preserve audit; verify no action reached out-of-scope asset; require new approval. Treat as release-blocking defect. |
| Unsafe or altered scenario pack | Digest/signature mismatch, unknown action type, changed risk metadata | Typed data-only schema, review/signing, allowlisted adapters, version pinning | Reject pack; quarantine artifact; inspect prior runs using same digest; restore known-good pack. |
| Connector compromised or noisy | Authentication anomaly, schema drift, unusual event volume, impossible timestamps | Separate process/identity, read-only credential, ingestion quotas, schema validation, source mTLS where available | Disable connector credential; mark its coverage unknown; preserve raw metadata; rotate secrets; re-ingest from trusted source after review. |
| Operator/controller account takeover | New device/location, MFA anomaly, unexpected approvals | MFA/OIDC, least privilege, separation of duties, short sessions, audit alerts | Revoke sessions/tokens; pause active runs; review approvals and exports; restore roles from trusted config. |
| Evidence or personal-data leak | Unexpected sensitive fields, export spike, retention violation | Data minimization, redaction, encryption, RBAC, retention/deletion policy | Stop export; revoke access; preserve access records; notify privacy owner; follow jurisdictional breach process; rotate exposed secrets. |
| Dependency or release compromise | Advisory, signature/provenance mismatch, unexpected build input | Lockfile, review, SBOM, signed artifacts, provenance, protected CI | Block rollout; revoke signing credentials if implicated; roll back to known-good signed release; assess affected runs and evidence. |
| Lost telemetry / parser failure | Heartbeat gap, queue overflow, timestamp skew, parse errors | Health metrics, buffering, schema versions, clear “unknown” state | Pause tests if detection evidence is essential; restore connector; mark interval incomplete; re-run only with approval. |
| Service impact or resource overload | Health check fail, latency/error budget breach, queue/storage high-water mark | Per-run caps, timeout, small canary, test windows, abort thresholds | Stop scenario; use owner-approved isolation; recover service from known-good backup/snapshot; validate independently. |
| AI prompt injection or unsafe suggestion | Imported text asks model to change scope or launch action | AI read-only; untrusted-data labeling; no action tools; output schema validation | Disable assistant; retain model/evidence IDs; review affected suggestions; continue deterministic workflow. |
| Host/hypervisor compromise | Guest isolation alert, host health anomaly | Separate lab host/network, patched hypervisor, minimal shared devices, no secrets in guests | Stop all exercises; disconnect lab segment if safe; preserve host and guest evidence; rebuild from trusted images. |

NIST SP 800-61 Rev. 3 integrates incident response into ongoing risk management and emphasizes preparation, detection, response and recovery. The table’s specific thresholds and procedures are PurpleNet design choices that the network owner must tailor. ([NIST SP 800-61 Rev. 3](https://csrc.nist.gov/pubs/sp/800/61/r3/final), [NIST SP 800-34 Rev. 1](https://csrc.nist.gov/pubs/sp/800/34/r1/upd1/final))

### Incident playbooks

**Scope violation:** stop the scheduler and issue the global stop signal; mark the run `RecoveryRequired`; revoke run-scoped connector credentials; preserve scope, approval, dispatch and connector logs; determine which assets were contacted; notify the owner; validate that no out-of-scope state changed; and do not re-run until the manifest and policy defect are reviewed. A scope violation blocks release until the cause is fixed and a negative regression case demonstrates rejection.

**Compromised connector:** disable its identity and ingestion endpoint; quarantine queued payloads; keep run coverage marked unknown; preserve source IDs, parser versions and relevant access logs; rotate credentials; compare connector package digest with the signed release; then redeploy from a known-good artifact and reprocess only after review. Do not let connector compromise authorize scheduler operations.

**Bad update or release:** stop rollout, block new runs, preserve the installed version and provenance, revoke signing/build credentials if suspected, roll back to a prior signed release, validate database migrations are reversible or recoverable, and check whether the issue changed scope or evidence. Publish an advisory and fixed release according to impact; preserve old run records with the version that produced them.

**Data exposure:** restrict export and revoke sessions, identify records and people/assets affected, preserve access audit, contact the privacy/security owner, follow the applicable notification process, rotate any exposed credentials, and verify deletion or retention actions. Do not silently delete evidence required for an incident investigation; document the legal/contractual decision.

**Telemetry loss:** mark affected sensors and time intervals unhealthy; pause runs whose evaluation depends on that source; do not label missing evidence as a successful detection or a clean result; restore connector connectivity and schema compatibility; reconcile buffered events; then repeat only under a renewed or still-valid approval.

**Overload or service degradation:** stop new work, ask adapters to stop, disable scheduling, notify the service owner, preserve health and queue metrics, and use only the owner-approved containment/restore action. Validate core service health and sensor health independently before closing the run; document whether the load came from PurpleNet, a sensor, the tested application or unrelated activity.

### Privacy and evidence lifecycle

Define an event dictionary before connecting production telemetry. For each field record purpose, source, sensitivity, whether it can identify a person, retention period, export permission and redaction rule. Default to metadata and normalized alerts; raw packet payloads, credentials, message contents and unrelated user activity stay off. Encrypt evidence at rest and in transit, enforce role-based access, record every read/export/delete, and test retention deletion plus backup restoration. Jurisdiction, sector, employment, contract and data-residency rules can impose additional duties; this blueprint is not a legal determination. NIST privacy material supports minimizing collection, limiting retention and restricting access for systems that aggregate security logs. ([NIST privacy risk assessment](https://www.nist.gov/itl/applied-cybersecurity/privacy-engineering/collaboration-space/privacy-risk-assessment), [NIST SP 1800-28](https://www.nccoe.nist.gov/publication/1800-28/VolB/index.html))

Store original event IDs and hashes where practical, but avoid claiming forensic admissibility without a validated chain-of-custody process and legal review. Keep a reproducibility bundle with scenario digest, scope digest, connector versions, framework mapping versions, time sync health and report generator version. Set retention through configuration with a documented owner and deletion job; a starting design can be 30 days for raw lab telemetry and 180 days for metadata/evidence, but those periods are examples, not source-mandated defaults. Production retention must be selected after privacy and operational review.

## 5. Maintenance, metrics, application, and definition of done

### Update and maintenance cadence

No cited NIST or CISA source sets a universal update schedule for PurpleNet. The cadence below is a recommended operating policy; adjust it based on risk, exposure, staffing and support commitments. MITRE ATT&CK publishes major updates twice a year, with minor corrections possible; v20.1 is current on the blueprint date. ([ATT&CK FAQ](https://attack.mitre.org/resources/faq/), [ATT&CK versions](https://attack.mitre.org/resources/versions/))

| Item | Proposed cadence | Immediate trigger |
|---|---|---|
| Dependency advisories | Automated daily check, human review weekly | Critical/high issue in reachable dependency: triage the same business day; publish an out-of-band fix based on exposure. |
| ATT&CK/ATLAS mappings | Review monthly and after framework release | Version mapping content; migration is reviewed and never rewrites historical reports. |
| Connectors | Monthly compatibility review; test against upstream stable release | Security fix, schema-breaking change or repeated ingestion gaps. |
| Scope and approvals | Validate on every run; approval expires at end of window | Asset owner, route, network boundary or scenario changes. |
| Main application | Planned monthly release; emergency releases as needed | Scope bypass, unsafe state transition, credential exposure or critical dependency issue. |
| Backup and restore | Monitor backup daily; restore rehearsal quarterly in production-like use | Failed restore, evidence-integrity problem or major storage change. |
| Roles, credentials and retention | Review quarterly and on personnel/vendor change | Suspected token loss, role change, contract end or legal retention update. |
| AI model and prompt configuration | Review before each model/provider change and quarterly | Data handling change, new tool permission, prompt-injection incident or model behavior regression. |

Use `Cargo.lock`, automated advisory alerts and dependency review; do not auto-merge every update without compatibility review. Generate an SBOM, publish artifact hashes and sign releases. Add provenance and only claim the SLSA level actually achieved by the build system. NIST SSDF v1.1 is final; NIST lists v1.2 as a draft in the research notes, so verify publication status before treating it as adopted. SLSA 1.2 defines build provenance and verification concepts. ([NIST SSDF publications](https://csrc.nist.gov/Projects/ssdf/publications), [NIST SSDF 1.1](https://csrc.nist.gov/pubs/sp/800/218/final), [SLSA 1.2](https://slsa.dev/spec/v1.2/))

### Operational metrics and service targets

Treat these as proposed targets, not standards mandates. For a single-node lab, begin with simple indicators rather than enterprise-scale availability promises: **100% of runs have a valid scope and approval record; 0 actions execute after stop acknowledgement; 100% of out-of-scope requests are rejected; 100% of sensor gaps are represented as degraded/unknown; 100% of evidence exports are audited; 100% of cleanup attempts end as verified or recovery-required.** Define stop latency per adapter before connecting it; use a conservative target such as under 10 seconds in a lab and tighten only after measurement. For the management plane, a starting target could be 99% monthly availability during scheduled exercise windows, excluding planned maintenance, but a student project should report measured uptime rather than promise production reliability.

Measure detection quality only on labeled scenarios and separate control runs: coverage by expected observation, time-to-detection, false-positive rate on controls, telemetry completeness, parser failure rate, cleanup completeness, service health before/after, restore time, run reproducibility and operator approval latency. Report denominators and uncertainty; a small fixture set does not demonstrate real-world detection performance. Each report should distinguish `detected`, `not detected`, `not observable`, `connector unavailable`, `parser error`, `out of retention`, and `not evaluated`.

### Recommended validation gates

Before live use, require static/unit checks for the policy engine and state machine, schema and parser checks with malformed inputs, integration runs with mock adapters, fault injection for controller restart and connector loss, and destructive-action deny tests proving no arbitrary command route exists. Test scope expiry, ambiguous identity, out-of-scope target, stop switch, cleanup idempotency, backup restoration, redaction, role boundaries, AI prompt injection in imported evidence and release signature validation. These are recommended engineering gates, not claims that a particular framework mandates these exact tests.

A test result is not release evidence by itself. Require review of expected-versus-observed telemetry, safety limits, logs, recovery artifacts, dependency changes and release provenance. Any open critical/high defect in scope enforcement, authentication, arbitrary code execution, evidence confidentiality, stop behavior or recovery blocks a live test release. Use a disposable environment for failure and recovery exercises.

### Where PurpleNet is useful and how it can be distinct

PurpleNet applies to a university cyber range, a small company’s internal validation network, a product team’s staging network, or a lab that needs repeatable evidence for a specific sensor stack. It is especially useful when the operator needs to show that a planned exercise ran within a written boundary, that a sensor was available, and that cleanup occurred. It does not replace a SIEM, EDR, vulnerability scanner, packet analyzer, CALDERA, Atomic Red Team, or a professional assessment team.

The defensible contribution is a **single-network governance and measurement layer** that connects approvals, scope, scenario lifecycle, multi-sensor evidence, detection coverage, cleanup and recovery in one reproducible run. CALDERA and Atomic Red Team already cover adversary emulation; Zeek and Suricata already analyze network activity; osquery already provides endpoint telemetry. Novelty therefore lies in a demonstrably useful integration workflow and its safety properties, not in claiming the category is new. ([MITRE CALDERA](https://www.mitre.org/our-impact/intellectual-property/caldera), [Atomic Red Team](https://www.atomicredteam.io/docs), [Zeek](https://docs.zeek.org/en/current/), [Suricata](https://docs.suricata.io/en/latest/), [osquery](https://osquery.io/))

### Definition of done

PurpleNet v1 is done when an authorized operator can create a versioned scope, obtain a separately recorded approval, run a mock scenario against a disposable single-network lab, ingest at least one independent sensor stream, correlate expected observations with actual evidence, show sensor gaps honestly, stop and clean up safely, verify service health, export a reproducible report, and restore from a documented recovery point. Every action is tied to the actor, approval, scope digest, scenario digest, connector version and timestamps. A reviewer can reproduce the demo from documented files and confirm that an out-of-scope or expired request is rejected before execution.

**Conclusion.** The project’s hardest engineering problem is governance across boundaries: the network scope, scenario pack, connector identity, operator role, telemetry source and recovery process must all agree before PurpleNet reports a successful exercise. AI-era features increase the importance of this rule because imported content can be hostile and AI agents can overreach; the model should help explain evidence, while deterministic policy remains in control. Build the mock workflow and recovery path first, then add a single real telemetry connector, then multi-user approvals, and add live validation or AI only after the safety and evidence gates hold.
