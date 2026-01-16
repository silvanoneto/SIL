// ═══════════════════════════════════════════════════════════════════════
//  SIL QUANTUM GATES COMPUTE SHADER
// ═══════════════════════════════════════════════════════════════════════
//
//  Aplica gates quânticos parametrizados em batch de estados.
//
//  Gates suportadas:
//    0 = Hadamard (H)      4 = Rotation X (Rx)
//    1 = Pauli X           5 = Rotation Y (Ry)
//    2 = Pauli Y           6 = Rotation Z (Rz)
//    3 = Pauli Z           7 = Phase (P)
//    8 = S Gate            9 = T Gate
//
//  Cada estado: 2 amplitudes complexas (α, β) = 4 × f32 = 16 bytes
//  Layout: [α_re, α_im, β_re, β_im]
//
// ═══════════════════════════════════════════════════════════════════════

// Constantes matemáticas
const PI: f32 = 3.141592653589793;
const SQRT2_INV: f32 = 0.7071067811865476;  // 1/√2
const NUM_COMPONENTS: u32 = 4u;

// Tipos de gate
const GATE_HADAMARD: u32 = 0u;
const GATE_PAULI_X: u32 = 1u;
const GATE_PAULI_Y: u32 = 2u;
const GATE_PAULI_Z: u32 = 3u;
const GATE_RX: u32 = 4u;
const GATE_RY: u32 = 5u;
const GATE_RZ: u32 = 6u;
const GATE_PHASE: u32 = 7u;
const GATE_S: u32 = 8u;
const GATE_T: u32 = 9u;

// ─────────────────────────────────────────────────────────────────────────
//  Uniforms
// ─────────────────────────────────────────────────────────────────────────

struct GateParams {
    num_states: u32,
    gate_type: u32,
    theta: f32,         // Parâmetro de rotação (Rx, Ry, Rz, Phase)
    phi: f32,           // Parâmetro secundário (reservado)
}

// Matriz 2x2 complexa em formato explícito
// Elementos: [[m00, m01], [m10, m11]]
struct GateMatrix {
    m00_re: f32,
    m00_im: f32,
    m01_re: f32,
    m01_im: f32,
    m10_re: f32,
    m10_im: f32,
    m11_re: f32,
    m11_im: f32,
}

@group(0) @binding(0) var<uniform> params: GateParams;
@group(0) @binding(1) var<uniform> matrix: GateMatrix;
@group(0) @binding(2) var<storage, read> input_states: array<f32>;
@group(0) @binding(3) var<storage, read_write> output_states: array<f32>;

// ─────────────────────────────────────────────────────────────────────────
//  Operações Complexas
// ─────────────────────────────────────────────────────────────────────────

fn complex_mul(a_re: f32, a_im: f32, b_re: f32, b_im: f32) -> vec2<f32> {
    return vec2<f32>(
        a_re * b_re - a_im * b_im,
        a_re * b_im + a_im * b_re
    );
}

fn complex_add(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return a + b;
}

fn complex_scale(c: vec2<f32>, s: f32) -> vec2<f32> {
    return c * s;
}

// ─────────────────────────────────────────────────────────────────────────
//  Matrizes de Gates (calculadas on-the-fly)
// ─────────────────────────────────────────────────────────────────────────

// Hadamard: (1/√2) * [[1, 1], [1, -1]]
fn apply_hadamard(alpha_re: f32, alpha_im: f32, beta_re: f32, beta_im: f32) -> array<f32, 4> {
    var result: array<f32, 4>;
    result[0] = (alpha_re + beta_re) * SQRT2_INV;
    result[1] = (alpha_im + beta_im) * SQRT2_INV;
    result[2] = (alpha_re - beta_re) * SQRT2_INV;
    result[3] = (alpha_im - beta_im) * SQRT2_INV;
    return result;
}

// Pauli X: [[0, 1], [1, 0]] - bit flip
fn apply_pauli_x(alpha_re: f32, alpha_im: f32, beta_re: f32, beta_im: f32) -> array<f32, 4> {
    var result: array<f32, 4>;
    result[0] = beta_re;
    result[1] = beta_im;
    result[2] = alpha_re;
    result[3] = alpha_im;
    return result;
}

// Pauli Y: [[0, -i], [i, 0]] - bit flip com fase
fn apply_pauli_y(alpha_re: f32, alpha_im: f32, beta_re: f32, beta_im: f32) -> array<f32, 4> {
    var result: array<f32, 4>;
    // α' = -i * β = (0 - i)(β_re + iβ_im) = β_im - iβ_re
    result[0] = beta_im;
    result[1] = -beta_re;
    // β' = i * α = (0 + i)(α_re + iα_im) = -α_im + iα_re
    result[2] = -alpha_im;
    result[3] = alpha_re;
    return result;
}

