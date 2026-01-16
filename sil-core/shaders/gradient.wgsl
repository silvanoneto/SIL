// ═══════════════════════════════════════════════════════════════════════
//  SIL GRADIENT COMPUTE SHADER
// ═══════════════════════════════════════════════════════════════════════
//
//  Calcula gradientes ∇ = (∂/∂ρ, ∂/∂θ) para batch de estados SIL.
//
//  Cada estado: 16 camadas × ByteSil(ρ: i4, θ: u4) = 128 bits = 16 bytes
//  Cada gradiente: 16 camadas × (∇ρ: f32, ∇θ: f32) = 128 floats = 512 bytes
//
// ═══════════════════════════════════════════════════════════════════════

// Constantes
const NUM_LAYERS: u32 = 16u;
const EPSILON: f32 = 0.001;
const TWO_PI: f32 = 6.283185307179586;

// Uniforms
struct Params {
    num_states: u32,
    epsilon: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0) var<storage, read> states: array<u32>;
@group(0) @binding(1) var<storage, read_write> gradients: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

// ─────────────────────────────────────────────────────────────────────────
//  Helpers
// ─────────────────────────────────────────────────────────────────────────

// Extrai ByteSil de um estado
// Cada estado = 4 × u32 = 16 bytes = 16 camadas
// Cada camada = 1 byte = (rho: i4, theta: u4)
fn extract_layer(state_idx: u32, layer_idx: u32) -> vec2<f32> {
    let word_idx = state_idx * 4u + layer_idx / 4u;
    let byte_offset = (layer_idx % 4u) * 8u;
    let byte_val = (states[word_idx] >> byte_offset) & 0xFFu;
    
    // rho: bits 4-7 (signed, -8 to 7)
    // theta: bits 0-3 (unsigned, 0 to 15)
    let rho_raw = i32((byte_val >> 4u) & 0xFu);
    let rho = select(rho_raw, rho_raw - 16, rho_raw > 7);
    let theta = f32(byte_val & 0xFu);
    
    return vec2<f32>(f32(rho), theta);
}

// Converte (ρ, θ) para número complexo
fn to_complex(rho_theta: vec2<f32>) -> vec2<f32> {
    let rho = rho_theta.x;
    let theta = rho_theta.y;
    
    // Magnitude: 2^(ρ/4), mas para ρ=-8 (NULL) retorna 0
    let magnitude = select(pow(2.0, rho / 4.0), 0.0, rho <= -8.0);
    
    // Fase: θ × (2π/16)
    let phase = theta * TWO_PI / 16.0;
    
    // z = magnitude × (cos(phase) + i×sin(phase))
    return vec2<f32>(
        magnitude * cos(phase),
        magnitude * sin(phase)
    );
}

// Calcula magnitude de número complexo
fn complex_magnitude(z: vec2<f32>) -> f32 {
    return sqrt(z.x * z.x + z.y * z.y);
}

// ─────────────────────────────────────────────────────────────────────────
//  Compute Gradient
// ─────────────────────────────────────────────────────────────────────────

@compute @workgroup_size(64)
fn compute_gradient(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let state_idx = global_id.x;
    
    if (state_idx >= params.num_states) {
        return;
    }
    
    let eps = params.epsilon;
    
    // Para cada camada, calcular gradiente
    for (var layer: u32 = 0u; layer < NUM_LAYERS; layer = layer + 1u) {
        let rho_theta = extract_layer(state_idx, layer);
        let rho = rho_theta.x;
        let theta = rho_theta.y;
        
        // ∂f/∂ρ via diferenças finitas centrais
        let z_rho_plus = to_complex(vec2<f32>(rho + eps, theta));
        let z_rho_minus = to_complex(vec2<f32>(rho - eps, theta));
        let grad_rho = (complex_magnitude(z_rho_plus) - complex_magnitude(z_rho_minus)) / (2.0 * eps);
        
        // ∂f/∂θ via diferenças finitas (circular mod 16)
        let theta_plus = (theta + eps) % 16.0;
        let theta_minus = select(theta - eps, theta - eps + 16.0, theta < eps);
        let z_theta_plus = to_complex(vec2<f32>(rho, theta_plus));
        let z_theta_minus = to_complex(vec2<f32>(rho, theta_minus));
        let grad_theta = (complex_magnitude(z_theta_plus) - complex_magnitude(z_theta_minus)) / (2.0 * eps);
        
        // Armazenar gradientes
        // Layout: [state0_layer0_rho, state0_layer0_theta, state0_layer1_rho, ...]
        let out_idx = state_idx * NUM_LAYERS * 2u + layer * 2u;
        gradients[out_idx] = grad_rho;
        gradients[out_idx + 1u] = grad_theta;
    }
}

// ─────────────────────────────────────────────────────────────────────────
//  Interpolate States
// ─────────────────────────────────────────────────────────────────────────

@compute @workgroup_size(64)
fn interpolate_states(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // TODO: Implementar interpolação entre estados
    // lerp e slerp no plano complexo
}

// ─────────────────────────────────────────────────────────────────────────
//  Jacobian Matrix
// ─────────────────────────────────────────────────────────────────────────

@compute @workgroup_size(64)
fn compute_jacobian(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // TODO: Implementar Jacobiano de transformações
    // J = [∂ρ'/∂ρ, ∂ρ'/∂θ; ∂θ'/∂ρ, ∂θ'/∂θ]
}
