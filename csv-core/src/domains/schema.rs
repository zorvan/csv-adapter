//! Schema Domain
//!
//! Domain-separated hashing for CSV contract schemas.

use crate::domain_hash::Domain;

/// Schema domain for CSV contract schema hashing
pub struct SchemaDomain;

impl Domain for SchemaDomain {
    const DOMAIN: &'static [u8] = b"csv.schema.v1";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_domain_tag() {
        assert_eq!(SchemaDomain::DOMAIN, b"csv.schema.v1");
    }
}
