#[macro_export]
macro_rules! otry {
    ($res:expr) => {
        match $res {
            Ok(__val) => __val,
            Err(err) => return Some(Err(From::from(err))),
        }
    };
}
