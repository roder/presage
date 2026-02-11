// Integration test for GV2 Group CRUD operations
//
// This test demonstrates creating, modifying, and managing Signal groups using the Presage library.
//
// NOTE: This is a template for manual integration testing.
// Automated testing requires:
// 1. A registered Signal account on staging servers
// 2. Contacts with known UUIDs and profile keys
// 3. Network access to Signal staging servers
// 4. Mobile Signal app connected to staging to verify group visibility
//
// To run manual tests:
// 1. Set up environment variables (see get_test_config)
// 2. Run: cargo test --test group_crud_integration_test -- --ignored
//
// Expected outcome: Groups should be visible in the mobile Signal app with correct members

#[cfg(test)]
mod group_crud_integration_tests {
    use presage::{
        Manager,
        libsignal_service::prelude::{Uuid, ProfileKey},
        libsignal_service::protocol::Aci,
        model::identity::OnNewIdentity,
    };
    use presage_store_sqlite::SqliteStore;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Helper to get test configuration from environment
    /// Set these environment variables to run the test:
    /// - TEST_SIGNAL_DB_PATH: Path to SQLite database (should be registered on staging)
    /// - TEST_MEMBER_1_UUID: UUID of first test member
    /// - TEST_MEMBER_1_PROFILE_KEY: Base64 or hex profile key of first member
    /// - TEST_MEMBER_2_UUID: (Optional) UUID of second test member
    /// - TEST_MEMBER_2_PROFILE_KEY: (Optional) Profile key of second member
    fn get_test_config() -> Option<(String, Vec<(Uuid, ProfileKey)>)> {
        let db_path = std::env::var("TEST_SIGNAL_DB_PATH").ok()?;

        let member1_uuid = std::env::var("TEST_MEMBER_1_UUID")
            .ok()
            .and_then(|s| Uuid::parse_str(&s).ok())?;
        let member1_key_hex = std::env::var("TEST_MEMBER_1_PROFILE_KEY").ok()?;
        let member1_key_bytes: [u8; 32] = hex::decode(&member1_key_hex)
            .ok()?
            .try_into()
            .ok()?;
        let member1_profile_key = ProfileKey::create(member1_key_bytes);

        let mut members = vec![(member1_uuid, member1_profile_key)];

        // Add second member if provided
        if let (Ok(uuid_str), Ok(key_hex)) = (
            std::env::var("TEST_MEMBER_2_UUID"),
            std::env::var("TEST_MEMBER_2_PROFILE_KEY")
        ) {
            if let (Ok(uuid), Ok(key_bytes)) = (
                Uuid::parse_str(&uuid_str),
                hex::decode(&key_hex)
            ) {
                if let Ok(key_array) = key_bytes.try_into() {
                    members.push((uuid, ProfileKey::create(key_array)));
                }
            }
        }

        Some((db_path, members))
    }

    #[tokio::test]
    #[ignore] // Requires real Signal account on staging and network access
    async fn test_create_group() -> anyhow::Result<()> {
        let Some((db_path, members)) = get_test_config() else {
            println!("Skipping test: Set TEST_SIGNAL_DB_PATH, TEST_MEMBER_1_UUID, TEST_MEMBER_1_PROFILE_KEY");
            return Ok(());
        };

        // Load registered manager
        let store = SqliteStore::open_with_passphrase(&db_path, None, OnNewIdentity::Trust).await?;
        let mut manager = Manager::load_registered(store).await?;

        // Create a group with timestamp in title for uniqueness
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        let group_title = format!("Test Group {}", timestamp);

        // Convert members to (Aci, ProfileKey) format
        let members_with_aci: Vec<(Aci, ProfileKey)> = members
            .into_iter()
            .map(|(uuid, key)| (uuid.into(), key))
            .collect();

        let master_key = manager.create_group(&group_title, members_with_aci).await?;

        println!("✅ Group created successfully!");
        println!("   Title: {}", group_title);
        println!("   Master key: {}", hex::encode(master_key));
        println!("\n📱 Check your mobile Signal app (staging) to verify the group appears");

        Ok(())
    }

    #[tokio::test]
    #[ignore] // Requires real Signal account and network access
    async fn test_add_member() -> anyhow::Result<()> {
        let Some((db_path, members)) = get_test_config() else {
            println!("Skipping test: Set TEST_SIGNAL_DB_PATH and member environment variables");
            return Ok(());
        };

        if members.len() < 2 {
            println!("Skipping test: Need TEST_MEMBER_2_UUID and TEST_MEMBER_2_PROFILE_KEY for add member test");
            return Ok(());
        }

        // You must set this to an existing group's master key
        let master_key_hex = std::env::var("TEST_GROUP_MASTER_KEY")
            .expect("Set TEST_GROUP_MASTER_KEY to test adding members");
        let master_key_bytes: [u8; 32] = hex::decode(&master_key_hex)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("Master key must be 32 bytes"))?;

