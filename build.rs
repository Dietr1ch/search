use shadow_rs::BuildPattern;
use shadow_rs::ShadowBuilder;

fn main() -> std::io::Result<()> {
    ShadowBuilder::builder()
        .build_pattern(BuildPattern::Lazy)
        .deny_const(Default::default())
        .build()
        .unwrap();

    Ok(())
}
