// sqlx::migrate! embeds the folder at compile time; without this a new migration file alone does not rebuild the crate.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
