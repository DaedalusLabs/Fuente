use std::process::Command;

fn main() {
    // Step 0: skip if running on GitHub CI
    if std::env::var("CI").is_ok() {
        println!("cargo:warning=Skipping tailwindcss command on GitHub CI");
        return;
    }
     // Step 1: Run Tailwind CSS command
     let tailwind_output = Command::new("tailwindcss")
         .arg("-i")
         .arg("../public/styles/input.css")
         .arg("-o")
         .arg("./styles/output.css")
         .arg("-c")
         .arg("../tailwind.config.cjs")
         .output()
         .expect("Failed to run tailwindcss command");
    println!("Rebuilding the project...");

     if !tailwind_output.status.success() {
         panic!("Tailwind CSS command failed");
     }
}
