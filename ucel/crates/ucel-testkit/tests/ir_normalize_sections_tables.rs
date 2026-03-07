use ucel_ir::normalize::{sections::sections_from_text, tables::csv_to_table};

#[test]
fn section_and_table_stable() {
    let s = sections_from_text("# A\nbody");
    assert_eq!(s[0].ordinal, 0);
    let t = csv_to_table("h1,h2\n1,2");
    assert_eq!(t.headers.len(), 2);
    assert_eq!(t.rows[0][1], "2");
}