// Pauli Z: [[1, 0], [0, -1]] - phase flip
fn apply_pauli_z(alpha_re: f32, alpha_im: f32, beta_re: f32, beta_im: f32) -> array<f32, 4> {
    var result: array<f32, 4>;
    result[0] = alpha_re;
    result[1] = alpha_im;
    result[2] = -beta_re;
    result[3] = -beta_im;
    return result;
}

// Rx(θ): [[cos(θ/2), -i*sin(θ/2)], [-i*sin(θ/2), cos(θ/2)]]
fn apply_rx(alpha_re: f32, alpha_im: f32, beta_re: f32, beta_im: f32, theta: f32) -> array<f32, 4> {
    let c = cos(theta * 0.5);
    let s = sin(theta * 0.5);

    var result: array<f32, 4>;
    // α' = cos(θ/2)*α - i*sin(θ/2)*β
    result[0] = c * alpha_re + s * beta_im;
    result[1] = c * alpha_im - s * beta_re;
    // β' = -i*sin(θ/2)*α + cos(θ/2)*β
    result[2] = s * alpha_im + c * beta_re;
    result[3] = -s * alpha_re + c * beta_im;
    return result;
}

// Ry(θ): [[cos(θ/2), -sin(θ/2)], [sin(θ/2), cos(θ/2)]]
fn apply_ry(alpha_re: f32, alpha_im: f32, beta_re: f32, beta_im: f32, theta: f32) -> array<f32, 4> {
    let c = cos(theta * 0.5);
    let s = sin(theta * 0.5);

    var result: array<f32, 4>;
    // α' = cos(θ/2)*α - sin(θ/2)*β
    result[0] = c * alpha_re - s * beta_re;
    result[1] = c * alpha_im - s * beta_im;
    // β' = sin(θ/2)*α + cos(θ/2)*β
    result[2] = s * alpha_re + c * beta_re;
    result[3] = s * alpha_im + c * beta_im;
    return result;
}

// Rz(θ): [[e^(-iθ/2), 0], [0, e^(iθ/2)]]
fn apply_rz(alpha_re: f32, alpha_im: f32, beta_re: f32, beta_im: f32, theta: f32) -> array<f32, 4> {
    let half_theta = theta * 0.5;
    let c_neg = cos(-half_theta);
    let s_neg = sin(-half_theta);
    let c_pos = cos(half_theta);
    let s_pos = sin(half_theta);

    var result: array<f32, 4>;
    // α' = e^(-iθ/2) * α
    let alpha_new = complex_mul(c_neg, s_neg, alpha_re, alpha_im);
    result[0] = alpha_new.x;
    result[1] = alpha_new.y;
    // β' = e^(iθ/2) * β
    let beta_new = complex_mul(c_pos, s_pos, beta_re, beta_im);
    result[2] = beta_new.x;
    result[3] = beta_new.y;
    return result;
}

// Phase(φ): [[1, 0], [0, e^(iφ)]]
fn apply_phase(alpha_re: f32, alpha_im: f32, beta_re: f32, beta_im: f32, phi: f32) -> array<f32, 4> {
    let c = cos(phi);
    let s = sin(phi);

    var result: array<f32, 4>;
    result[0] = alpha_re;
    result[1] = alpha_im;
    // β' = e^(iφ) * β
    let beta_new = complex_mul(c, s, beta_re, beta_im);
    result[2] = beta_new.x;
    result[3] = beta_new.y;
    return result;
}

// S Gate: [[1, 0], [0, i]] = Phase(π/2)
fn apply_s(alpha_re: f32, alpha_im: f32, beta_re: f32, beta_im: f32) -> array<f32, 4> {
    var result: array<f32, 4>;
    result[0] = alpha_re;
    result[1] = alpha_im;
    // β' = i * β = -β_im + iβ_re
    result[2] = -beta_im;
    result[3] = beta_re;
    return result;
}

// T Gate: [[1, 0], [0, e^(iπ/4)]] = Phase(π/4)
fn apply_t(alpha_re: f32, alpha_im: f32, beta_re: f32, beta_im: f32) -> array<f32, 4> {
    // e^(iπ/4) = cos(π/4) + i*sin(π/4) = (1 + i)/√2
    let c = SQRT2_INV;
    let s = SQRT2_INV;

    var result: array<f32, 4>;
    result[0] = alpha_re;
    result[1] = alpha_im;
    let beta_new = complex_mul(c, s, beta_re, beta_im);
    result[2] = beta_new.x;
    result[3] = beta_new.y;
    return result;
}

