fn main() {
    if std::env::var("CI").is_err() {
        let _ = std::process::Command::new("tailwindcss")
            .arg("-i")
            .arg("../public/styles/input.css")
            .arg("-o")
            .arg("styles/output.css")
            .arg("-c")
            .arg("../tailwind.config.cjs")
            .output()
            .expect("Failed to run tailwindcss command");
    }
}

