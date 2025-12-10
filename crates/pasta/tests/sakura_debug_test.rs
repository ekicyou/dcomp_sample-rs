//! Debug test for sakura script parsing

use pasta::{PastaEngine, ir::{ScriptEvent, ContentPart}};

#[test]
fn debug_sakura_parsing() -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
＊test
    さくら：こんにちは\w8お元気ですか
"#;

    let mut engine = PastaEngine::new(script)?;
    let events = engine.execute_label("test")?;

    println!("Total events: {}", events.len());
    for (i, event) in events.iter().enumerate() {
        println!("Event {}: {:?}", i, event);
    }

    Ok(())
}
