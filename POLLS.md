# Signal Protocol v8 Polls in Presage

This document describes how to test Signal protocol v8 poll functionality using the presage-cli tool.

## Overview

Presage supports Signal's protocol v8 polls, enabling:
- Create polls in groups with 2-10 options
- Vote on polls (single or multiple choice)
- Terminate polls (poll creator only)

**Note**: Polls only work in groups (not 1:1 chats).

## Prerequisites

### 1. Build the CLI

```bash
cargo build --bin presage-cli --release
```

### 2. Register or Link a Device

#### Option A: Link as Secondary Device (Recommended for Testing)

```bash
cargo run --bin presage-cli -- link-device --device-name "presage-test"
```

Scan the QR code with your Signal mobile app.

#### Option B: Register New Number (Staging Servers)

```bash
cargo run --bin presage-cli -- register \
    --servers staging \
    --phone-number "+1234567890" \
    --captcha "https://signalcaptchas.org/..."
```

### 3. Find a Test Group

Get your group's master key (64-character hex string, 32 bytes):

```bash
cargo run --bin presage-cli -- list-groups
```

Output:
```
<master_key_hex> Group Name: Description / revision 5 / 3 members
```

### 4. Get Your UUID

```bash
cargo run --bin presage-cli -- whoami
```

Output:
```
WhoAmIResponse { aci: "12345678-1234-1234-1234-123456789abc", pni: "...", ... }
```

---

## Quick Test (5 minutes)

### 1. Create a Poll

```bash
cargo run --bin presage-cli -- send-poll \
    --master-key "YOUR_GROUP_KEY_HERE" \
    --question "What's for lunch?" \
    -o "Pizza" -o "Tacos" -o "Sushi" -o "Salad"
```

**Parameters:**
- `--master-key` / `-k`: Group master key (hex string)
- `--question` / `-q`: The poll question
- `-o`: Answer option (repeat for each option, 2-10 options)
- `--allow-multiple`: Allow multiple selections (optional)

### 2. Sync to See the Poll

```bash
cargo run --bin presage-cli -- sync
```

Output:
```
📊 Poll: What's for lunch?
  [0] Pizza
  [1] Tacos
  [2] Sushi
  [3] Salad
(single choice)
```

### 3. Vote on the Poll

Get the poll timestamp from the sync output or list-messages:

```bash
cargo run --bin presage-cli -- list-messages --group-master-key YOUR_GROUP_KEY_HERE
```

Then vote:

```bash
cargo run --bin presage-cli -- vote-poll \
    --master-key "YOUR_GROUP_KEY_HERE" \
    --poll-author-uuid "POLL_CREATOR_UUID_HERE" \
    --poll-timestamp TIMESTAMP_HERE \
    --selected-options 1
```

**Parameters:**
- `--master-key` / `-k`: Group master key (hex string)
- `--poll-author-uuid`: UUID of the poll creator
- `--poll-timestamp`: Timestamp of the poll message
- `--selected-options` / `-s`: Indices of selected options (0-based, comma-separated for multiple)

### 4. Terminate the Poll

Only the poll creator can terminate:

```bash
cargo run --bin presage-cli -- terminate-poll \
    --master-key "YOUR_GROUP_KEY_HERE" \
    --poll-timestamp TIMESTAMP_HERE
```

---

## Complete Example Session

```bash
# 1. Link device (do this once)
cargo run --bin presage-cli -- link-device --device-name "Presage Poll Test"

# 2. Get your group master key
cargo run --bin presage-cli -- list-groups
# Output: abc123def456... Family Group: ... / revision 3 / 5 members

# 3. Get your UUID
cargo run --bin presage-cli -- whoami
# Output: WhoAmIResponse { aci: "12345678-...", pni: "...", ... }

# 4. Create a poll
cargo run --bin presage-cli -- send-poll \
    -k abc123def456... \
    -q "Weekend plans?" \
    -o "Movie night" -o "Game night" -o "Dinner out" \
    --allow-multiple

# 5. View the poll and get timestamp
cargo run --bin presage-cli -- list-messages --group-master-key abc123def456...
# You'll see: 📊 Poll: Weekend plans? @ 1707438947123

# 6. Vote on the poll (select options 0 and 1)
cargo run --bin presage-cli -- vote-poll \
    -k abc123def456... \
    --poll-author-uuid 12345678-1234-1234-1234-123456789abc \
    --poll-timestamp 1707438947123 \
    -s 0,1

# 7. Terminate the poll
cargo run --bin presage-cli -- terminate-poll \
    -k abc123def456... \
    --poll-timestamp 1707438947123
```

