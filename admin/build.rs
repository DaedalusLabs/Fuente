use std::process::Command;

fn main() {
    // Step 1: Run Tailwind CSS command
    let tailwind_output = Command::new("tailwindcss")
        .arg("-i")
        .arg("./public/styles/input.css")
        .arg("-o")
        .arg("./public/styles/output.css")
        .arg("-c")
        .arg("../tailwind.config.cjs")
        .output()
        .expect("Failed to run tailwindcss command");

    if !tailwind_output.status.success() {
        panic!("Tailwind CSS command failed");
    }
}

