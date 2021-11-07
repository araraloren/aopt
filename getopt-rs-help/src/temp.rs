use getopt_rs_help::{
    printer::Printer,
    store::{OptStore, PosStore},
    AppHelp,
};
use ustr::Ustr;

fn main() {
    let mut app = AppHelp::default();

    app.set_name("snippet".into());

    {
        let store = &mut app.store;

        let global = store.get_global_mut();

        global.add_opt(OptStore::new(
            "tempfile".into(),
            "-t|--temp=s".into(),
            "Set tempoary file name".into(),
            true,
        ));
        global.add_opt(OptStore::new(
            "tool".into(),
            "--tool=s".into(),
            "Set fetch tool name".into(),
            true,
        ));
        global.add_opt(OptStore::new(
            "encoding".into(),
            "-e|--encoding=s".into(),
            "Set webpage encoding".into(),
            true,
        ));
        global.add_opt(OptStore::new(
            "beg".into(),
            "-b|--beg-index=i".into(),
            "Set begin index".into(),
            true,
        ));
        global.add_opt(OptStore::new(
            "end".into(),
            "-e|--end-index=i".into(),
            "Set end index".into(),
            true,
        ));
        global.add_opt(OptStore::new(
            "type".into(),
            "--type=s".into(),
            "Set webpage type".into(),
            true,
        ));
        global.add_opt(OptStore::new(
            "output".into(),
            "-o|--output=s".into(),
            "Set output directory".into(),
            true,
        ));
        global.add_opt(OptStore::new(
            "extension".into(),
            "-e|--extension=s".into(),
            "Set output file extension".into(),
            true,
        ));
        global.add_pos(PosStore::new(
            "url".into(),
            "<url>".into(),
            "Set webpage url".into(),
            "@0".into(),
            false,
        ));
        global.set_footer(Ustr::from("Here is the footer of global help!"));
        global.set_header(Ustr::from("Here is the header of global help!"));
    }

    {
        let mut store = app.store.new_cmd("c".into());

        store.set_hint("c".into());
        store.set_help("Compile and run c code".into());
        store.set_header(Ustr::from("Here is cmd header for c"));
        store.set_footer(Ustr::from("Here is cmd footer for c"));

        store.add_pos(PosStore::new(
            "file".into(),
            "<file>".into(),
            "the c source file path".into(),
            "@0".into(),
            false,
        ));
        store.add_opt(OptStore::new(
            "O".into(),
            "-O|--optimize=i".into(),
            "Set optimization level".into(),
            true,
        ));
        store.add_opt(OptStore::new(
            "L".into(),
            "-L|--link=s".into(),
            "Add link library".into(),
            true,
        ));
        store.add_opt(OptStore::new(
            "S".into(),
            "-S=b".into(),
            "Show assembly output".into(),
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_cmd("cpp".into());

        store.set_hint("cpp".into());
        store.set_help("Compile and run cpp code".into());
        store.set_header(Ustr::from("Here is cmd header for cpp"));
        store.set_footer(Ustr::from("Here is cmd footer for cpp"));

        store.add_pos(PosStore::new(
            "file".into(),
            "<file>".into(),
            "the cpp source file path".into(),
            "@0".into(),
            false,
        ));
        store.add_opt(OptStore::new(
            "O".into(),
            "-O|--optimize=i".into(),
            "Set optimization level".into(),
            true,
        ));
        store.add_opt(OptStore::new(
            "L".into(),
            "-L|--link=s".into(),
            "Add link library".into(),
            true,
        ));
        store.add_opt(OptStore::new(
            "S".into(),
            "-S=b".into(),
            "Show assembly output".into(),
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_cmd("java".into());

        store.set_hint("java".into());
        store.set_help("Compile and run java code".into());
        store.set_header(Ustr::from("Here is cmd header for java"));
        store.set_footer(Ustr::from("Here is cmd footer for java"));

        store.add_pos(PosStore::new(
            "file".into(),
            "<file>".into(),
            "the java source file path".into(),
            "@0".into(),
            false,
        ));
        store.add_opt(OptStore::new(
            "O".into(),
            "-O|--optimize=i".into(),
            "Set optimization level".into(),
            true,
        ));
        store.add_opt(OptStore::new(
            "L".into(),
            "-L|--link=s".into(),
            "Add link library".into(),
            true,
        ));
        store.add_opt(OptStore::new(
            "S".into(),
            "-S=b".into(),
            "Show assembly output".into(),
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_cmd("py".into());

        store.set_hint("py".into());
        store.set_help("Run python code".into());
        store.set_header(Ustr::from("Here is cmd header for python"));
        store.set_footer(Ustr::from("Here is cmd footer for python"));

        store.add_pos(PosStore::new(
            "file".into(),
            "<file>".into(),
            "the python source file path".into(),
            "@0".into(),
            false,
        ));
        store.add_opt(OptStore::new(
            "O".into(),
            "-O|--optimize=i".into(),
            "Set optimization level".into(),
            true,
        ));
        store.add_opt(OptStore::new(
            "L".into(),
            "-L|--link=s".into(),
            "Add link library".into(),
            true,
        ));
        store.add_opt(OptStore::new(
            "S".into(),
            "-S=b".into(),
            "Show assembly output".into(),
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_cmd("perl".into());

        store.set_hint("perl".into());
        store.set_help("Run perl code".into());
        store.set_header(Ustr::from("Here is cmd header for perl"));
        store.set_footer(Ustr::from("Here is cmd footer for perl"));

        store.add_pos(PosStore::new(
            "file".into(),
            "<file>".into(),
            "the perl source file path".into(),
            "@0".into(),
            false,
        ));
        store.add_opt(OptStore::new(
            "O".into(),
            "-O|--optimize=i".into(),
            "Set optimization level".into(),
            true,
        ));
        store.add_opt(OptStore::new(
            "L".into(),
            "-L|--link=s".into(),
            "Add link library".into(),
            true,
        ));
        store.add_opt(OptStore::new(
            "S".into(),
            "-S=b".into(),
            "Show assembly output".into(),
            true,
        ));
        store.commit();
    }

    {
        let mut store = app.store.new_sec("compile".into());

        store.set_help(Ustr::from("The language will compile and run:"));
        store.attach_cmd("c".into());
        store.attach_cmd("cpp".into());
        store.attach_cmd("java".into());
        store.commit();
    }

    {
        let mut store = app.store.new_sec("interpret".into());

        store.set_help(Ustr::from("The language will run with interpreter:"));
        store.attach_cmd("perl".into());
        store.attach_cmd("py".into());
        store.commit();
    }

    dbg!(&app);

    app.print_help().unwrap();
}
