use anyhow::Result;
use pirogue::machine::Vm;
use pirogue::compiler::Compiler;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() -> Result<()> {
    let mut vm = Vm::new();
    let mut rl = DefaultEditor::new()?;

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let mut compiler = Compiler::new(line.as_str());
                let bytcode = compiler.compile();
                if let Err(e) = vm.eval(&bytcode) {
                    println!("{}", e);
                }
                vm.print()?;
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }

    Ok(())
}