// ─────────────────────────────────────────────────────────────────────────
//  Aplicação de Matriz Customizada
// ─────────────────────────────────────────────────────────────────────────

fn apply_custom_matrix(alpha_re: f32, alpha_im: f32, beta_re: f32, beta_im: f32) -> array<f32, 4> {
    var result: array<f32, 4>;

    // α' = m00 * α + m01 * β
    let term1 = complex_mul(matrix.m00_re, matrix.m00_im, alpha_re, alpha_im);
    let term2 = complex_mul(matrix.m01_re, matrix.m01_im, beta_re, beta_im);
    let alpha_new = complex_add(term1, term2);

    // β' = m10 * α + m11 * β
    let term3 = complex_mul(matrix.m10_re, matrix.m10_im, alpha_re, alpha_im);
    let term4 = complex_mul(matrix.m11_re, matrix.m11_im, beta_re, beta_im);
    let beta_new = complex_add(term3, term4);

    result[0] = alpha_new.x;
    result[1] = alpha_new.y;
    result[2] = beta_new.x;
    result[3] = beta_new.y;

    return result;
}

// ─────────────────────────────────────────────────────────────────────────
//  Main Compute Kernel - Gate por tipo
// ─────────────────────────────────────────────────────────────────────────

@compute @workgroup_size(64)
fn apply_gate(@builtin(global_invocation_id) gid: vec3<u32>) {
    let state_idx = gid.x;

    if (state_idx >= params.num_states) {
        return;
    }

    let base = state_idx * NUM_COMPONENTS;

    let alpha_re = input_states[base];
    let alpha_im = input_states[base + 1u];
    let beta_re = input_states[base + 2u];
    let beta_im = input_states[base + 3u];

    var result: array<f32, 4>;

    switch (params.gate_type) {
        case GATE_HADAMARD: {
            result = apply_hadamard(alpha_re, alpha_im, beta_re, beta_im);
        }
        case GATE_PAULI_X: {
            result = apply_pauli_x(alpha_re, alpha_im, beta_re, beta_im);
        }
        case GATE_PAULI_Y: {
            result = apply_pauli_y(alpha_re, alpha_im, beta_re, beta_im);
        }
        case GATE_PAULI_Z: {
            result = apply_pauli_z(alpha_re, alpha_im, beta_re, beta_im);
        }
        case GATE_RX: {
            result = apply_rx(alpha_re, alpha_im, beta_re, beta_im, params.theta);
        }
        case GATE_RY: {
            result = apply_ry(alpha_re, alpha_im, beta_re, beta_im, params.theta);
        }
        case GATE_RZ: {
            result = apply_rz(alpha_re, alpha_im, beta_re, beta_im, params.theta);
        }
        case GATE_PHASE: {
            result = apply_phase(alpha_re, alpha_im, beta_re, beta_im, params.theta);
        }
        case GATE_S: {
            result = apply_s(alpha_re, alpha_im, beta_re, beta_im);
        }
        case GATE_T: {
            result = apply_t(alpha_re, alpha_im, beta_re, beta_im);
        }
        default: {
            // Fallback: usar matriz customizada
            result = apply_custom_matrix(alpha_re, alpha_im, beta_re, beta_im);
        }
    }

    output_states[base] = result[0];
    output_states[base + 1u] = result[1];
    output_states[base + 2u] = result[2];
    output_states[base + 3u] = result[3];
}

// ─────────────────────────────────────────────────────────────────────────
//  Kernel Alternativo - Matriz customizada diretamente
// ─────────────────────────────────────────────────────────────────────────

@compute @workgroup_size(64)
fn apply_matrix(@builtin(global_invocation_id) gid: vec3<u32>) {
    let state_idx = gid.x;

    if (state_idx >= params.num_states) {
        return;
    }

    let base = state_idx * NUM_COMPONENTS;

    let alpha_re = input_states[base];
    let alpha_im = input_states[base + 1u];
    let beta_re = input_states[base + 2u];
    let beta_im = input_states[base + 3u];

    let result = apply_custom_matrix(alpha_re, alpha_im, beta_re, beta_im);

    output_states[base] = result[0];
    output_states[base + 1u] = result[1];
    output_states[base + 2u] = result[2];
    output_states[base + 3u] = result[3];
}
