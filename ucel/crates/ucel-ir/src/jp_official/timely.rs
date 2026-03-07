use super::fetch::JpOfficialAdapter;

pub fn timely_adapter() -> JpOfficialAdapter {
    JpOfficialAdapter::new("jp_tdnet_timely_html")
}
