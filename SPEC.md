# Milestone Escrow — Contract Specification

This document is the normative reference for the observable behaviour of the
`milestone-escrow` contract (`contracts/milestone-escrow/src/lib.rs`).
Where this document and the code disagree, the code is authoritative; please
open an issue so the two can be reconciled.

## Milestone lifecycle

Each milestone carries a `MilestoneStatus`:

| Status              | Meaning                                                              |
| ------------------- | -------------------------------------------------------------------- |
| `Pending`           | Funded, not yet delivered.                                           |
| `Delivered`         | The freelancer has marked the milestone delivered.                   |
| `PartiallyReleased` | Some, but not all, of the milestone has been released to the freelancer. |
| `Released`          | Fully paid out to the freelancer. Terminal.                          |
| `Refunded`          | Fully refunded to the client. Terminal.                              |
| `Disputed`          | Frozen for arbitration.                                              |
| `Split`             | Settled by an arbiter-directed split. Terminal.                      |

## Dispute semantics

A dispute is raised with `raise_dispute(caller, milestone_index)`. Either the
client or the freelancer may raise it, and the caller must authorize the call.
Raise-able statuses and their intended meaning:

| Status at raise time | Allowed? | Meaning                                                                 |
| -------------------- | -------- | ----------------------------------------------------------------------- |
| `Pending`            | Yes      | **Pre-delivery (scope) dispute.** Either party flags that the engagement terms are in question before any work is accepted. |
| `Delivered`          | Yes      | **Delivery (quality) dispute** on submitted work.                        |
| `PartiallyReleased`  | Yes      | **Remainder dispute** over the not-yet-released balance.                 |
| `Disputed`           | No       | Already disputed — returns `InvalidStatus` (see `DisputeAlreadyRaised` for the lock path). |
| `Released`           | No       | Terminal — returns `InvalidStatus`.                                      |
| `Refunded`           | No       | Terminal — returns `InvalidStatus`.                                      |
| `Split`              | No       | Terminal — returns `InvalidStatus`.                                      |

Notes:

- Raising a dispute moves the milestone to `Disputed` and freezes it. While
  `Disputed`, `mark_delivered` is refused (`mark_delivered` only accepts
  `Pending`), so the only way forward is the arbiter.
- The arbiter resolves a `Disputed` milestone with one of:
  - `resolve_dispute(release_to_freelancer = true)` → `Released`
  - `resolve_dispute(release_to_freelancer = false)` → `Refunded`
  - `apply_dispute_arbitration_split(client_refund_bps)` → `Split` (full
    refund becomes `Refunded`, full freelancer award becomes `Released`).
- Pre-delivery and delivery disputes share the same arbiter outcomes. The
  distinction is semantic: the arbiter is adjudicating scope/withhold for a
  `Pending` milestone versus work quality for a `Delivered` one.
- A freelancer who never delivers is handled by the escrow delivery deadline
  and the cancellation / auto-release paths — `raise_dispute` is not the
  mechanism for non-delivery.
- When a dispute is rejected, no milestone or storage state is mutated.

## Events

See [`EVENTS.md`](EVENTS.md) for the full topic reference. Every topic emitted
by the contract is listed there, and the `event_docs_sync_tests` suite fails
the build if a topic is added to the source without a matching row.

## Related

- [`README.md`](README.md) — build and test instructions.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — contribution workflow, including the
  end-to-end test harness.
