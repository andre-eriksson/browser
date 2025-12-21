pub trait StyleHandler {
    fn process_css(&mut self, css_char: char);
}

pub trait ScriptHandler {
    fn process_js(&mut self, js_char: char);
}
