// ═══════════════════════════════════════════════════════════════════════
//  SIL INTERPOLATE COMPUTE SHADER
// ═══════════════════════════════════════════════════════════════════════

const NUM_LAYERS: u32 = 16u;
const TWO_PI: f32 = 6.283185307179586;

struct InterpolateParams {
    num_pairs: u32,
    t: f32,           // Parâmetro de interpolação [0, 1]
    use_slerp: u32,   // 0 = lerp, 1 = slerp
    _padding: f32,
}

@group(0) @binding(0) var<storage, read> states_a: array<u32>;
@group(0) @binding(1) var<storage, read> states_b: array<u32>;
@group(0) @binding(2) var<storage, read_write> output: array<u32>;
@group(0) @binding(3) var<uniform> params: InterpolateParams;

fn extract_layer(states: ptr<storage, array<u32>, read>, state_idx: u32, layer_idx: u32) -> vec2<f32> {
    let word_idx = state_idx * 4u + layer_idx / 4u;
    let byte_offset = (layer_idx % 4u) * 8u;
    let byte_val = ((*states)[word_idx] >> byte_offset) & 0xFFu;
    
    let rho_raw = i32((byte_val >> 4u) & 0xFu);
    let rho = select(rho_raw, rho_raw - 16, rho_raw > 7);
    let theta = f32(byte_val & 0xFu);
    
    return vec2<f32>(f32(rho), theta);
}

fn pack_layer(rho: i32, theta: u32) -> u32 {
    let rho_u = u32(select(rho, rho + 16, rho < 0)) & 0xFu;
    return (rho_u << 4u) | (theta & 0xFu);
}

// Lerp linear
fn lerp_layer(a: vec2<f32>, b: vec2<f32>, t: f32) -> vec2<f32> {
    return vec2<f32>(
        mix(a.x, b.x, t),
        mix(a.y, b.y, t)
    );
}

// Slerp esférico (para interpolação de fase)
fn slerp_angle(a: f32, b: f32, t: f32) -> f32 {
    var delta = b - a;
    
    // Caminho mais curto no círculo
    if (delta > 8.0) {
        delta = delta - 16.0;
    } else if (delta < -8.0) {
        delta = delta + 16.0;
    }
    
    var result = a + delta * t;
    
    // Normalizar para [0, 16)
    if (result < 0.0) {
        result = result + 16.0;
    } else if (result >= 16.0) {
        result = result - 16.0;
    }
    
    return result;
}

@compute @workgroup_size(64)
fn interpolate(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pair_idx = global_id.x;
    
    if (pair_idx >= params.num_pairs) {
        return;
    }
    
    let t = params.t;
    let use_slerp = params.use_slerp != 0u;
    
    // Interpolar cada camada
    for (var layer: u32 = 0u; layer < NUM_LAYERS; layer = layer + 1u) {
        let a = extract_layer(&states_a, pair_idx, layer);
        let b = extract_layer(&states_b, pair_idx, layer);
        
        var result: vec2<f32>;
        
        if (use_slerp) {
            // Slerp: interpolar magnitude linearmente, fase circularmente
            result = vec2<f32>(
                mix(a.x, b.x, t),
                slerp_angle(a.y, b.y, t)
            );
        } else {
            // Lerp simples
            result = lerp_layer(a, b, t);
        }
        
        // Clamping
        let rho_clamped = clamp(i32(round(result.x)), -8, 7);
        let theta_clamped = u32(round(result.y)) % 16u;
        
        // Pack e armazenar
        let packed = pack_layer(rho_clamped, theta_clamped);
        let word_idx = pair_idx * 4u + layer / 4u;
        let byte_offset = (layer % 4u) * 8u;
        
        // Atomic para thread-safety
        let old = atomicLoad(&output[word_idx]);
        let mask = ~(0xFFu << byte_offset);
        let new_val = (old & mask) | (packed << byte_offset);
        atomicStore(&output[word_idx], new_val);
    }
}
