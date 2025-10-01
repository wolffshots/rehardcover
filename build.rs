fn main() {
    // Only register schema if the file exists
    // This allows the build to work even before schema introspection
    if std::path::Path::new("schemas/hardcover.graphql").exists() {
        cynic_codegen::register_schema("hardcover")
            .from_sdl_file("schemas/hardcover.graphql")
            .expect("Failed to register hardcover schema")
            .as_default()
            .expect("Failed to set hardcover as default schema");
    } else {
        println!("cargo:warning=Schema file not found. Run schema introspection first.");
    }
}
