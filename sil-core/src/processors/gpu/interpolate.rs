//! Interpolação de estados no plano complexo

use crate::state::{ByteSil, SilState, NUM_LAYERS};

/// Interpolação linear (lerp) entre dois estados
/// 
/// Para cada camada: result = (1-t) × a + t × b
/// 
/// # Argumentos
/// * `a` - Estado inicial (t=0)
/// * `b` - Estado final (t=1)
/// * `t` - Parâmetro de interpolação [0, 1]
pub fn lerp_states(a: &SilState, b: &SilState, t: f32) -> SilState {
    let t = t.clamp(0.0, 1.0);
    let mut layers = [ByteSil::NULL; NUM_LAYERS];
    
    for i in 0..NUM_LAYERS {
        let rho_a = a.layers[i].rho as f32;
        let rho_b = b.layers[i].rho as f32;
        let theta_a = a.layers[i].theta as f32;
        let theta_b = b.layers[i].theta as f32;
        
        // Lerp linear
        let rho = rho_a * (1.0 - t) + rho_b * t;
        let theta = theta_a * (1.0 - t) + theta_b * t;
        
        // Converter de volta
        let rho_i = rho.round().clamp(-8.0, 7.0) as i8;
        let theta_u = (theta.round() as i32).rem_euclid(16) as u8;
        
        layers[i] = ByteSil::new(rho_i, theta_u);
    }
    
    SilState { layers }
}

/// Interpolação esférica (slerp) entre dois estados
/// 
/// Para cada camada:
/// - ρ: interpolação linear
/// - θ: interpolação pelo caminho mais curto no círculo
/// 
/// # Argumentos
/// * `a` - Estado inicial (t=0)
/// * `b` - Estado final (t=1)
/// * `t` - Parâmetro de interpolação [0, 1]
pub fn slerp_states(a: &SilState, b: &SilState, t: f32) -> SilState {
    let t = t.clamp(0.0, 1.0);
    let mut layers = [ByteSil::NULL; NUM_LAYERS];
    
    for i in 0..NUM_LAYERS {
        let rho_a = a.layers[i].rho as f32;
        let rho_b = b.layers[i].rho as f32;
        let theta_a = a.layers[i].theta as f32;
        let theta_b = b.layers[i].theta as f32;
        
        // ρ: lerp linear
        let rho = rho_a * (1.0 - t) + rho_b * t;
        
        // θ: slerp circular (caminho mais curto)
        let theta = slerp_angle(theta_a, theta_b, t);
        
        // Converter de volta
        let rho_i = rho.round().clamp(-8.0, 7.0) as i8;
        let theta_u = (theta.round() as i32).rem_euclid(16) as u8;
        
        layers[i] = ByteSil::new(rho_i, theta_u);
    }
    
    SilState { layers }
}

/// Interpolação de ângulo pelo caminho mais curto
fn slerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let mut delta = b - a;
    
    // Caminho mais curto no círculo [0, 16)
    if delta > 8.0 {
        delta -= 16.0;
    } else if delta < -8.0 {
        delta += 16.0;
    }
    
    let mut result = a + delta * t;
    
    // Normalizar para [0, 16)
    if result < 0.0 {
        result += 16.0;
    } else if result >= 16.0 {
        result -= 16.0;
    }
    
    result
}

/// Gera sequência de estados interpolados
/// 
/// # Argumentos
/// * `a` - Estado inicial
/// * `b` - Estado final
/// * `steps` - Número de estados intermediários (inclui a e b)
/// * `use_slerp` - true para slerp, false para lerp
pub fn interpolate_sequence(a: &SilState, b: &SilState, steps: usize, use_slerp: bool) -> Vec<SilState> {
    if steps == 0 {
        return vec![];
    }
    if steps == 1 {
        return vec![*a];
    }
    
    let interp_fn = if use_slerp { slerp_states } else { lerp_states };
    
    (0..steps)
        .map(|i| {
            let t = i as f32 / (steps - 1) as f32;
            interp_fn(a, b, t)
        })
        .collect()
}

/// Interpolação Bezier quadrática (3 pontos de controle)
/// 
/// B(t) = (1-t)²×P₀ + 2(1-t)t×P₁ + t²×P₂
pub fn bezier_quadratic(p0: &SilState, p1: &SilState, p2: &SilState, t: f32) -> SilState {
    let t = t.clamp(0.0, 1.0);
    let t2 = t * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    
    let mut layers = [ByteSil::NULL; NUM_LAYERS];
    
    for i in 0..NUM_LAYERS {
        let rho = mt2 * p0.layers[i].rho as f32 
                + 2.0 * mt * t * p1.layers[i].rho as f32 
                + t2 * p2.layers[i].rho as f32;
        
        // Para theta, usamos interpolação circular
        let theta_01 = slerp_angle(p0.layers[i].theta as f32, p1.layers[i].theta as f32, t);
        let theta_12 = slerp_angle(p1.layers[i].theta as f32, p2.layers[i].theta as f32, t);
        let theta = slerp_angle(theta_01, theta_12, t);
        
        let rho_i = rho.round().clamp(-8.0, 7.0) as i8;
        let theta_u = (theta.round() as i32).rem_euclid(16) as u8;
        
        layers[i] = ByteSil::new(rho_i, theta_u);
    }
    
    SilState { layers }
}

