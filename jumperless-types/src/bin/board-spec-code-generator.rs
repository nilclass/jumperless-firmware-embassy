use jumperless_types::board_spec::generator;

fn main() -> Result<(), generator::Error> {
    let path = std::env::args().nth(1).expect("path to board spec");
    let output = std::env::args().nth(2).expect("path to code output");
    let board_spec = generator::parse(&path)?;
    println!("Loaded {} ✔", path);
    println!("Board spec summary:");
    println!("  {} Nodes", board_spec.nodes.len());
    println!("  {} Node ports", board_spec.node_ports.len());
    println!("  {} Lanes", board_spec.lanes.len());
    println!("  {} Bounce ports", board_spec.bounce_ports.len());
    generator::sanity_check(&board_spec);
    println!("Sanity check passed ✔");
    let header = r#"
        //  _____   ____    _   _  ____ _______   ______ _____ _____ _______
        // |  __ \ / __ \  | \ | |/ __ \__   __| |  ____|  __ \_   _|__   __|
        // | |  | | |  | | |  \| | |  | | | |    | |__  | |  | || |    | |
        // | |  | | |  | | | . ` | |  | | | |    |  __| | |  | || |    | |
        // | |__| | |__| | | |\  | |__| | | |    | |____| |__| || |_   | |
        // |_____/ \____/  |_| \_|\____/  |_|    |______|_____/_____|  |_|
        //    _______ _    _ _____  _____   ______ _____ _      ______
        //   |__   __| |  | |_   _|/ ____| |  ____|_   _| |    |  ____|
        //      | |  | |__| | | | | (___   | |__    | | | |    | |__
        //      | |  |  __  | | |  \___ \  |  __|   | | | |    |  __|
        //      | |  | |  | |_| |_ ____) | | |     _| |_| |____| |____
        //      |_|  |_|  |_|_____|_____/  |_|    |_____|______|______|
        //
        // This file was auto-generated from a board spec definition.
    "#;

    let code = generator::generate_board_spec_code(&board_spec).to_string();

    std::fs::write(output, format!("{}\n{}", header, code)).unwrap();

    Ok(())
}
