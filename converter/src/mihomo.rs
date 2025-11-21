use std::{
    error::Error,
    ffi::{c_char, c_int},
    fmt::Display,
};

#[link(name = "converter")]
unsafe extern "C" {
    fn mihomoCompileRuleset(
        dat: *const i8,
        len: c_int,
        outputPath: *const c_char,
        outputLen: c_int,
    ) -> *mut c_char;
}

pub struct Mihomo {}

#[derive(Debug)]
pub struct MihomoRulesetError(String);

impl Display for MihomoRulesetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Mihomo {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&self, buf: &[u8], output: &str) -> Result<(), MihomoRulesetError> {
        let len = buf.len();
        if len == 0 {
            return Ok(());
        }
        unsafe {
            let res = mihomoCompileRuleset(
                buf.as_ptr() as *const i8,
                len as i32,
                output.as_ptr() as *const c_char,
                output.len() as i32,
            );
            if !res.is_null() {
                println!("{:?}", String::from_utf8_lossy(buf));
                return Err(MihomoRulesetError(res.read().to_string()));
            }
        }
        Ok(())
    }
}