/// Interpolação Bezier cúbica (4 pontos de controle)
/// 
/// B(t) = (1-t)³×P₀ + 3(1-t)²t×P₁ + 3(1-t)t²×P₂ + t³×P₃
pub fn bezier_cubic(p0: &SilState, p1: &SilState, p2: &SilState, p3: &SilState, t: f32) -> SilState {
    let t = t.clamp(0.0, 1.0);
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    
    let mut layers = [ByteSil::NULL; NUM_LAYERS];
    
    for i in 0..NUM_LAYERS {
        let rho = mt3 * p0.layers[i].rho as f32 
                + 3.0 * mt2 * t * p1.layers[i].rho as f32 
                + 3.0 * mt * t2 * p2.layers[i].rho as f32 
                + t3 * p3.layers[i].rho as f32;
        
        // Para theta, usamos De Casteljau com slerp
        let theta_01 = slerp_angle(p0.layers[i].theta as f32, p1.layers[i].theta as f32, t);
        let theta_12 = slerp_angle(p1.layers[i].theta as f32, p2.layers[i].theta as f32, t);
        let theta_23 = slerp_angle(p2.layers[i].theta as f32, p3.layers[i].theta as f32, t);
        let theta_012 = slerp_angle(theta_01, theta_12, t);
        let theta_123 = slerp_angle(theta_12, theta_23, t);
        let theta = slerp_angle(theta_012, theta_123, t);
        
        let rho_i = rho.round().clamp(-8.0, 7.0) as i8;
        let theta_u = (theta.round() as i32).rem_euclid(16) as u8;
        
        layers[i] = ByteSil::new(rho_i, theta_u);
    }
    
    SilState { layers }
}

/// Calcula "distância" entre dois estados
/// 
/// Soma das distâncias por camada no plano complexo
pub fn state_distance(a: &SilState, b: &SilState) -> f32 {
    let mut total = 0.0f32;
    
    for i in 0..NUM_LAYERS {
        let za = a.layers[i].to_complex();
        let zb = b.layers[i].to_complex();
        total += (za - zb).norm() as f32;
    }
    
    total
}

/// Calcula "distância" geodésica (considerando periodicidade de θ)
pub fn geodesic_distance(a: &SilState, b: &SilState) -> f32 {
    let mut total = 0.0;
    
    for i in 0..NUM_LAYERS {
        let d_rho = (a.layers[i].rho as f32 - b.layers[i].rho as f32).abs();
        
        let d_theta = {
            let ta = a.layers[i].theta as f32;
            let tb = b.layers[i].theta as f32;
            let mut d = (ta - tb).abs();
            if d > 8.0 { d = 16.0 - d; }
            d
        };
        
        // Distância no espaço (ρ, θ)
        total += (d_rho * d_rho + d_theta * d_theta).sqrt();
    }
    
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lerp_endpoints() {
        let a = SilState::vacuum();
        let b = SilState::maximum();
        
        let at_0 = lerp_states(&a, &b, 0.0);
        let at_1 = lerp_states(&a, &b, 1.0);
        
        // t=0 deve ser igual a 'a'
        for i in 0..NUM_LAYERS {
            assert_eq!(at_0.layers[i].to_u8(), a.layers[i].to_u8());
        }
        
        // t=1 deve ser igual a 'b'
        for i in 0..NUM_LAYERS {
            assert_eq!(at_1.layers[i].to_u8(), b.layers[i].to_u8());
        }
    }
    
    #[test]
    fn test_slerp_angle() {
        // Caminho curto de 15 para 1 (passando por 0)
        let result = slerp_angle(15.0, 1.0, 0.5);
        assert!((result - 0.0).abs() < 0.01 || (result - 16.0).abs() < 0.01);
        
        // Caminho curto de 1 para 3
        let result = slerp_angle(1.0, 3.0, 0.5);
        assert!((result - 2.0).abs() < 0.01);
    }
    
    #[test]
    fn test_interpolate_sequence() {
        let a = SilState::vacuum();
        let b = SilState::neutral();
        
        let seq = interpolate_sequence(&a, &b, 5, false);
        
        assert_eq!(seq.len(), 5);
        
        // Distância deve aumentar monotonicamente de a
        let mut prev_dist = 0.0;
        for state in &seq[1..] {
            let dist = state_distance(&a, state);
            assert!(dist >= prev_dist - 0.1); // tolerância numérica
            prev_dist = dist;
        }
    }
    
    #[test]
    fn test_bezier_endpoints() {
        let p0 = SilState::vacuum();
        let p1 = SilState::neutral();
        let p2 = SilState::maximum();
        
        let at_0 = bezier_quadratic(&p0, &p1, &p2, 0.0);
        let at_1 = bezier_quadratic(&p0, &p1, &p2, 1.0);
        
        // t=0 deve ser p0
        for i in 0..NUM_LAYERS {
            assert_eq!(at_0.layers[i].to_u8(), p0.layers[i].to_u8());
        }
        
        // t=1 deve ser p2
        for i in 0..NUM_LAYERS {
            assert_eq!(at_1.layers[i].to_u8(), p2.layers[i].to_u8());
        }
    }
    
    #[test]
    fn test_state_distance_self_zero() {
        let a = SilState::neutral();
        assert!(state_distance(&a, &a) < 1e-6);
    }
    
    #[test]
    fn test_geodesic_distance_symmetric() {
        let a = SilState::vacuum();
        let b = SilState::maximum();
        
        let d1 = geodesic_distance(&a, &b);
        let d2 = geodesic_distance(&b, &a);
        
        assert!((d1 - d2).abs() < 1e-6);
    }
}
