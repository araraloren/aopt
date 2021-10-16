use getopt_rs_help::{
    printer::Printer,
    store::{OptStore, PosStore},
    AppHelp,
};

fn main() {
    let mut app = AppHelp::default();

    app.set_name("snippet".to_owned());

    {
        let store = &mut app.store;

        let global = store.get_global_mut();

        global.add_opt(OptStore::new(
            "tempfile".to_owned(),
            "-t|--temp=s".to_owned(),
            "Set tempoary file name".to_owned(),
            true,
        ));
        global.add_opt(OptStore::new(
            "tool".to_owned(),
            "--tool=s".to_owned(),
            "Set fetch tool name".to_owned(),
            true,
        ));
        global.add_opt(OptStore::new(
            "encoding".to_owned(),
            "-e|--encoding=s".to_owned(),
            "Set webpage encoding".to_owned(),
            true,
        ));
        global.add_opt(OptStore::new(
            "beg".to_owned(),
            "-b|--beg-index=i".to_owned(),
            "Set begin index".to_owned(),
            true,
        ));
        global.add_opt(OptStore::new(
            "end".to_owned(),
            "-e|--end-index=i".to_owned(),
            "Set end index".to_owned(),
            true,
        ));
        global.add_opt(OptStore::new(
            "type".to_owned(),
            "--type=s".to_owned(),
            "Set webpage type".to_owned(),
            true,
        ));
        global.add_opt(OptStore::new(
            "output".to_owned(),
            "-o|--output=s".to_owned(),
            "Set output directory".to_owned(),
            true,
        ));
        global.add_opt(OptStore::new(
            "extension".to_owned(),
            "-e|--extension=s".to_owned(),
            "Set output file extension".to_owned(),
            true,
        ));
        global.add_pos(PosStore::new(
            "url".to_owned(),
            "<url>".to_owned(),
            "Set webpage url".to_owned(),
            "@0".to_owned(),
            false,
        ));
        global.set_footer(String::from("Here is the footer of global help!"));
        global.set_header(String::from("Here is the header of global help!"));
    }

    {
        let mut store = app.store.new_cmd("c".to_owned());

        store.set_hint("c".to_owned());
        store.set_help("Compile and run c code".to_owned());
        store.set_header(String::from("Here is cmd header for c"));
        store.set_footer(String::from("Here is cmd footer for c"));

        store.add_pos(PosStore::new(
            "file".to_owned(),
            "<file>".to_owned(),
            "the c source file path".to_owned(),
            "@0".to_owned(),
            false,
        ));
        store.add_opt(OptStore::new(
            "O".to_owned(),
            "-O|--optimize=i".to_owned(),
            "Set optimization level".to_owned(),
            true,
        ));
        store.add_opt(OptStore::new(
            "L".to_owned(),
            "-L|--link=s".to_owned(),
            "Add link library".to_owned(),
            true,
        ));
        store.add_opt(OptStore::new(
            "S".to_owned(),
            "-S=b".to_owned(),
            "Show assembly output".to_owned(),
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_cmd("cpp".to_owned());

        store.set_hint("cpp".to_owned());
        store.set_help("Compile and run cpp code".to_owned());
        store.set_header(String::from("Here is cmd header for cpp"));
        store.set_footer(String::from("Here is cmd footer for cpp"));

        store.add_pos(PosStore::new(
            "file".to_owned(),
            "<file>".to_owned(),
            "the cpp source file path".to_owned(),
            "@0".to_owned(),
            false,
        ));
        store.add_opt(OptStore::new(
            "O".to_owned(),
            "-O|--optimize=i".to_owned(),
            "Set optimization level".to_owned(),
            true,
        ));
        store.add_opt(OptStore::new(
            "L".to_owned(),
            "-L|--link=s".to_owned(),
            "Add link library".to_owned(),
            true,
        ));
        store.add_opt(OptStore::new(
            "S".to_owned(),
            "-S=b".to_owned(),
            "Show assembly output".to_owned(),
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_cmd("java".to_owned());

        store.set_hint("java".to_owned());
        store.set_help("Compile and run java code".to_owned());
        store.set_header(String::from("Here is cmd header for java"));
        store.set_footer(String::from("Here is cmd footer for java"));

        store.add_pos(PosStore::new(
            "file".to_owned(),
            "<file>".to_owned(),
            "the java source file path".to_owned(),
            "@0".to_owned(),
            false,
        ));
        store.add_opt(OptStore::new(
            "O".to_owned(),
            "-O|--optimize=i".to_owned(),
            "Set optimization level".to_owned(),
            true,
        ));
        store.add_opt(OptStore::new(
            "L".to_owned(),
            "-L|--link=s".to_owned(),
            "Add link library".to_owned(),
            true,
        ));
        store.add_opt(OptStore::new(
            "S".to_owned(),
            "-S=b".to_owned(),
            "Show assembly output".to_owned(),
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_cmd("py".to_owned());

        store.set_hint("py".to_owned());
        store.set_help("Run python code".to_owned());
        store.set_header(String::from("Here is cmd header for python"));
        store.set_footer(String::from("Here is cmd footer for python"));

        store.add_pos(PosStore::new(
            "file".to_owned(),
            "<file>".to_owned(),
            "the python source file path".to_owned(),
            "@0".to_owned(),
            false,
        ));
        store.add_opt(OptStore::new(
            "O".to_owned(),
            "-O|--optimize=i".to_owned(),
            "Set optimization level".to_owned(),
            true,
        ));
        store.add_opt(OptStore::new(
            "L".to_owned(),
            "-L|--link=s".to_owned(),
            "Add link library".to_owned(),
            true,
        ));
        store.add_opt(OptStore::new(
            "S".to_owned(),
            "-S=b".to_owned(),
            "Show assembly output".to_owned(),
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_cmd("perl".to_owned());

        store.set_hint("perl".to_owned());
        store.set_help("Run perl code".to_owned());
        store.set_header(String::from("Here is cmd header for perl"));
        store.set_footer(String::from("Here is cmd footer for perl"));

        store.add_pos(PosStore::new(
            "file".to_owned(),
            "<file>".to_owned(),
            "the perl source file path".to_owned(),
            "@0".to_owned(),
            false,
        ));
        store.add_opt(OptStore::new(
            "O".to_owned(),
            "-O|--optimize=i".to_owned(),
            "Set optimization level".to_owned(),
            true,
        ));
        store.add_opt(OptStore::new(
            "L".to_owned(),
            "-L|--link=s".to_owned(),
            "Add link library".to_owned(),
            true,
        ));
        store.add_opt(OptStore::new(
            "S".to_owned(),
            "-S=b".to_owned(),
            "Show assembly output".to_owned(),
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_sec("compile".to_owned());

        store.set_help(String::from("The language will compile and run:"));
        store.attach_cmd("c".to_owned());
        store.attach_cmd("cpp".to_owned());
        store.attach_cmd("java".to_owned());
        store.commit();
    }

    {
        let mut store = app.store.new_sec("interpret".to_owned());

        store.set_help(String::from("The language will run with interpreter:"));
        store.attach_cmd("perl".to_owned());
        store.attach_cmd("py".to_owned());
        store.commit();
    }

    dbg!(&app);

    app.print_help().unwrap();
}
