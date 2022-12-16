use aopt_help::prelude::*;
use aopt_help::Error;

fn main() -> Result<(), Error> {
    let mut app_help = AppHelp::<std::io::Stdout>::new(
        "snippet",
        "Here is the header of global help!",
        "Here is the footer of global help!",
        Style::default(),
        std::io::stdout(),
    );

    {
        let global = app_help.global_mut();

        global.push(Store::new(
            "tempfile",
            "-t|--temp",
            "Set tempoary file name",
            "str",
            true,
            false,
        ));
        global.push(Store::new(
            "tool",
            "--tool",
            "Set fetch tool name",
            "str",
            true,
            false,
        ));
        global.push(Store::new(
            "encoding",
            "-e|--encoding",
            "Set webpage encoding",
            "str",
            true,
            false,
        ));
        global.push(Store::new(
            "beg",
            "-b|--beg-index",
            "Set begin index",
            "int",
            true,
            false,
        ));
        global.push(Store::new(
            "end",
            "-e|--end-index",
            "Set end index",
            "int",
            true,
            false,
        ));
        global.push(Store::new(
            "type",
            "--type",
            "Set webpage type",
            "str",
            true,
            false,
        ));
        global.push(Store::new(
            "output",
            "-o|--output",
            "Set output directory",
            "str",
            true,
            false,
        ));
        global.push(Store::new(
            "extension",
            "-e|--extension",
            "Set output file extension",
            "str",
            true,
            false,
        ));
        global.push(Store::new(
            "url",
            "url",
            "Set webpage url",
            "NOA",
            false,
            true,
        ));
    }

    {
        app_help
            .new_block("compile")?
            .set_hint("compile")
            .set_head("The language will compile and run:");
        app_help
            .new_block("interpret")?
            .set_hint("interpret")
            .set_head("The language will run with interpreter:");
    }

    {
        let mut store = app_help.new_cmd("compile", "c")?;

        store.set_hint("c");
        store.set_help("Compile and run c code");
        store.set_head("Here is cmd header for c");
        store.set_foot("Here is cmd footer for c");

        let inner = store.inner();
        let mut block = inner.new_block("command")?;

        block
            .set_head("Commands:")
            .set_hint("[Commands]")
            .set_help("Commands block ");

        block
            .new_store("file")
            .set_hint("<file>")
            .set_help("the c source file path")
            .set_optional(false)
            .set_position(true)
            .set_type("NOA");
        block.submit();

        let mut block = inner.new_block("option")?;

        block
            .set_head("Options:")
            .set_hint("[Options]")
            .set_help("Option block ");

        block
            .new_store("O")
            .set_hint("-O|--optimize")
            .set_help("Set optimization level")
            .set_optional(true)
            .set_position(false)
            .set_type("int");

        block
            .new_store("L")
            .set_hint("-L|--link")
            .set_help("Add link library")
            .set_optional(true)
            .set_position(false)
            .set_type("str");

        block
            .new_store("S")
            .set_hint("-S")
            .set_help("Show assembly output")
            .set_optional(true)
            .set_position(false)
            .set_type("bool");
        block.submit();
        store.submit();
    }

    {
        let mut store = app_help.new_cmd("compile", "cpp")?;

        store.set_hint("cpp");
        store.set_help("Compile and run cpp code");
        store.set_head("Here is cmd header for cpp");
        store.set_foot("Here is cmd footer for cpp");

        let inner = store.inner();
        let mut block = inner.new_block("command")?;

        block
            .set_head("Commands:")
            .set_hint("[Commands]")
            .set_help("Command block ");

        block
            .new_store("file")
            .set_hint("<file>")
            .set_help("the cpp source file path")
            .set_optional(false)
            .set_position(true)
            .set_type("NOA");
        block.submit();

        let mut block = inner.new_block("option")?;

        block
            .set_head("Options:")
            .set_hint("[Options]")
            .set_help("Option block ");

        block
            .new_store("O")
            .set_hint("-O|--optimize")
            .set_help("Set optimization level")
            .set_optional(true)
            .set_position(false)
            .set_type("int");

        block
            .new_store("L")
            .set_hint("-L|--link")
            .set_help("Add link library")
            .set_optional(true)
            .set_position(false)
            .set_type("str");

        block
            .new_store("S")
            .set_hint("-S")
            .set_help("Show assembly output")
            .set_optional(true)
            .set_position(false)
            .set_type("bool");
        block.submit();
        store.submit();
    }

    {
        let mut store = app_help.new_cmd("compile", "java")?;

        store.set_hint("java");
        store.set_help("Compile and run java code");
        store.set_head("Here is cmd header for java");
        store.set_foot("Here is cmd footer for java");

        let inner = store.inner();

        let mut block = inner.new_block("command")?;

        block
            .set_head("Commands:")
            .set_hint("[Commands]")
            .set_help("Command block ");

        block
            .new_store("file")
            .set_hint("<file>")
            .set_help("the java source file path")
            .set_optional(false)
            .set_position(true)
            .set_type("NOA");
        block.submit();

        let mut block = inner.new_block("option")?;

        block
            .set_head("Options:")
            .set_hint("[Options]")
            .set_help("Option block ");

        block
            .new_store("O")
            .set_hint("-O|--optimize")
            .set_help("Set optimization level")
            .set_optional(true)
            .set_position(false)
            .set_type("int");

        block
            .new_store("L")
            .set_hint("-L|--link")
            .set_help("Add link library")
            .set_optional(true)
            .set_position(false)
            .set_type("str");

        block
            .new_store("S")
            .set_hint("-S")
            .set_help("Show assembly output")
            .set_optional(true)
            .set_position(false)
            .set_type("bool");
        block.submit();
        store.submit();
    }

    {
        let mut store = app_help.new_cmd("interpret", "py")?;

        store.set_hint("py");
        store.set_help("Run python code");
        store.set_head("Here is cmd header for python");
        store.set_foot("Here is cmd footer for python");

        let inner = store.inner();

        let mut block = inner.new_block("command")?;

        block
            .set_head("Commands:")
            .set_hint("[Commands]")
            .set_help("Command block ");

        block
            .new_store("file")
            .set_hint("<file>")
            .set_help("the python source file path")
            .set_optional(false)
            .set_position(true)
            .set_type("NOA");
        block.submit();

        let mut block = inner.new_block("option")?;

        block
            .set_head("Options:")
            .set_hint("[Options]")
            .set_help("Option block ");

        block
            .new_store("O")
            .set_hint("-O|--optimize")
            .set_help("Set optimization level")
            .set_optional(true)
            .set_position(false)
            .set_type("int");

        block
            .new_store("L")
            .set_hint("-L|--link")
            .set_help("Add link library")
            .set_optional(true)
            .set_position(false)
            .set_type("str");

        block
            .new_store("S")
            .set_hint("-S")
            .set_help("Show assembly output")
            .set_optional(true)
            .set_position(false)
            .set_type("bool");
        block.submit();
        store.submit();
    }

    {
        let mut store = app_help.new_cmd("interpret", "perl")?;

        store.set_hint("perl");
        store.set_help("Run perl code");
        store.set_head("Here is cmd header for perl");
        store.set_foot("Here is cmd footer for perl");

        let inner = store.inner();

        let mut block = inner.new_block("command")?;

        block
            .set_head("Commands:")
            .set_hint("[Commands]")
            .set_help("Command block ");

        block
            .new_store("file")
            .set_hint("<file>")
            .set_help("the perl source file path")
            .set_optional(false)
            .set_position(true)
            .set_type("NOA");
        block.submit();

        let mut block = inner.new_block("option")?;

        block
            .set_head("Options:")
            .set_hint("[Options]")
            .set_help("Option block ");

        block
            .new_store("O")
            .set_hint("-O|--optimize")
            .set_help("Set optimization level")
            .set_optional(true)
            .set_position(false)
            .set_type("int");

        block
            .new_store("L")
            .set_hint("-L|--link")
            .set_help("Add link library")
            .set_optional(true)
            .set_position(false)
            .set_type("str");

        block
            .new_store("S")
            .set_hint("-S")
            .set_help("Show assembly output")
            .set_optional(true)
            .set_position(false)
            .set_type("bool");
        block.submit();
        store.submit();
    }

    dbg!(&app_help);

    println!("help ---------------> ");
    app_help.display(true).unwrap();
    println!("help ---------------> ");

    println!("help of cmd cpp ---------------> ");
    app_help.display_cmd("cpp").unwrap();
    println!("help of cmd cpp ---------------> ");

    println!("help of cmd perl ---------------> ");
    app_help.display_cmd("perl").unwrap();
    println!("help of cmd perl ---------------> ");

    println!("help of cmd py ---------------> ");
    app_help.display_cmd("py").unwrap();
    println!("help of cmd py ---------------> ");

    println!("help of cmd java ---------------> ");
    app_help.display_cmd("java").unwrap();
    println!("help of cmd java ---------------> ");

    Ok(())
}
