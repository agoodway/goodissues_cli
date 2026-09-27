/// Match the Zig CLI's flag/value and positional conventions.
pub struct Args<'a>(pub &'a [String]);
impl<'a> Args<'a> {
    pub fn flag(&self, name: &str) -> Option<&'a str> {
        for (i, arg) in self.0.iter().enumerate() {
            if arg == name {
                if let Some(value) = self.0.get(i + 1) {
                    return Some(value);
                }
            } else if let Some(value) = arg.strip_prefix(name).and_then(|s| s.strip_prefix('=')) {
                return Some(value);
            }
        }
        None
    }
    pub fn has(&self, name: &str) -> bool {
        self.0.iter().any(|arg| arg == name)
    }
    pub fn positional(&self, index: usize) -> Option<&'a str> {
        let mut args = self.0.iter();
        let mut count = 0;
        while let Some(arg) = args.next() {
            if arg.starts_with("--") {
                if !arg.contains('=') && arg != "--json" {
                    args.next();
                }
                continue;
            }
            if arg.starts_with('-') && arg.len() > 1 {
                continue;
            }
            if count == index {
                return Some(arg);
            }
            count += 1;
        }
        None
    }
    pub fn required(&self, name: &str) -> Result<&'a str, String> {
        self.flag(name)
            .ok_or_else(|| format!("Error: {name} is required."))
    }
}
