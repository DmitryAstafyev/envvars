use std::io::Error;

mod extractor;
mod injector;
mod paths;

fn main() -> Result<(), Error> {
    println!(">>>>>>>>>>>>>>>>>>>>>> BUILDING");
    extractor::copy_sources()?;
    extractor::build()?;
    injector::inject()?;
    // assets::write_extractor_file_name()?;
    // assets::write_extractor_checksum()?;
    println!(">>>>>>>>>>>>>>>>>>>>>> BUILT");
    Ok(())
}
