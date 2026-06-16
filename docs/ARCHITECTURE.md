# Architecture

RemitSure locks a payment for one named payee, so money sent for school fees, rent or a hospital bill can only go there.

## Lifecycle of a remittance

1. A payee (school, landlord, hospital, utility) calls `register_payee` with a display name and a category. It starts unverified.
2. The admin checks the payee off chain and calls `set_verified`. Unverified payees can still receive, but the app warns senders.
3. A sender calls `send` with the payee, token, amount, a reference ("Term 1 fees, Ada Obi, JSS2") and a deadline. The funds move into the contract.
4. Before the deadline, the payee either calls `claim` (funds go to the payee) or `reject` (funds go straight back to the sender).
5. If the deadline passes with no claim, the sender calls `refund`.

## State machine

```
            claim (payee, before deadline)
Pending ───────────────────────────────▶ Claimed
   │
   ├── reject (payee, any time) ───────▶ Rejected   (sender refunded)
   │
   └── refund (sender, after deadline) ▶ Refunded
```

Every terminal state is final. There is no way to move funds out of a remittance except these three calls.

## Who can do what

| Function         | Signer          | Notes                                       |
| ---------------- | --------------- | ------------------------------------------- |
| `register_payee` | payee           | Once per address                            |
| `set_verified`   | admin           | Trust badge only, not required to receive   |
| `set_admin`      | admin           | Rotate the admin key                        |
| `send`           | sender          | Payee must be registered, deadline within 366 days |
| `claim`          | payee           | Before the deadline                         |
| `reject`         | payee           | Any time while pending                      |
| `refund`         | sender          | At or after the deadline                    |

The admin can never move funds. It only manages the verified badge.

## Storage

| Key                 | Type         | Storage    |
| ------------------- | ------------ | ---------- |
| `Admin`             | `Address`    | instance   |
| `RemittanceCount`   | `u64`        | instance   |
| `Remittance(id)`    | `Remittance` | persistent |
| `Payee(address)`    | `Payee`      | persistent |

Persistent entries have their TTL bumped on every write.

## Events

The backend indexer depends on these. Treat them as public API.

| Event                 | Topic   | Data                                                      |
| --------------------- | ------- | --------------------------------------------------------- |
| `payee_registered`    | `payee` | name, category                                            |
| `payee_verified`      | `payee` | verified                                                  |
| `remittance_sent`     | `id`    | sender, payee, token, amount, reference, expires_at       |
| `remittance_claimed`  | `id`    | payee, amount                                             |
| `remittance_rejected` | `id`    | payee, amount                                             |
| `remittance_refunded` | `id`    | sender, amount                                            |
| `admin_changed`       |         | old_admin, new_admin                                      |

## Known limits (MVP)

- Payees receive tokens in a Stellar wallet. Cashing out to local currency through a Stellar anchor (SEP-24) is on the roadmap.
- Any token can be sent. A token allowlist is planned.
- Payees cannot update their name or category after registering yet.
- Partial claims and split payments (for example half to school, half to uniform vendor) are not supported yet.
