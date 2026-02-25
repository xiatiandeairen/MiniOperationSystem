use proc_macro::TokenStream;

/// Automatically wraps a function body in a trace span.
///
/// Usage: `#[traced(module = "memory")]`
#[proc_macro_attribute]
pub fn traced(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Stub: pass through unchanged until CP3 implements real logic
    item
}
