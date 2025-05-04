use std::path::Path;
use std::io::Write;

use crate::FeatureMember;

use super::FeatureMemberEmitter;

pub struct MultiFileEmitter {
    address_writer: std::io::BufWriter<std::fs::File>,
    admin_unit_writer: std::io::BufWriter<std::fs::File>,
    address_area_writer: std::io::BufWriter<std::fs::File>,
    thoroughfare_writer: std::io::BufWriter<std::fs::File>,
}

impl MultiFileEmitter {
    pub fn new(base_path: &Path) -> Self {
        // Create separate files for each feature member type
        let address_file = std::fs::File::create(base_path.join("addresses.tsv"))
            .expect("Failed to create address file");
        let admin_unit_file = std::fs::File::create(base_path.join("admin_units.tsv"))
            .expect("Failed to create admin unit file");
        let address_area_file = std::fs::File::create(base_path.join("address_areas.tsv"))
            .expect("Failed to create address area file");
        let thoroughfare_file = std::fs::File::create(base_path.join("thoroughfares.tsv"))
            .expect("Failed to create thoroughfare file");

        // Create buffered writers for each file
        let address_writer = std::io::BufWriter::new(address_file);
        let admin_unit_writer = std::io::BufWriter::new(admin_unit_file);
        let address_area_writer = std::io::BufWriter::new(address_area_file);
        let thoroughfare_writer = std::io::BufWriter::new(thoroughfare_file);

        MultiFileEmitter {
            address_writer,
            admin_unit_writer,
            address_area_writer,
            thoroughfare_writer,
        }
    }
}

impl FeatureMemberEmitter for MultiFileEmitter {
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
                // Write the address data to the address file
                writeln!(
                    self.address_writer,
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    local_id,
                    number.unwrap_or_default(),
                    number_extension.unwrap_or_default(),
                    number_2nd_extension.unwrap_or_default(),
                    postal_delivery_identifier.unwrap_or_default(),
                    admin_unit_ref.unwrap_or_default(),
                    address_area_ref.unwrap_or_default(),
                    thoroughfare_ref.unwrap_or_default()
                )
                .expect("Failed to write to address file");
            }
            FeatureMember::AdminUnitName { local_id, name } => {
                // Write the admin unit name data to the admin unit file
                writeln!(
                    self.admin_unit_writer,
                    "{}\t{}",
                    local_id,
                    name.unwrap_or_default()
                )
                .expect("Failed to write to admin unit file");
            }
            FeatureMember::AddressAreaName {
                local_id,
                name,
                situated_in_ref,
            } => {
                // Write the address area name data to the address area file
                writeln!(
                    self.address_area_writer,
                    "{}\t{}\t{}",
                    local_id,
                    name.unwrap_or_default(),
                    situated_in_ref.unwrap_or_default()
                )
                .expect("Failed to write to address area file");
            }
            FeatureMember::ThoroughfareName {
                local_id,
                name,
                situated_in_ref,
            } => {
                // Write the thoroughfare name data to the thoroughfare file
                writeln!(
                    self.thoroughfare_writer,
                    "{}\t{}\t{}",
                    local_id,
                    name.unwrap_or_default(),
                    situated_in_ref.unwrap_or_default()
                )
                .expect("Failed to write to thoroughfare file");
            }
        }
    }

    fn start(&mut self) {
        // Write headers to each file
        writeln!(
            self.address_writer,
            "local_id\tnumber\tnumber_extension\tnumber_2nd_extension\tpostal_delivery_identifier\tadmin_unit_ref\taddress_area_ref\tthoroughfare_ref"
        ).expect("Failed to write address header");
        self.address_writer
            .flush()
            .expect("Failed to flush address writer");

        writeln!(self.admin_unit_writer, "local_id\tname").expect("Failed to write admin unit header");
        self.admin_unit_writer
            .flush()
            .expect("Failed to flush admin unit writer");

        writeln!(self.address_area_writer, "local_id\tname\tsituated_in_ref")
            .expect("Failed to write address area header");
        self.address_area_writer
            .flush()
            .expect("Failed to flush address area writer");

        writeln!(self.thoroughfare_writer, "local_id\tname\tsituated_in_ref")
            .expect("Failed to write thoroughfare header");
        self.thoroughfare_writer
            .flush()
            .expect("Failed to flush thoroughfare writer");
    }

    fn end(&mut self) {
        self.address_writer
            .flush()
            .expect("Failed to flush address writer");
        self.admin_unit_writer
            .flush()
            .expect("Failed to flush admin unit writer");
        self.address_area_writer
            .flush()
            .expect("Failed to flush address area writer");
        self.thoroughfare_writer
            .flush()
            .expect("Failed to flush thoroughfare writer");
    }
}
