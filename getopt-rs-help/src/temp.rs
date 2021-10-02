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
            "[-t|--temp=s]".to_owned(),
            "Set tempoary file name".to_owned(),
            true,
        ));
        global.add_opt(OptStore::new(
            "tool".to_owned(),
            "[--tool=s]".to_owned(),
            "Set fetch tool name".to_owned(),
            true,
        ));
        global.add_opt(OptStore::new(
            "encoding".to_owned(),
            "[-e|--encoding=s]".to_owned(),
            "Set webpage encoding".to_owned(),
            true,
        ));
        global.add_opt(OptStore::new(
            "beg".to_owned(),
            "[-b|--beg-index=i]".to_owned(),
            "Set begin index".to_owned(),
            true,
        ));
        global.add_opt(OptStore::new(
            "end".to_owned(),
            "[-e|--end-index=i]".to_owned(),
            "Set end index".to_owned(),
            true,
        ));
        global.add_opt(OptStore::new(
            "type".to_owned(),
            "[--type=s]".to_owned(),
            "Set webpage type".to_owned(),
            true,
        ));
        global.add_opt(OptStore::new(
            "output".to_owned(),
            "[-o|--output=s]".to_owned(),
            "Set output directory".to_owned(),
            true,
        ));
        global.add_opt(OptStore::new(
            "extension".to_owned(),
            "[-e|--extension=s]".to_owned(),
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
    }

    dbg!(&app);

    app.print_usage();
}