---

## Using the Test Script

A bash script automates the test sequence:

```bash
# 1. Set environment variables
export SIGNAL_TEST_DB_PATH="./test.db"
export TEST_GROUP_KEY="your_64_char_hex_group_key"

# 2. Run the script
./examples/test_polls.sh
```

The script will:
1. Create a test poll
2. Prompt you for the poll timestamp (from sync output)
3. Prompt you for the poll author's UUID (from whoami)
4. Submit a vote
5. Optionally terminate the poll

---

## API Reference

The presage library provides three methods for poll operations:

### Create a Poll

```rust
manager.send_poll(
    &master_key_bytes,     // Group's 32-byte master key
    "What's for lunch?",   // Poll question
    vec![                  // Answer options (2-10)
        "Pizza".to_string(),
        "Tacos".to_string(),
        "Salad".to_string(),
    ],
    false,                 // Allow multiple selections?
).await?;
```

### Vote on a Poll

```rust
manager.vote_on_poll(
    &master_key_bytes,     // Group's master key
    poll_author_uuid,      // UUID of poll creator
    poll_timestamp,        // Original poll message timestamp
    vec![0],               // Selected option indices (0-based)
).await?;
```

### Terminate a Poll

```rust
manager.terminate_poll(
    &master_key_bytes,     // Group's master key
    poll_timestamp,        // Poll message timestamp to terminate
).await?;
```

---

## Automated Integration Testing

### Test Harness

```rust
use presage::{Manager, libsignal_service::prelude::Uuid};
use presage_store_sqlite::SqliteStore;

#[tokio::test]
#[ignore] // Requires real Signal credentials
async fn test_poll_lifecycle() {
    // 1. Initialize manager
    let store = SqliteStore::open("/tmp/test.db", None, OnNewIdentity::Trust).await?;
    let mut manager = Manager::load_registered(store).await?;

    // 2. Send poll
    let master_key = /* group master key bytes */;
    manager.send_poll(
        &master_key,
        "Test question?",
        vec!["Option 1".to_string(), "Option 2".to_string()],
        false
    ).await?;

    // 3. Vote on poll
    let poll_author_uuid = /* poll creator's UUID */;
    let poll_timestamp = /* timestamp */;
    manager.vote_on_poll(
        &master_key,
        poll_author_uuid,
        poll_timestamp,
        vec![0]
    ).await?;

    // 4. Terminate poll
    manager.terminate_poll(&master_key, poll_timestamp).await?;
}
```

### Environment Variables

```bash
export SIGNAL_TEST_PHONE="+1234567890"
export SIGNAL_TEST_DB_PATH="/path/to/test.db"
export SIGNAL_TEST_GROUP_KEY="hex_encoded_key"
```

### Running Tests

```bash
# Unit tests
cargo test --package presage

# Integration tests (requires credentials)
cargo test --package presage -- --ignored --test-threads=1
```

---

## Test Coverage Checklist

### Core Functionality
- [ ] Create poll with 2-10 options
- [ ] Create poll with single/multiple selection modes
- [ ] Vote on poll (single option)
- [ ] Vote on poll (multiple options if allowed)
- [ ] Terminate poll as creator
- [ ] Receive poll messages
- [ ] Receive vote messages
- [ ] Receive terminate messages

