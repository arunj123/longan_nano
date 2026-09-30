# Longan Nano Agent Guidelines & Invariants

See [GEMINI.md](file:///c:/Users/arunj/projects/longan_nano/GEMINI.md) for full hardware, peripheral, and build documentation.

## CRITICAL MANDATORY INVARIANT: PER-BUILD GIT COMMITS
> [!CAUTION]
> **STRICT COMMIT-AFTER-EVERY-BUILD ENFORCEMENT**:
> 1. **Immediate Commit After Testing**: After testing ANY firmware build on the target hardware or Linux host, you MUST create a git commit immediately documenting the build number and test results before running further tests, before modifying code, and before ending the turn.
> 2. **Session Start Verification**: At the start of EVERY new session or task, run `git status`. If uncommitted changes exist from a previously tested build, commit them immediately before proceeding with any other actions.
> 3. **Strict Commit Message Structure**: Every build commit message must follow the standardized 5-part schema:
>    - **Header**: `build(<component>): Build <NNNN> - <Concise Headline>`
>    - **Summary of Changes**: Explicit, bulleted breakdown of code, register, timing, or configuration adjustments across modified files and the design rationale behind them.
>    - **Test Results**: Concrete empirical observations from hardware/host testing (e.g., pass/fail status of SCSI tags, sector counts, host `dmesg`, `usbmon` packet trace status, transfer throughput).
>    - **Root Cause & Diagnostics**: Detailed register states, hardware counter readings, or findings explaining any observed failures.
>    - **What Next**: Prioritized, actionable roadmap outlining the exact steps and hypotheses to investigate in the subsequent build iteration.
> 4. **No Batching Across Builds**: Never combine multiple build iterations into a single commit. Every single build iteration (e.g., Build 009B, Build 009C, etc.) requires its own distinct commit.
