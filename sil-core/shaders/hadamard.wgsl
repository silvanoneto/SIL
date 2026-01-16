// ═══════════════════════════════════════════════════════════════════════
//  SIL HADAMARD GATE COMPUTE SHADER
// ═══════════════════════════════════════════════════════════════════════
//
//  Aplica gate Hadamard em batch de estados quânticos.
//
//  Hadamard: H|0⟩ = (|0⟩ + |1⟩)/√2,  H|1⟩ = (|0⟩ - |1⟩)/√2
//  Matriz: [[1/√2, 1/√2], [1/√2, -1/√2]]
//
//  Cada estado: 2 amplitudes complexas (α, β) = 4 × f32 = 16 bytes
//  Layout: [α_re, α_im, β_re, β_im]
//
// ═══════════════════════════════════════════════════════════════════════

// Constantes
const SQRT2_INV: f32 = 0.7071067811865476;  // 1/√2
const NUM_COMPONENTS: u32 = 4u;              // 2 complex = 4 floats

// Uniforms
struct HadamardParams {
    num_states: u32,
    target_layer: u32,    // Camada SIL alvo (0-15), 0xFFFF = todas
    _padding: vec2<f32>,
}

@group(0) @binding(0) var<uniform> params: HadamardParams;
@group(0) @binding(1) var<storage, read> input_states: array<f32>;
@group(0) @binding(2) var<storage, read_write> output_states: array<f32>;

// ─────────────────────────────────────────────────────────────────────────
//  Helpers
// ─────────────────────────────────────────────────────────────────────────

// Multiplicação complexa: (a + bi)(c + di) = (ac - bd) + (ad + bc)i
fn complex_mul(a_re: f32, a_im: f32, b_re: f32, b_im: f32) -> vec2<f32> {
    return vec2<f32>(
        a_re * b_re - a_im * b_im,
        a_re * b_im + a_im * b_re
    );
}

// Adição complexa
fn complex_add(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return a + b;
}

// ─────────────────────────────────────────────────────────────────────────
//  Hadamard Transform
// ─────────────────────────────────────────────────────────────────────────
//
//  H = (1/√2) * [[1,  1],
//                [1, -1]]
//
//  |ψ'⟩ = H|ψ⟩
//  α' = (α + β) / √2
//  β' = (α - β) / √2
//

fn hadamard_single(alpha_re: f32, alpha_im: f32, beta_re: f32, beta_im: f32) -> array<f32, 4> {
    var result: array<f32, 4>;

    // α' = (α + β) * (1/√2)
    result[0] = (alpha_re + beta_re) * SQRT2_INV;
    result[1] = (alpha_im + beta_im) * SQRT2_INV;

    // β' = (α - β) * (1/√2)
    result[2] = (alpha_re - beta_re) * SQRT2_INV;
    result[3] = (alpha_im - beta_im) * SQRT2_INV;

    return result;
}

// ─────────────────────────────────────────────────────────────────────────
//  Main Compute Kernel
// ─────────────────────────────────────────────────────────────────────────

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let state_idx = gid.x;

    // Bounds check
    if (state_idx >= params.num_states) {
        return;
    }

    // Índice base no array (4 floats por estado)
    let base = state_idx * NUM_COMPONENTS;

    // Ler estado de entrada: |ψ⟩ = α|0⟩ + β|1⟩
    let alpha_re = input_states[base];
    let alpha_im = input_states[base + 1u];
    let beta_re = input_states[base + 2u];
    let beta_im = input_states[base + 3u];

    // Aplicar Hadamard
    let result = hadamard_single(alpha_re, alpha_im, beta_re, beta_im);

    // Escrever estado de saída
    output_states[base] = result[0];
    output_states[base + 1u] = result[1];
    output_states[base + 2u] = result[2];
    output_states[base + 3u] = result[3];
}

// ─────────────────────────────────────────────────────────────────────────
//  Variante: Hadamard em camada específica de SilState
// ─────────────────────────────────────────────────────────────────────────
//
//  Para integração com SilState de 16 camadas, cada camada pode ser
//  tratada como um qubit com amplitude derivada de ByteSil(ρ, θ).
//

@compute @workgroup_size(64)
fn hadamard_layer(@builtin(global_invocation_id) gid: vec3<u32>) {
    let state_idx = gid.x;

    if (state_idx >= params.num_states) {
        return;
    }

    // Para SilState: 16 camadas × 4 floats = 64 floats por estado
    let layers_per_state = 16u;
    let floats_per_layer = 4u;
    let state_size = layers_per_state * floats_per_layer;

    let target = params.target_layer;

    // Aplicar em todas as camadas ou apenas na target
    if (target == 0xFFFFu) {
        // Aplicar em todas as 16 camadas
        for (var layer = 0u; layer < layers_per_state; layer = layer + 1u) {
            let base = state_idx * state_size + layer * floats_per_layer;

            let alpha_re = input_states[base];
            let alpha_im = input_states[base + 1u];
            let beta_re = input_states[base + 2u];
            let beta_im = input_states[base + 3u];

            let result = hadamard_single(alpha_re, alpha_im, beta_re, beta_im);

            output_states[base] = result[0];
            output_states[base + 1u] = result[1];
            output_states[base + 2u] = result[2];
            output_states[base + 3u] = result[3];
        }
    } else {
        // Aplicar apenas na camada target
        let base = state_idx * state_size + target * floats_per_layer;

        let alpha_re = input_states[base];
        let alpha_im = input_states[base + 1u];
        let beta_re = input_states[base + 2u];
        let beta_im = input_states[base + 3u];

        let result = hadamard_single(alpha_re, alpha_im, beta_re, beta_im);

        output_states[base] = result[0];
        output_states[base + 1u] = result[1];
        output_states[base + 2u] = result[2];
        output_states[base + 3u] = result[3];

        // Copiar demais camadas inalteradas
        for (var layer = 0u; layer < layers_per_state; layer = layer + 1u) {
            if (layer != target) {
                let src = state_idx * state_size + layer * floats_per_layer;
                output_states[src] = input_states[src];
                output_states[src + 1u] = input_states[src + 1u];
                output_states[src + 2u] = input_states[src + 2u];
                output_states[src + 3u] = input_states[src + 3u];
            }
        }
    }
}
