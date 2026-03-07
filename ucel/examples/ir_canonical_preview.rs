use ucel_sdk::IrFacade;

fn main() {
    let facade = IrFacade;
    let support = facade.preview_ir_source_support().expect("ir support preview");
    println!("ir_canonical sources={}", support.len());
    for (id, policy) in support {
        println!("{id}: {:?}", policy);
    }
}
