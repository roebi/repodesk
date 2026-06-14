fn main() {
    let repo_path = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_dir().ok());

    if let Err(e) = repodesk_ui::app::run(repo_path) {
        eprintln!("repodesk error: {}", e);
        std::process::exit(1);
    }
}
