use aopt_help::{AppHelp, DefaultFormat, OptStore, PosStore, Printer};

fn main() {
    let mut app = AppHelp::<std::io::Stdout, DefaultFormat>::default();

    app.set_name("snippet");

    {
        let store = &mut app.store;

        let global = store.get_global_mut();

        global.add_opt(OptStore::new(
            "tempfile",
            "-t|--temp",
            "str",
            "Set tempoary file name",
            true,
        ));
        global.add_opt(OptStore::new(
            "tool",
            "--tool",
            "str",
            "Set fetch tool name",
            true,
        ));
        global.add_opt(OptStore::new(
            "encoding",
            "-e|--encoding",
            "str",
            "Set webpage encoding",
            true,
        ));
        global.add_opt(OptStore::new(
            "beg",
            "-b|--beg-index",
            "int",
            "Set begin index",
            true,
        ));
        global.add_opt(OptStore::new(
            "end",
            "-e|--end-index",
            "int",
            "Set end index",
            true,
        ));
        global.add_opt(OptStore::new(
            "type",
            "--type",
            "str",
            "Set webpage type",
            true,
        ));
        global.add_opt(OptStore::new(
            "output",
            "-o|--output",
            "str",
            "Set output directory",
            true,
        ));
        global.add_opt(OptStore::new(
            "extension",
            "-e|--extension",
            "str",
            "Set output file extension",
            true,
        ));
        global.add_pos(PosStore::new(
            "url",
            "<url>",
            "Set webpage url",
            "@0",
            false,
        ));
        global.set_footer("Here is the footer of global help!");
        global.set_header("Here is the header of global help!");
    }

    {
        let mut store = app.store.new_cmd("c");

        store.set_hint("c");
        store.set_help("Compile and run c code");
        store.set_header("Here is cmd header for c");
        store.set_footer("Here is cmd footer for c");

        store.add_pos(PosStore::new(
            "file",
            "<file>",
            "the c source file path",
            "@0",
            false,
        ));
        store.add_opt(OptStore::new(
            "O",
            "-O|--optimize",
            "int",
            "Set optimization level",
            true,
        ));
        store.add_opt(OptStore::new(
            "L",
            "-L|--link",
            "str",
            "Add link library",
            true,
        ));
        store.add_opt(OptStore::new(
            "S",
            "-S",
            "bool",
            "Show assembly output",
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_cmd("cpp");

        store.set_hint("cpp");
        store.set_help("Compile and run cpp code");
        store.set_header("Here is cmd header for cpp");
        store.set_footer("Here is cmd footer for cpp");

        store.add_pos(PosStore::new(
            "file",
            "<file>",
            "the cpp source file path",
            "@0",
            false,
        ));
        store.add_opt(OptStore::new(
            "O",
            "-O|--optimize",
            "int",
            "Set optimization level",
            true,
        ));
        store.add_opt(OptStore::new(
            "L",
            "-L|--link",
            "str",
            "Add link library",
            true,
        ));
        store.add_opt(OptStore::new(
            "S",
            "-S",
            "bool",
            "Show assembly output",
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_cmd("java");

        store.set_hint("java");
        store.set_help("Compile and run java code");
        store.set_header("Here is cmd header for java");
        store.set_footer("Here is cmd footer for java");

        store.add_pos(PosStore::new(
            "file",
            "<file>",
            "the java source file path",
            "@0",
            false,
        ));
        store.add_opt(OptStore::new(
            "O",
            "-O|--optimize",
            "int",
            "Set optimization level",
            true,
        ));
        store.add_opt(OptStore::new(
            "L",
            "-L|--link",
            "str",
            "Add link library",
            true,
        ));
        store.add_opt(OptStore::new(
            "S",
            "-S",
            "bool",
            "Show assembly output",
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_cmd("py");

        store.set_hint("py");
        store.set_help("Run python code");
        store.set_header("Here is cmd header for python");
        store.set_footer("Here is cmd footer for python");

        store.add_pos(PosStore::new(
            "file",
            "<file>",
            "the python source file path",
            "@0",
            false,
        ));
        store.add_opt(OptStore::new(
            "O",
            "-O|--optimize",
            "int",
            "Set optimization level",
            true,
        ));
        store.add_opt(OptStore::new(
            "L",
            "-L|--link",
            "str",
            "Add link library",
            true,
        ));
        store.add_opt(OptStore::new(
            "S",
            "-S",
            "bool",
            "Show assembly output",
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_cmd("perl");

        store.set_hint("perl");
        store.set_help("Run perl code");
        store.set_header("Here is cmd header for perl");
        store.set_footer("Here is cmd footer for perl");

        store.add_pos(PosStore::new(
            "file",
            "<file>",
            "the perl source file path",
            "@0",
            false,
        ));
        store.add_opt(OptStore::new(
            "O",
            "-O|--optimize",
            "int",
            "Set optimization level",
            true,
        ));
        store.add_opt(OptStore::new(
            "L",
            "-L|--link",
            "str",
            "Add link library",
            true,
        ));
        store.add_opt(OptStore::new(
            "S",
            "-S",
            "bool",
            "Show assembly output",
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_sec("compile");

        store.set_help("The language will compile and run:");
        store.attach_cmd("c");
        store.attach_cmd("cpp");
        store.attach_cmd("java");
        store.commit();
    }

    {
        let mut store = app.store.new_sec("interpret");

        store.set_help("The language will run with interpreter:");
        store.attach_cmd("perl");
        store.attach_cmd("py");
        store.commit();
    }

    dbg!(&app);

    println!("help ---------------> ");
    app.print_help().unwrap();
    println!("help ---------------> ");

    println!("help of golbal ---------------> ");
    app.print_cmd_help(None).unwrap();
    println!("help of golbal ---------------> ");

    println!("help of cmd perl ---------------> ");
    app.print_cmd_help(Some("perl")).unwrap();
    println!("help of cmd perl ---------------> ");

    println!("help of cmd py ---------------> ");
    app.print_cmd_help(Some("py")).unwrap();
    println!("help of cmd py ---------------> ");

    println!("help of cmd java ---------------> ");
    app.print_cmd_help(Some("java")).unwrap();
    println!("help of cmd java ---------------> ");
}
