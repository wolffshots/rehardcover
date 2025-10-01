fn main() {
    cynic_codegen::register_schema("hardcover")
        .from_sdl_file("schemas/hardcover.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
