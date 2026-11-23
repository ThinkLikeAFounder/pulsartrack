# Security Policy

## Supported Versions

PulsarTrack is currently in active pre-1.0 development (`v0.1.0`). We support and provide security updates for the current active development target (`main` branch / latest `0.x` release):

| Component | Supported Versions |
|-----------|--------------------|
| Smart Contracts (Soroban) | `0.x` (latest on `main`) |
| Frontend Web Application | `0.x` (latest on `main`) |
| Backend Services / APIs | `0.x` (latest on `main`) |

Older versions or stale development branches are not supported with backported security updates. This policy will be revisited and updated once formal versioned releases reach `v1.0.0`.


## Reporting a Vulnerability

**Please do NOT report security vulnerabilities through public GitHub issues.**

If you believe you have found a security vulnerability in PulsarTrack, please report it to us privately to ensure it can be addressed before being publicly disclosed. 

Please send an email to: **security@drips.network** (or open a confidential security advisory on GitHub).

In your report, please include:
- A description of the vulnerability and its potential impact.
- Steps to reproduce the issue (proof-of-concept scripts or detailed instructions).
- The specific files or endpoints affected.
- Any potential mitigation or workaround if you have one.

## Response Timeline

We take all security reports seriously and commit to the following timelines:
- **Acknowledgment:** Within 48 hours of your report.
- **Triage & Initial Assessment:** Within 7 days of acknowledgment.
- **Resolution & Patch:** We aim to release a patch or mitigation for critical vulnerabilities within 14 days, though complex issues affecting smart contracts may take longer due to required testing and auditing.

## Scope

### In Scope
The following components are fully in scope for security reports:
- **Soroban Smart Contracts:** Any contract deployed under the PulsarTrack project (e.g., publisher-reputation, subscription-manager, campaign-orchestrator).
- **Backend API:** All backend services and endpoints maintained by PulsarTrack.
- **Frontend App:** Client-side validation, wallet interactions, and general web vulnerabilities (XSS, CSRF, etc.) within the main web application.

### Out of Scope
The following are explicitly **out of scope**:
- Theoretical vulnerabilities without a reproducible proof-of-concept.
- Vulnerabilities relying on social engineering, phishing, or physical access.
- Bugs in third-party dependencies (unless the vulnerability is introduced by our specific misuse of the library).
- Issues related to the Stellar network itself or the Soroban execution environment (please report these directly to the Stellar Development Foundation).
- Denial of Service (DoS) attacks requiring massive external resources.

## Accepted Dependency Advisories

`npm audit fix` has been applied to the workspace lockfile. The advisory below
remains because no non-breaking fix exists. It is accepted until the listed
follow-up lands. Recheck with `npm run audit` in `backend/` and
`npm audit --audit-level=high` in `frontend/`.

| Package | Severity | Path | Why accepted | Follow-up |
|---|---|---|---|---|
| `crypto-bigint` <0.7.6 ([RUSTSEC-2026-0387](https://rustsec.org/advisories/RUSTSEC-2026-0387)) | High | `soroban-sdk` 22 -> `soroban-env-host` -> `k256` 0.13 -> `elliptic-curve` 0.13 | Every published Soroban SDK line still pins `elliptic-curve ^0.13`, and no `elliptic-curve` 0.13 release uses `crypto-bigint >=0.7.6`. `soroban-env-host` is declared only for `cfg(not(target_family = "wasm"))`, so the affected arithmetic is not present in the deployed contracts. Scoped exception lives in `.cargo/audit.toml`. | Remove `.cargo/audit.toml` once the Soroban dependency chain resolves `crypto-bigint >=0.7.6`, then rerun `cargo audit`. |

## Bug Bounty

We highly value the work of the security community. At this time, all validated and accepted reports of critical or high severity vulnerabilities in our core smart contracts may be eligible for a bug bounty. 

Bounty amounts are determined on a case-by-case basis depending on the severity and impact of the vulnerability. Tiers generally range from:
- **Critical:** Complete loss of funds or core protocol failure.
- **High:** Contract logic bypass or unauthorized data manipulation.
- **Medium/Low:** Limited impact UI bugs or defense-in-depth issues.

## Safe Harbor

We support safe harbor for security researchers who:
- Report the vulnerability to us privately and give us reasonable time to fix it before public disclosure.
- Make a good faith effort to avoid privacy violations, destruction of data, and interruption or degradation of our service.
- Do not exploit the vulnerability beyond what is necessary to demonstrate the issue.
- Comply with all applicable laws.

If you conduct your research and reporting in accordance with these guidelines, we will consider your actions authorized and will not initiate or support any legal action against you related to your research.
