fn main() {
    let doc = brainml::api::openapi::BrainmlApiDoc::openapi();
    match doc.to_json() {
        Ok(json) => println!("{}", json),
        Err(err) => {
            eprintln!("failed to render OpenAPI spec: {err}");
            std::process::exit(1);
        }
    }
}
