pub fn write_file(path: std::path::PathBuf, contents: String) {
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;

    let path = Path::new(&path);
    let display = path.display();

    let folder = path.parent().unwrap();
    std::fs::create_dir_all(folder).unwrap();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(contents.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => {}
    }
}
