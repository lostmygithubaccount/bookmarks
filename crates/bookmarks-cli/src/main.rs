fn main() -> anyhow::Result<()> {
    bookmarks::run_cli(std::env::args())
}
