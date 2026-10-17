# Longan Nano Agent Guidelines & Invariants

See [GEMINI.md](GEMINI.md) for full hardware, peripheral, and build documentation.

## CRITICAL MANDATORY INVARIANT: MASTER ROADMAP TRACKING & SESSION CLOSING
> [!IMPORTANT]
> **ROADMAP DOCUMENTATION & SESSION CLOSING INVARIANT**:
> 1. **Master Tracking Document**: [`docs/FIRMWARE_REVIEW_AND_ROADMAP.md`](docs/FIRMWARE_REVIEW_AND_ROADMAP.md) is the authoritative, permanent engineering record and multi-session implementation blueprint.
> 2. **Session Updates**: At the end of EVERY session, or immediately upon completing any roadmap item/milestone:
>    - The agent MUST update [`docs/FIRMWARE_REVIEW_AND_ROADMAP.md`](docs/FIRMWARE_REVIEW_AND_ROADMAP.md) to mark completed tasks (`[x]`), document architectural decisions, record empirical verification results, and note any blockers.
>    - Ensure all information needed to continue seamlessly in a brand-new session (exact next steps, active phase, file locations, known bugs/findings) is fully documented in the file without requiring the next agent to re-analyze from scratch.
> 3. **Session Start Verification**: At the start of EVERY new session or task, run `git status` and read [`docs/FIRMWARE_REVIEW_AND_ROADMAP.md`](docs/FIRMWARE_REVIEW_AND_ROADMAP.md) before performing any work.
> 4. **Roadmap Archival & Size Management**: When [`docs/FIRMWARE_REVIEW_AND_ROADMAP.md`](docs/FIRMWARE_REVIEW_AND_ROADMAP.md) grows beyond maintainable size (e.g. > 600-800 lines or > 50-60 KB), archive completed historical phases, milestones, detailed progress notes, and empirical verification logs into dedicated archive documents (e.g. `docs/archive/FIRMWARE_ROADMAP_PHASES_1_TO_4.md`), retaining in the active roadmap an executive summary/table of completed phases, the active phase, upcoming phases, and the active session handoff protocol to keep the document lightweight, fast to read, and fully maintainable.


## CRITICAL MANDATORY INVARIANT: ECONOMIC AI SESSION HANDOFF
> [!TIP]
> **ECONOMIC SESSION TRANSITION ENFORCEMENT**:
> 1. **Context & Token Economy**: Chat sessions with high token consumption or extended history become economically inefficient, slow, and prone to context degradation.
> 2. **Mandatory Handoff Prompting**: Whenever session context grows long, or when a discrete roadmap task/milestone/build verification is completed:
>    - The agent MUST ensure all changes are tested, documented in [`docs/FIRMWARE_REVIEW_AND_ROADMAP.md`](docs/FIRMWARE_REVIEW_AND_ROADMAP.md), and committed to git.
>    - The agent MUST explicitly direct and prompt the user to start a **new chat/agent session**, providing the exact prompt/command for the user to resume in the new session.

## CRITICAL MANDATORY INVARIANT: PER-BUILD GIT COMMITS
> [!CAUTION]
> **STRICT COMMIT-AFTER-EVERY-BUILD ENFORCEMENT**:
> 1. **Immediate Commit After Testing**: After testing ANY firmware build on the target hardware or Linux host, you MUST create a git commit immediately documenting the build number and test results before running further tests, before modifying code, and before ending the turn.
> 2. **Session Start Verification**: At the start of EVERY new session or task, run `git status`. If uncommitted changes exist from a previously tested build, commit them immediately before proceeding with any other actions.
> 3. **Strict Commit Message Structure**: Every build commit message must follow the standardized 5-part schema:
>    - **Header**: `build(<component>): Build <NNNN> - <Concise Headline>`
>    - **Summary of Changes**: Explicit, bulleted breakdown of code, register, timing, or configuration adjustments across modified files and the design rationale behind them.
>    - **Test Results**: Concrete empirical observations from hardware/host testing (e.g., pass/fail status of SCSI tags, sector counts, host `dmesg`, `usbmon` packet trace status, transfer throughput).
>    - **Root Cause & Diagnostics**: Detailed register states, hardware counter readings, or findings explaining any observed failures. **CRITICAL: In case of a build failure, compiler error, panic, or hardware test failure, all debug data, compiler error traces, register dumps, or bus logs currently obtained MUST be explicitly documented in this section!**
>    - **What Next**: Prioritized, actionable roadmap outlining the exact steps and hypotheses to investigate in the subsequent build iteration.
> 4. **No Batching Across Builds**: Never combine multiple build iterations into a single commit. Every single build iteration (e.g., Build 009B, Build 009C, etc.) requires its own distinct commit.
