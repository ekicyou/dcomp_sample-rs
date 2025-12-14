use rune::Context;

fn main() {
    let code = r#"
pub mod actors {
    pub const test = #{ name: "test" };
}

pub mod test_mod {
    use crate::actors::*;
    
    pub fn test_fn() {
        let a = test;
        a.name
    }
}

pub fn main() {
    test_mod::test_fn()
}
"#;

    let mut context = Context::with_default_modules().unwrap();
    let mut sources = rune::Sources::new();
    sources.insert(rune::Source::new("test", code).unwrap()).unwrap();
    
    match rune::prepare(&mut sources).with_context(&context).build() {
        Ok(_) => println!("Success!"),
        Err(e) => println!("Error: {}", e),
    }
}
