#[rocket::main]
async fn main() {
    rask_api::assemble_rocket()
        .launch()
        .await
        .expect("error starting server");
}
