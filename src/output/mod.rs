
extern crate uuid;
extern crate itertools;

pub mod client;
pub mod structs;

use std::io;
use parser::ast::*;

use self::client::{ClientOutput, Result};

#[allow(dead_code)]
pub fn write_client_html<'input>(w: &mut io::Write, template: &'input Template) -> Result {
    let output = ClientOutput::from_template(template);
    output.write_html(w)
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::fs;
    use std::path::Path;

    #[test]
    pub fn test_output1() {
        let template = ::parser::parse_file(Path::new("./res/tests/test1.ism")).ok().unwrap();
        let stdout = io::stdout();
        let mut stream = stdout.lock();

        assert!(super::write_client_html(&mut stream, &template).is_ok());

        let mut file = fs::File::create(Path::new("./output/test_output1.html")).ok().unwrap();
        assert!(super::write_client_html(&mut file, &template).is_ok());
    }

    #[test]
    pub fn test_output2() {
        let template = ::parser::parse_file(Path::new("./res/tests/test2.ism")).ok().unwrap();
        let stdout = io::stdout();
        let mut stream = stdout.lock();

        assert!(super::write_client_html(&mut stream, &template).is_ok());

        let mut file = fs::File::create(Path::new("./output/test_output2.html")).ok().unwrap();
        assert!(super::write_client_html(&mut file, &template).is_ok());
    }

    #[test]
    pub fn test_output3() {
        let template = ::parser::parse_file(Path::new("./res/tests/test3.ism")).ok().unwrap();
        let stdout = io::stdout();
        let mut stream = stdout.lock();

        assert!(super::write_client_html(&mut stream, &template).is_ok());

        let mut file = fs::File::create(Path::new("./output/test_output3.html")).ok().unwrap();
        assert!(super::write_client_html(&mut file, &template).is_ok());
    }
}
