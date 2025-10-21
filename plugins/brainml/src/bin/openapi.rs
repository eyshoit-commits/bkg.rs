fn main() {
    let doc = brainml::api::openapi::BrainmlApiDoc::openapi();
    println!("{}", doc.to_json().unwrap());
}
