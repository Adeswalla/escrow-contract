# Milestone Escrow Specification

## Excess asset recovery

`sweep_excess(caller, token)` lets the **client or admin** recover assets that
are not required to settle the escrow.

- Escrow token: sweepable amount is `contract_balance - outstanding_obligations`.
  Obligations are each non-terminal milestone's `amount - released_amount`
  (Pending, Delivered, PartiallyReleased, Disputed), plus any accrued yield.
  Terminal statuses (`Released`, `Refunded`, `Split`) are excluded so they
  cannot be double-counted after settlement.
- If the contract balance is below obligations, the call returns
  `InsufficientBalance` and transfers nothing.
- Zero excess returns `NothingToSweep`.
- Other tokens: the full contract balance is recoverable (they are not used
  for milestone settlement).
- Excess is always sent to the client. Callers cannot pick another recipient.
- Emits `sweep` with token, amount, and recipient.
