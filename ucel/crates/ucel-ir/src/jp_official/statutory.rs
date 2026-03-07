use super::fetch::JpOfficialAdapter;

pub fn statutory_adapter() -> JpOfficialAdapter {
    JpOfficialAdapter::new("edinet_api_documents_v2")
}
