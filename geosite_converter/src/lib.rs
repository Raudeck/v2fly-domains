use std::ffi::c_char;

extern "C" {
    pub fn v2site_to_sing();
    pub fn generate(geositeInputFile: *const c_char, ruleSetOutputDir: *const c_char);
}