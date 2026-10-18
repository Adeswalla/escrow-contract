# Milestone Escrow Specification

## Emergency pause control

- `emergency_pause(client, freelancer)`: both parties must authorize. Sets a
  party-owned pause (`EpParty = true`).
- `emergency_unpause_parties(client, freelancer)`: both parties must authorize
  to lift a pause **they jointly created**. One party acting alone is
  `Unauthorized`. This path cannot lift an admin pause.
- `emergency_unpause(admin)`: admin recovery for any pause.
- `emergency_pause_admin_override(admin, paused)`: admin-only force pause or
  unpause. Clears `EpParty` so a later party unpause cannot undo an admin freeze.
- Unpausing a running escrow returns `NotPaused`.
- Pause state is per contract instance; unpausing one escrow does not affect
  another.
