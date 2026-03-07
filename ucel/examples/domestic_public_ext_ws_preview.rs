use ucel_core::vendor_public_ws_operation_specs;

fn main() {
    for spec in vendor_public_ws_operation_specs() {
        println!(
            "venue={} op={} category={:?} schema={}.{}.{} payload={:?} ready={:?} integrity={:?} resume={:?}",
            spec.venue,
            spec.operation_id,
            spec.category,
            spec.schema_version.major,
            spec.schema_version.minor,
            spec.schema_version.patch,
            spec.payload_type,
            spec.readiness_mode,
            spec.integrity_mode,
            spec.resume_mode
        );
    }
}