### Edge Cases
- [ ] Create poll with empty options (should fail)
- [ ] Create poll with > 10 options (should fail)
- [ ] Vote with out-of-range option index
- [ ] Vote multiple times (should update vote)
- [ ] Terminate already-terminated poll
- [ ] Terminate poll as non-creator (server should reject)

### Protocol Validation
- [ ] Messages use protocol v8 format
- [ ] Poll messages include required fields
- [ ] Vote messages reference correct poll
- [ ] Terminate messages close poll properly

---

## Storage Backends

### SQLite Store (Default)

```bash
cargo run --bin presage-cli -- --sqlite-db-path /path/to/db.sqlite3 <command>
```

With encryption:

```bash
cargo run --bin presage-cli -- --sqlite-db-path /path/to/db.sqlite3 --passphrase "secret" <command>
```

### Custom Store (e.g., StromaStore)

Implement the `Store` trait. See `presage/src/store.rs` for the trait definition.

---

## Troubleshooting

### "Master key should be 32 bytes long"
- Ensure your group key is exactly 64 hex characters (32 bytes)
- Get it from `list-groups` command

### "Poll must have at least 2 options"
- Polls require minimum 2 options and maximum 10

### "Must select at least one option"
- When voting, select at least one option

### "Failed to send message"
- Check you're registered/linked: `cargo run --bin presage-cli -- whoami`
- Ensure you're a member of the group
- Try syncing first: `cargo run --bin presage-cli -- sync`

### "Not seeing poll messages"
- Run `sync` after sending
- Check you're looking at the right group
- Verify the group key is correct

### "Wrong Poll Timestamp"
- The poll timestamp must match exactly
- Check messages: `cargo run --bin presage-cli -- list-messages --group-master-key <key>`

### Poll commands not available
- Make sure you built from the correct branch with poll support
- Check `cargo run --bin presage-cli -- --help` shows poll commands

### Authentication Errors
- Verify registration: `cargo run --bin presage-cli -- whoami`

### Group Not Found
- Verify the group master key: `cargo run --bin presage-cli -- list-groups`

---

## Protocol Details

### Protocol v8 Support

Polls require protocol v8, which includes:
- `PollCreate`: Creates a new poll with question and options
- `PollVote`: Casts votes on a poll (one or more option indices)
- `PollTerminate`: Closes a poll to prevent further voting

### DataMessage Structure

Polls are sent as `DataMessage` with one of these fields:
- `poll_create`: Contains question, options, and allow_multiple flag
- `poll_vote`: Contains target_author_aci_binary, target_sent_timestamp, option_indexes
- `poll_terminate`: Contains target_sent_timestamp

### Protocol Constraints
- Polls only work in groups (not 1:1 chats)
- Maximum 10 options per poll
- Minimum 2 options required
- Multiple selection is optional
- Only poll creator can terminate
- Vote updates replace previous votes

### libsignal-service Dependency

Poll support depends on `libsignal-service-rs` with protocol v8:
- Repository: https://github.com/whisperfish/libsignal-service-rs
- Branch/commit must include v8 poll support

---

## Server Environments

### Staging Servers (for development)

```bash
cargo run --bin presage-cli -- register --servers staging ...
```

### Production Servers (default)

```bash
cargo run --bin presage-cli -- register --servers production ...
```

---

## Known Limitations

1. **Real Signal Account Required**
   - No mock server available for full E2E testing
   - Staging servers available but limited
   - Tests must be careful not to spam real accounts

2. **Async Nature**
   - Poll creation and voting are async
   - Must receive messages to see results
   - Tests need to handle timing

3. **Group Only**
   - Polls only work in groups, not 1:1 chats
   - Requires test group setup

---

## References

- [Signal Protocol](https://signal.org/docs/)
- [libsignal-service-rs](https://github.com/whisperfish/libsignal-service-rs)
- [Presage Documentation](https://whisperfish.github.io/presage/presage)
- [Signal Desktop Source](https://github.com/signalapp/Signal-Desktop) - reference implementation
- [Presage Issues](https://github.com/whisperfish/presage/issues)
