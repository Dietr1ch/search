use calliper::Runner;
use calliper::Scenario;
use calliper::utils::black_box;

#[inline(never)]
#[unsafe(no_mangle)]
fn binary_search_impl(haystack: &[u8], needle: u8) -> Result<usize, usize> {
    haystack.binary_search(&needle)
}
fn binary_search() {
    let range = (0..255).collect::<Vec<_>>();
    let _ = black_box(binary_search_impl(black_box(&range), black_box(253)));
}

#[inline(never)]
#[unsafe(no_mangle)]
fn linear_search_impl(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|n| *n == needle)
}

fn linear_search() {
    let range = (0..255).collect::<Vec<_>>();
    black_box(linear_search_impl(black_box(&range), black_box(253)));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let benches = [
        Scenario::new(linear_search), // Slow?
        Scenario::new(binary_search), // Fast?
    ];

    if let Some(results) = Runner::default().run(&benches)? {
        for res in results.into_iter() {
            println!("{}", res.parse());
        }
    }
    Ok(())
}