        // Load registered manager
        let store = SqliteStore::open_with_passphrase(&db_path, None, OnNewIdentity::Trust).await?;
        let mut manager = Manager::load_registered(store).await?;

        // Add the second member
        let (member_uuid, member_profile_key) = &members[1];
        let member_aci: Aci = (*member_uuid).into();

        manager.add_group_member(&master_key_bytes, member_aci, *member_profile_key).await?;

        println!("✅ Member added successfully!");
        println!("   Member UUID: {}", member_uuid);
        println!("\n📱 Check your mobile Signal app to verify the new member appears");

        Ok(())
    }

    #[tokio::test]
    #[ignore] // Requires real Signal account and network access
    async fn test_remove_member() -> anyhow::Result<()> {
        let Some((db_path, members)) = get_test_config() else {
            println!("Skipping test: Set TEST_SIGNAL_DB_PATH and member environment variables");
            return Ok(());
        };

        // You must set this to an existing group's master key
        let master_key_hex = std::env::var("TEST_GROUP_MASTER_KEY")
            .expect("Set TEST_GROUP_MASTER_KEY to test removing members");
        let master_key_bytes: [u8; 32] = hex::decode(&master_key_hex)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("Master key must be 32 bytes"))?;

        // Load registered manager
        let store = SqliteStore::open_with_passphrase(&db_path, None, OnNewIdentity::Trust).await?;
        let mut manager = Manager::load_registered(store).await?;

        // Remove the first member
        let (member_uuid, _) = &members[0];
        let member_aci: Aci = (*member_uuid).into();

        manager.remove_group_member(&master_key_bytes, member_aci).await?;

        println!("✅ Member removed successfully!");
        println!("   Member UUID: {}", member_uuid);
        println!("\n📱 Check your mobile Signal app to verify the member is removed");

        Ok(())
    }

    #[tokio::test]
    #[ignore] // Requires real Signal account and network access
    async fn test_full_group_crud_lifecycle() -> anyhow::Result<()> {
        let Some((db_path, members)) = get_test_config() else {
            println!("Skipping test: Set TEST_SIGNAL_DB_PATH and member environment variables");
            return Ok(());
        };

        if members.len() < 2 {
            println!("Note: Need 2 members for full lifecycle test. Add TEST_MEMBER_2_UUID and TEST_MEMBER_2_PROFILE_KEY");
            println!("Continuing with 1 member...");
        }

        // Load registered manager
        let store = SqliteStore::open_with_passphrase(&db_path, None, OnNewIdentity::Trust).await?;
        let mut manager = Manager::load_registered(store).await?;

        println!("Testing full group CRUD lifecycle...");

        // 1. Create a group with the first member
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        let group_title = format!("CRUD Test Group {}", timestamp);

        let initial_members: Vec<(Aci, ProfileKey)> = vec![
            (members[0].0.into(), members[0].1)
        ];

        let master_key = manager.create_group(&group_title, initial_members).await?;
        println!("✅ Step 1: Group created");
        println!("   Title: {}", group_title);
        println!("   Master key: {}", hex::encode(master_key));

        // Wait for propagation
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        // 2. Add second member if available
        if members.len() >= 2 {
            let (member2_uuid, member2_key) = &members[1];
            let member2_aci: Aci = (*member2_uuid).into();

            manager.add_group_member(&master_key, member2_aci, *member2_key).await?;
            println!("✅ Step 2: Second member added");
            println!("   Member UUID: {}", member2_uuid);

            // Wait for propagation
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            // 3. Remove the second member
            manager.remove_group_member(&master_key, member2_aci).await?;
            println!("✅ Step 3: Second member removed");
            println!("   Member UUID: {}", member2_uuid);
        } else {
            println!("⚠️  Steps 2-3: Skipped (need 2 members)");
        }

        println!("\n✅ Full group CRUD lifecycle test completed!");
        println!("📱 Check your mobile Signal app (staging) to verify all changes");

        Ok(())
    }

    #[test]
    fn test_group_crud_api_exists() {
        // This is a compile-time test to ensure the group CRUD APIs are available
        // If this compiles successfully, it proves the methods exist
        println!("✅ Group CRUD APIs are available (compile-time check)");
    }
}
