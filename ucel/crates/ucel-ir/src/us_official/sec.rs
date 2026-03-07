use super::fetch::UsOfficialAdapter;

pub fn sec_adapter() -> UsOfficialAdapter {
    UsOfficialAdapter::new("sec_edgar_submissions_api")
}
