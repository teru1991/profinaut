use ucel_registry::okx::OkxCatalog;

use crate::CatalogContractIndex;

impl CatalogContractIndex {
    pub fn missing_okx_catalog_ids(&self, catalog: &OkxCatalog) -> Vec<String> {
        let mut missing = Vec::new();

        for id in catalog
            .rest_endpoints
            .iter()
            .map(|row| row.id.as_str())
            .chain(catalog.ws_channels.iter().map(|row| row.id.as_str()))
        {
            if !self.registered_tests.contains(id) {
                missing.push(id.to_string());
            }
        }

        missing
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use ucel_registry::okx::load_okx_catalog_from_repo_root;

    use crate::CatalogContractIndex;

    #[test]
    fn okx_contract_index_covers_all_catalog_rows() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_okx_catalog_from_repo_root(&repo_root).unwrap();

        let mut index = CatalogContractIndex::default();
        for id in catalog
            .rest_endpoints
            .iter()
            .map(|entry| entry.id.as_str())
            .chain(catalog.ws_channels.iter().map(|entry| entry.id.as_str()))
        {
            index.register_id(id);
        }

        let missing = index.missing_okx_catalog_ids(&catalog);
        assert!(missing.is_empty());
    }
}
