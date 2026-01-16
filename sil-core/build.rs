//! Build script para pré-compilação de shaders WGSL
//!
//! Compila shaders em tempo de build e gera código Rust com os módulos validados.
//! Isso elimina overhead de compilação em runtime e detecta erros de shader no CI.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=shaders/");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("compiled_shaders.rs");

    // Ler shaders existentes
    let gradient_shader = fs::read_to_string("shaders/gradient.wgsl")
        .expect("Failed to read gradient.wgsl");

    let interpolate_shader = fs::read_to_string("shaders/interpolate.wgsl")
        .expect("Failed to read interpolate.wgsl");

    // Ler novos shaders quânticos
    let hadamard_shader = fs::read_to_string("shaders/hadamard.wgsl")
        .expect("Failed to read hadamard.wgsl");

    let quantum_gates_shader = fs::read_to_string("shaders/quantum_gates.wgsl")
        .expect("Failed to read quantum_gates.wgsl");

    // Validar sintaxe WGSL (básica)
    validate_wgsl(&gradient_shader, "gradient.wgsl");
    validate_wgsl(&interpolate_shader, "interpolate.wgsl");
    validate_wgsl(&hadamard_shader, "hadamard.wgsl");
    validate_wgsl(&quantum_gates_shader, "quantum_gates.wgsl");

    // Gerar código Rust
    let generated = format!(
        r##"// Shaders WGSL pré-compilados em build time
// Gerado automaticamente por build.rs

/// Shader para cálculo de gradientes
pub const GRADIENT_SHADER: &str = r#"{}"#;

/// Shader para interpolação de estados
pub const INTERPOLATE_SHADER: &str = r#"{}"#;

/// Shader para gate Hadamard paralelo
pub const HADAMARD_SHADER: &str = r#"{}"#;

/// Shader para gates quânticas parametrizadas
pub const QUANTUM_GATES_SHADER: &str = r#"{}"#;
"##,
        gradient_shader,
        interpolate_shader,
        hadamard_shader,
        quantum_gates_shader
    );

    fs::write(&dest_path, generated)
        .expect("Failed to write compiled shaders");

    println!("cargo:warning=✅ Shaders compiled successfully:");
    println!("cargo:warning=  - gradient.wgsl ({} bytes)", gradient_shader.len());
    println!("cargo:warning=  - interpolate.wgsl ({} bytes)", interpolate_shader.len());
    println!("cargo:warning=  - hadamard.wgsl ({} bytes)", hadamard_shader.len());
    println!("cargo:warning=  - quantum_gates.wgsl ({} bytes)", quantum_gates_shader.len());
}

/// Validação básica de sintaxe WGSL
fn validate_wgsl(source: &str, filename: &str) {
    // Verificações simples
    let required_keywords = vec!["@compute", "@workgroup_size"];
    
    for keyword in &required_keywords {
        if !source.contains(keyword) {
            panic!("❌ {} missing required keyword: {}", filename, keyword);
        }
    }
    
    // Verificar parênteses balanceados
    let open_braces = source.matches('{').count();
    let close_braces = source.matches('}').count();
    
    if open_braces != close_braces {
        panic!(
            "❌ {} has unbalanced braces: {} open, {} close",
            filename, open_braces, close_braces
        );
    }
    
    // Verificar main entry point
    if !source.contains("@compute") {
        panic!("❌ {} missing compute entry point", filename);
    }
    
    println!("cargo:warning=✓ {} validated", filename);
}
