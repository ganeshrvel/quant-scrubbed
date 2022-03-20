pub fn trim_newline(value: String) -> String {
    let mut s = value;

    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }

    s
}
