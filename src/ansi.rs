pub fn strip(input: &[u8]) -> Vec<u8> {
    strip_ansi_escapes::strip(input)
}
