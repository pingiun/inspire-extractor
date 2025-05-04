use std::path::Path;

use crate::FeatureMember;

use super::FeatureMemberEmitter;

pub struct SqliteEmitter {
    db: rusqlite::Connection,
}

impl SqliteEmitter {
    pub fn new(db_path: &Path) -> rusqlite::Result<Self> {
        let db = rusqlite::Connection::open(db_path)?;
        db.pragma_update(None, "foreign_keys", "OFF")?;
        db.pragma_update(None, "journal_mode", "OFF")?;
        db.pragma_update(None, "synchronous", "OFF")?;
        db.pragma_update(None, "temp_store", "MEMORY")?;
        db.pragma_update(None, "cache_size", "10000")?;
        db.pragma_update(None, "locking_mode", "EXCLUSIVE")?;

        Ok(SqliteEmitter { db })
    }

    pub fn create_tables(&self) -> rusqlite::Result<()> {
        self.db.execute(
            "CREATE TABLE IF NOT EXISTS addresses (
                local_id INTEGER PRIMARY KEY,
                number TEXT,
                number_extension TEXT,
                number_2nd_extension TEXT,
                postal_delivery_identifier TEXT,
                admin_unit_ref TEXT,
                address_area_ref TEXT,
                thoroughfare_ref TEXT
            )",
            (),
        )?;

        self.db.execute(
            "CREATE TABLE IF NOT EXISTS admin_units (
                local_id INTEGER PRIMARY KEY,
                name TEXT
            )",
            (),
        )?;

        self.db.execute(
            "CREATE TABLE IF NOT EXISTS address_areas (
                local_id INTEGER PRIMARY KEY,
                name TEXT,
                situated_in_ref TEXT
            )",
            (),
        )?;

        self.db.execute(
            "CREATE TABLE IF NOT EXISTS thoroughfares (
                local_id INTEGER PRIMARY KEY,
                name TEXT,
                situated_in_ref TEXT
            )",
            (),
        )?;

        Ok(())
    }
}

impl FeatureMemberEmitter for SqliteEmitter {
    fn emit(&mut self, feature_member: FeatureMember) {
        match feature_member {
            FeatureMember::Address {
                local_id,
                number,
                number_extension,
                number_2nd_extension,
                postal_delivery_identifier,
                admin_unit_ref,
                address_area_ref,
                thoroughfare_ref,
            } => {
                self.db.execute(
                    "INSERT INTO addresses (
                        local_id, number, number_extension, number_2nd_extension,
                        postal_delivery_identifier, admin_unit_ref, address_area_ref, thoroughfare_ref
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    rusqlite::params![
                        local_id,
                        number,
                        number_extension,
                        number_2nd_extension,
                        postal_delivery_identifier,
                        admin_unit_ref,
                        address_area_ref,
                        thoroughfare_ref
                    ],
                ).expect("Failed to write to address file");
            }
            FeatureMember::AdminUnitName { local_id, name } => {
                self.db
                    .execute(
                        "INSERT INTO admin_units (local_id, name) VALUES (?, ?)",
                        rusqlite::params![local_id, name],
                    )
                    .expect("Failed to write to admin unit file");
            }
            FeatureMember::AddressAreaName {
                local_id,
                name,
                situated_in_ref,
            } => {
                self.db.execute(
                    "INSERT INTO address_areas (local_id, name, situated_in_ref) VALUES (?, ?, ?)",
                    rusqlite::params![local_id, name, situated_in_ref],
                ).expect("Failed to write to address area file");
            }
            FeatureMember::ThoroughfareName {
                local_id,
                name,
                situated_in_ref,
            } => {
                self.db.execute(
                    "INSERT INTO thoroughfares (local_id, name, situated_in_ref) VALUES (?, ?, ?)",
                    rusqlite::params![local_id, name, situated_in_ref],
                ).expect("Failed to write to thoroughfare file");
            }
        }
    }

    fn start(&mut self) {
        // Truncate tables
        self.db
            .execute("DELETE FROM addresses", [])
            .expect("Failed to truncate addresses table");
        self.db
            .execute("DELETE FROM admin_units", [])
            .expect("Failed to truncate admin units table");
        self.db
            .execute("DELETE FROM address_areas", [])
            .expect("Failed to truncate address areas table");
        self.db.execute("DELETE FROM thoroughfares", [])
            .expect("Failed to truncate thoroughfares table");
        self.db
            .execute("BEGIN TRANSACTION", [])
            .expect("Failed to begin transaction");
    }

    fn end(&mut self) {
        self.db
            .execute("COMMIT", [])
            .expect("Failed to commit transaction");
    }
}
