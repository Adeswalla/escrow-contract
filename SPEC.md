# Milestone Escrow Specification

## Revision workflow

After a milestone is `Delivered`, the authorized client may call
`request_revision(client, milestone_index, note_hash)`.

- The milestone returns to `Pending`.
- `delivered_at` is cleared (persistent and temporary), so the previous
  auto-release window cannot fire.
- `revision_count` increments by one. `MAX_REVISIONS` is 3.
- Further revision requests after the cap return `MaxRevisions`; the client
  must approve, partially approve, or dispute.
- The on-chain event `msrev` carries `milestone_index`, `note_hash`, and
  `revision_count`. The note itself is not stored.
- A later `mark_delivered` writes a fresh `delivered_at` and starts a new
  auto-release window.
- Revision does not change funding, released amounts, or escrow balances.
