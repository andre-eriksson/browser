/// A trait representing the ability to execute scripts within the browser context.
pub trait ScriptExecutor {
    /// Executes the given script in the context of the browser.
    fn execute_script(&self, script: &str);
}
