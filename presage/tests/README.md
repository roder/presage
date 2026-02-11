# Presage Integration Tests

This directory contains E2E integration tests that verify Presage functionality against real Signal servers.

## Prerequisites

1. **Signal Account**: A registered Signal account on staging servers
2. **Test Contacts**: Known UUIDs and profile keys for test members
3. **Network Access**: Connection to Signal staging servers
4. **Mobile Signal App**: (Optional but recommended) Signal app on mobile connected to staging servers for verification

## Group CRUD Integration Tests

Tests for creating, adding members, and removing members from Signal groups.

### Setup

Set the following environment variables:

```bash
# Required for all group tests
export TEST_SIGNAL_DB_PATH="/path/to/your/staging.db3"

# Required for create-group test
export TEST_MEMBER_1_UUID="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
export TEST_MEMBER_1_PROFILE_KEY="<hex_encoded_32_byte_profile_key>"

# Required for full lifecycle test (adding/removing members)
export TEST_MEMBER_2_UUID="yyyyyyyy-yyyy-yyyy-yyyy-yyyyyyyyyyyy"
export TEST_MEMBER_2_PROFILE_KEY="<hex_encoded_32_byte_profile_key>"

# Required for add-member and remove-member tests (use an existing group)
export TEST_GROUP_MASTER_KEY="<hex_encoded_32_byte_master_key>"
```

### Getting Test Values

#### 1. Setting up staging account

Use presage-cli to register on staging:

```bash
presage-cli --sqlite-db-path staging.db3 register \
  --servers staging \
  --phone-number +1234567890 \
  --captcha "https://signalcaptchas.org/..."
```

#### 2. Getting member UUIDs and profile keys

After syncing contacts, you can find UUIDs using:

```bash
presage-cli --sqlite-db-path staging.db3 list-contacts
```

To get profile keys, use:

```bash
# List contacts - profile keys are stored in the database
presage-cli --sqlite-db-path staging.db3 list-contacts | grep -A2 "uuid"
```

Alternatively, query the SQLite database directly:

```bash
sqlite3 staging.db3 "SELECT uuid, hex(profile_key) FROM contacts WHERE uuid IS NOT NULL LIMIT 5;"
```

### Running Tests

#### Test individual operations:

```bash
# Test group creation
cargo test --test group_crud_integration_test test_create_group -- --ignored --nocapture

# Test adding a member (requires existing group)
cargo test --test group_crud_integration_test test_add_member -- --ignored --nocapture

# Test removing a member (requires existing group)
cargo test --test group_crud_integration_test test_remove_member -- --ignored --nocapture
```

#### Test full lifecycle:

```bash
cargo test --test group_crud_integration_test test_full_group_crud_lifecycle -- --ignored --nocapture
```

This will:
1. Create a new group with initial member
2. Add a second member
3. Remove the second member
4. Print status at each step

#### Run all group CRUD tests:

```bash
cargo test --test group_crud_integration_test -- --ignored --nocapture
```

### Verification

After running tests, verify results:

1. **Mobile Signal App**: Check that the group appears with correct:
   - Group title
   - Member list
   - Changes reflected in real-time

2. **presage-cli**: List groups to verify:
   ```bash
   presage-cli --sqlite-db-path staging.db3 list-groups -v
   ```

### Testing with presage-cli directly

You can also test the CLI commands directly:

```bash
# Create a group
presage-cli --sqlite-db-path staging.db3 create-group \
  --title "My Test Group" \
  --member <UUID1> \
  --member <UUID2>

# The command will output the master key - save it!

# Add a member
presage-cli --sqlite-db-path staging.db3 add-member \
  -k <MASTER_KEY_HEX> \
  --uuid <NEW_MEMBER_UUID>

# Remove a member
presage-cli --sqlite-db-path staging.db3 remove-member \
  -k <MASTER_KEY_HEX> \
  --uuid <MEMBER_UUID>
```

## Poll Integration Tests

See `poll_integration_test.rs` for testing poll functionality.

## Troubleshooting

### "Contact not found" errors

Make sure you've synced contacts first:

```bash
presage-cli --sqlite-db-path staging.db3 sync-contacts
presage-cli --sqlite-db-path staging.db3 sync
```

### Profile key errors

If profile keys are invalid or empty:
1. Sync contacts to ensure you have the latest profile keys
2. Verify the hex encoding is correct (should be 64 hex characters = 32 bytes)
3. Try retrieving the profile explicitly:
   ```bash
   presage-cli --sqlite-db-path staging.db3 retrieve-profile --uuid <UUID>
   ```

### Group not visible on mobile

1. Ensure your mobile Signal app is connected to **staging** servers (not production)
2. Pull down to refresh the conversation list
3. Wait a few seconds for sync to complete
4. Check if any errors appear in the test output

## Notes

- All tests are marked with `#[ignore]` by default because they require real credentials
- Tests use `--nocapture` flag to see progress output
- Tests include delays (`tokio::time::sleep`) to allow for server propagation
- Always use staging servers for testing, never production
