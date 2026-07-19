//! Offline tool: consolidate `packs/` into runtime `data/` bundles.

fn main() {
    match pathfinder_viewer::pipeline::run() {
        Ok(()) => println!("Data written to {}", pathfinder_viewer::pipeline::output_dir().display()),
        Err(e) => {
            eprintln!("build_data failed: {e}");
            std::process::exit(1);
        }
    }
}
