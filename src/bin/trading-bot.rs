fn main() -> Result<(), Box<std::error::Error>> {
    let f = std::fs::File::open("example.yaml")?;
    // let d: String = serde_yaml::from_reader(f)?;
    // println!("Read YAML string: {}", d);
    Ok(())
}
