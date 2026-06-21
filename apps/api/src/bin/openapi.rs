fn main() {
    let document = fosslate_api::openapi::document();
    serde_json::to_writer_pretty(std::io::stdout(), &document).expect("serialize OpenAPI document");
    println!();
}
