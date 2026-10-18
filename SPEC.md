# Milestone Escrow Specification

## Funding modes

The escrow supports two funding modes.

### Upfront (default)

`initialize` always uses `FundingMode::Upfront`. The client calls `fund(client)` once and transfers the sum of every milestone amount. No milestone may be delivered or settled until that full deposit succeeds. Calling `fund_milestone` in this mode returns `InvalidFundingMode`.

### PerMilestone

`initialize_with_funding_mode(..., FundingMode::PerMilestone)` lets the client fund each milestone independently with `fund_milestone(client, milestone_index)`.

- Only the selected milestone amount is transferred.
- Duplicate funding of the same milestone returns `AlreadyFunded`.
- `fund` is rejected with `InvalidFundingMode`.
- A milestone that is not funded cannot be delivered, approved, disputed, released, refunded, or auto-released (`NotFunded`).
- Unrelated unfunded milestones do not block work on a funded milestone.
- `get_job` exposes each milestone's `funded` flag. `get_funding_mode` returns the mode.

Funding is recorded only after the client balance is confirmed sufficient, so a failed transfer does not mark a milestone funded.

## Conservation

A milestone can never release more than its funded amount. Partial approvals, refunds, and dispute splits operate only against that remaining funded balance.
