// Integration test for Protocol v8 Polls
//
// This test demonstrates the poll lifecycle (create, vote, terminate) using the Presage library.
//
// NOTE: This is a template for manual integration testing.
// Automated testing requires:
// 1. A registered Signal account
// 2. Membership in a Signal group
// 3. Network access to Signal servers
//
// To run manual tests:
// 1. Set up environment variables or a test config
// 2. Replace placeholder values with actual credentials
// 3. Run: cargo test --test poll_integration_test -- --ignored

#[cfg(test)]
mod poll_integration_tests {
    use presage::{
        Manager,
        libsignal_service::prelude::Uuid,
        model::identity::OnNewIdentity,
    };
    use presage_store_sqlite::SqliteStore;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Helper to get test configuration from environment
    /// Set these environment variables to run the test:
    /// - TEST_SIGNAL_DB_PATH: Path to SQLite database
    /// - TEST_GROUP_MASTER_KEY: Hex string of group master key
    /// - TEST_YOUR_ACI: Your Signal ACI (UUID)
    fn get_test_config() -> Option<(String, Vec<u8>, Uuid)> {
        let db_path = std::env::var("TEST_SIGNAL_DB_PATH").ok()?;
        let master_key_hex = std::env::var("TEST_GROUP_MASTER_KEY").ok()?;
        let aci_str = std::env::var("TEST_YOUR_ACI").ok()?;

        let master_key = hex::decode(&master_key_hex).ok()?;
        if master_key.len() != 32 {
            return None;
        }

        let aci = Uuid::parse_str(&aci_str).ok()?;

        Some((db_path, master_key, aci))
    }

    #[tokio::test]
    #[ignore] // Requires real Signal account and network access
    async fn test_send_poll() -> anyhow::Result<()> {
        let Some((db_path, master_key, _aci)) = get_test_config() else {
            println!("Skipping test: Set TEST_SIGNAL_DB_PATH, TEST_GROUP_MASTER_KEY, TEST_YOUR_ACI");
            return Ok(());
        };

        // Load registered manager
        let store = SqliteStore::open_with_passphrase(&db_path, None, OnNewIdentity::Trust).await?;
        let mut manager = Manager::load_registered(store).await?;

        // Create a poll
        let question = "Integration Test Poll - What's your favorite Rust feature?";
        let options = vec![
            "Ownership & Borrowing".to_string(),
            "Pattern Matching".to_string(),
            "Trait System".to_string(),
            "Async/Await".to_string(),
        ];

        manager.send_poll(
            &master_key,
            question,
            options,
            false, // single choice
        ).await?;

        println!("✅ Poll sent successfully");
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Requires real Signal account and network access
    async fn test_vote_on_poll() -> anyhow::Result<()> {
        let Some((db_path, master_key, aci)) = get_test_config() else {
            println!("Skipping test: Set TEST_SIGNAL_DB_PATH, TEST_GROUP_MASTER_KEY, TEST_YOUR_ACI");
            return Ok(());
        };

        // You must set this to the timestamp of an existing poll
        let poll_timestamp: u64 = std::env::var("TEST_POLL_TIMESTAMP")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        if poll_timestamp == 0 {
            println!("Skipping test: Set TEST_POLL_TIMESTAMP to an existing poll's timestamp");
            return Ok(());
        }

        // Load registered manager
        let store = SqliteStore::open_with_passphrase(&db_path, None, OnNewIdentity::Trust).await?;
        let mut manager = Manager::load_registered(store).await?;

        // Vote on the poll (select option 0)
        manager.vote_on_poll(
            &master_key,
            aci.into(),
            poll_timestamp,
            vec![0], // Select first option
        ).await?;

        println!("✅ Vote cast successfully");
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Requires real Signal account and network access
    async fn test_terminate_poll() -> anyhow::Result<()> {
        let Some((db_path, master_key, _aci)) = get_test_config() else {
            println!("Skipping test: Set TEST_SIGNAL_DB_PATH, TEST_GROUP_MASTER_KEY, TEST_YOUR_ACI");
            return Ok(());
        };

        // You must set this to the timestamp of an existing poll
        let poll_timestamp: u64 = std::env::var("TEST_POLL_TIMESTAMP")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        if poll_timestamp == 0 {
            println!("Skipping test: Set TEST_POLL_TIMESTAMP to an existing poll's timestamp");
            return Ok(());
        }

        // Load registered manager
        let store = SqliteStore::open_with_passphrase(&db_path, None, OnNewIdentity::Trust).await?;
        let mut manager = Manager::load_registered(store).await?;

        // Terminate the poll
        manager.terminate_poll(&master_key, poll_timestamp).await?;

        println!("✅ Poll terminated successfully");
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Requires real Signal account and network access
    async fn test_full_poll_lifecycle() -> anyhow::Result<()> {
        let Some((db_path, master_key, aci)) = get_test_config() else {
            println!("Skipping test: Set TEST_SIGNAL_DB_PATH, TEST_GROUP_MASTER_KEY, TEST_YOUR_ACI");
            return Ok(());
        };

        // Load registered manager
        let store = SqliteStore::open_with_passphrase(&db_path, None, OnNewIdentity::Trust).await?;
        let mut manager = Manager::load_registered(store).await?;

        println!("Testing full poll lifecycle...");

        // 1. Create a poll
        let question = format!(
            "Test Poll {} - Pick a number",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        );
        let options = vec![
            "One".to_string(),
            "Two".to_string(),
            "Three".to_string(),
        ];

        let poll_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;

        manager.send_poll(
            &master_key,
            question,
            options,
            true, // allow multiple
        ).await?;
        println!("✅ Step 1: Poll created at timestamp {}", poll_timestamp);

        // Wait a moment for the message to be sent
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // 2. Vote on the poll
        manager.vote_on_poll(
            &master_key,
            aci.into(),
            poll_timestamp,
            vec![0, 2], // Select options 0 and 2
        ).await?;
        println!("✅ Step 2: Vote cast on poll");

        // Wait a moment
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // 3. Terminate the poll
        manager.terminate_poll(&master_key, poll_timestamp).await?;
        println!("✅ Step 3: Poll terminated");

        println!("✅ Full poll lifecycle test completed successfully!");
        Ok(())
    }

    #[test]
    fn test_poll_api_exists() {
        // This is a compile-time test to ensure the poll APIs are available
        // If this compiles successfully, it proves the poll methods exist
        println!("✅ Poll APIs are available (compile-time check)");
    }
}
