/// KDBX 4.x round-trip test: write then read
#[cfg(test)]
mod kdbx4_roundtrip {
    use crate::crypto::keys::CompositeKey;
    use crate::kdbx::{KdbxReader, KdbxWriter};
    use crate::vault::Vault;

    #[test]
    fn test_kdbx4_write_read_roundtrip() {
        let mut vault = Vault::new("Test Vault");
        let root_uuid = vault.root_group_uuid;
        let entry_uuid = vault.create_entry(root_uuid).unwrap();
        if let Some(entry) = vault.get_entry_mut(&entry_uuid) {
            entry.title.set("Test Entry");
            entry.username.set("user@example.com");
            entry.password.set("secret123");
        }

        // Build composite key
        let mut key = CompositeKey::new();
        key.add_password("test_password_kdbx4");
        let composite = key.build().unwrap();

        // Write
        let writer = KdbxWriter::new();
        let data = writer.write(&vault, &composite).expect("Write failed");
        println!("Written {} bytes", data.len());

        // Read back
        let reader = KdbxReader::new();
        let vault2 = reader.read(&data, &composite).expect("Read failed");

        assert_eq!(vault2.meta.name, "Test Vault");
        assert_eq!(vault2.entry_count(), 1);
        println!("Round-trip SUCCESS: {} entries", vault2.entry_count());
    }

    #[test]
    fn test_kdbx4_wrong_password() {
        let vault = Vault::new("Test");
        let mut key = CompositeKey::new();
        key.add_password("correct_password");
        let composite = key.build().unwrap();

        let writer = KdbxWriter::new();
        let data = writer.write(&vault, &composite).expect("Write failed");

        // Try wrong password
        let mut wrong_key = CompositeKey::new();
        wrong_key.add_password("wrong_password");
        let wrong_composite = wrong_key.build().unwrap();

        let reader = KdbxReader::new();
        let result = reader.read(&data, &wrong_composite);
        assert!(result.is_err(), "Should fail with wrong password");
        println!("Wrong password correctly rejected");
    }
}
