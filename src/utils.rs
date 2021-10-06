use std::{
    fs,
    io::{self, BufReader, Read},
};

#[macro_export]
macro_rules! otry {
    ($res:expr) => {
        match $res {
            Ok(__val) => __val,
            Err(err) => return Some(Err(From::from(err))),
        }
    };
}

pub fn read_token(file: &mut fs::File) -> Result<String, io::Error> {
    let mut buf = "Bearer ".to_string();
    file.read_to_string(&mut buf)?;
    // ignore trailing whitespace
    buf.truncate(buf.trim_end().len());
    Ok(buf)
}

pub fn read_to_vec(file: &mut fs::File) -> Result<Vec<u8>, io::Error> {
    let mut buf = Vec::new();
    BufReader::new(file).read_to_end(&mut buf)?;
    Ok(buf)
}
