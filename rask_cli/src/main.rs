use rask_lib::models;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::get("http://localhost:8001/task/1")?.json::<models::Task>()?;
    println!("{:#?}", resp);
    Ok(())
}
