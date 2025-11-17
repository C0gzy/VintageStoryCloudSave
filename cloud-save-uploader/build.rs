// Build script to embed environment variables at compile time
// This allows embedding secrets during CI/CD builds without exposing them in the repo

fn main() {
    // Embed environment variables at compile time if they exist
    // These will be available via env!() macro in the code
    
    if let Ok(val) = std::env::var("B2_KEY_ID") {
        println!("cargo:rustc-env=B2_KEY_ID={}", val);
    }
    
    if let Ok(val) = std::env::var("B2_APPLICATION_KEY") {
        println!("cargo:rustc-env=B2_APPLICATION_KEY={}", val);
    }
    
    if let Ok(val) = std::env::var("B2_BUCKET") {
        println!("cargo:rustc-env=B2_BUCKET={}", val);
    }
    
    if let Ok(val) = std::env::var("B2_REGION") {
        println!("cargo:rustc-env=B2_REGION={}", val);
    }
    
    if let Ok(val) = std::env::var("B2_ENDPOINT") {
        println!("cargo:rustc-env=B2_ENDPOINT={}", val);
    }
    
    // Rebuild if build script changes
    println!("cargo:rerun-if-changed=build.rs");
}

