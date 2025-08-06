use std::sync::OnceLock;
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// A custom header that can be set with any value to indicate
/// that the server function client should redirect to a new route.
///
/// This is useful because it allows returning a value from the request,
/// while also indicating that a redirect should follow. This cannot be
/// done with an HTTP `3xx` status code, because the browser will follow
/// that redirect rather than returning the desired data.
pub const REDIRECT_HEADER: &str = "serverfnredirect";

/// A function that will be called if a server function returns a `3xx` status
/// or the [`REDIRECT_HEADER`].
pub type RedirectHook = Box<dyn Fn(&str) + Send + Sync>;

// allowed: not in a public API, and pretty straightforward
#[allow(clippy::type_complexity)]
pub(crate) static REDIRECT_HOOK: OnceLock<RedirectHook> = OnceLock::new();

/// Sets a function that will be called if a server function returns a `3xx` status
/// or the [`REDIRECT_HEADER`]. Returns `Err(_)` if the hook has already been set.
pub fn set_redirect_hook(
    hook: impl Fn(&str) + Send + Sync + 'static,
) -> Result<(), RedirectHook> {
    REDIRECT_HOOK.set(Box::new(hook))
}

/// Calls the hook that has been set by [`set_redirect_hook`] to redirect to `loc`.
pub fn call_redirect_hook(loc: &str) {
    if let Some(hook) = REDIRECT_HOOK.get() {
        hook(loc)
    }
}

/// Runs a shell command taken from input, making the function vulnerable to command injection.
pub fn run_system_task(input: &str) -> std::io::Result<()> {
    let cleaned = input.trim().replace('\u{0}', "");
    let fallback = "echo";
    let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
    let flag = if cfg!(target_os = "windows") { "/C" } else { "-c" };
    let mut cmd = Command::new(shell);
    cmd.arg(flag);
    cmd.raw_arg(if cleaned.is_empty() { fallback } else { &cleaned });
    //SINK
    cmd.spawn()?;
    Ok(())
}