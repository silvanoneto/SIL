//! Modelos de energia para diferentes processadores
//!
//! Cada modelo implementa a trait `EnergyModel` que estima o consumo
//! de energia em Joules baseado em métricas de execução.

use crate::{ProcessorType, constants};
use std::time::Duration;

/// Trait para modelos de estimativa de energia
pub trait EnergyModel: Send + Sync {
    /// Nome do modelo
    fn name(&self) -> &str;

    /// Tipo de processador
    fn processor_type(&self) -> ProcessorType;

    /// Joules por operação base (para operações SIL típicas)
    fn joules_per_operation(&self) -> f64;

    /// Joules por ciclo de clock
    fn joules_per_cycle(&self) -> f64;

    /// Potência em idle (Watts)
    fn idle_watts(&self) -> f64;

    /// Potência máxima (Watts)
    fn max_watts(&self) -> f64;

    /// Estima energia consumida
    ///
    /// # Argumentos
    ///
    /// * `duration` - Tempo de execução
    /// * `operations` - Número de operações executadas
    /// * `utilization` - Utilização do processador (0.0 - 1.0)
    ///
    /// # Retorna
    ///
    /// Energia estimada em Joules
    fn estimate_joules(
        &self,
        duration: Duration,
        operations: u64,
        utilization: f32,
    ) -> f64 {
        // Modelo básico: E = P_idle + (P_max - P_idle) * utilization * t
        let seconds = duration.as_secs_f64();
        let power = self.idle_watts() + (self.max_watts() - self.idle_watts()) * utilization as f64;

        // Energia base do tempo
        let time_energy = power * seconds;

        // Energia das operações
        let op_energy = operations as f64 * self.joules_per_operation();

        // Usa o maior valor (modelo conservador)
        time_energy.max(op_energy) * constants::OS_OVERHEAD_FACTOR
    }

    /// Estima potência instantânea (Watts)
    fn estimate_watts(&self, utilization: f32) -> f64 {
        self.idle_watts() + (self.max_watts() - self.idle_watts()) * utilization as f64
    }

    /// Clona o modelo em Box
    fn clone_box(&self) -> Box<dyn EnergyModel>;
}

/// Modelo de energia para CPU
#[derive(Debug, Clone)]
pub struct CpuEnergyModel {
    /// Nome do modelo
    name: String,
    /// Joules por ciclo
    joules_per_cycle: f64,
    /// Frequência em Hz
    frequency_hz: f64,
    /// Potência idle (W)
    idle_watts: f64,
    /// Potência máxima (W)
    max_watts: f64,
    /// Fator de eficiência (0-1)
    efficiency: f64,
}

impl CpuEnergyModel {
    /// Cria modelo customizado
    pub fn new(
        name: impl Into<String>,
        joules_per_cycle: f64,
        frequency_hz: f64,
        idle_watts: f64,
        max_watts: f64,
    ) -> Self {
        Self {
            name: name.into(),
            joules_per_cycle,
            frequency_hz,
            idle_watts,
            max_watts,
            efficiency: 1.0,
        }
    }

    /// Define fator de eficiência
    pub fn with_efficiency(mut self, efficiency: f64) -> Self {
        self.efficiency = efficiency.clamp(0.0, 1.0);
        self
    }

    /// Detecta automaticamente o modelo de CPU
    #[cfg(target_os = "macos")]
    pub fn detect() -> Self {
        // Apple Silicon detection
        if cfg!(target_arch = "aarch64") {
            Self::apple_silicon()
        } else {
            Self::x86_default()
        }
    }

    /// Detecta automaticamente o modelo de CPU
    #[cfg(target_os = "linux")]
    pub fn detect() -> Self {
        // Tenta detectar via /proc/cpuinfo
        Self::linux_detect().unwrap_or_else(|| Self::x86_default())
    }

    /// Detecta automaticamente o modelo de CPU
    #[cfg(target_os = "windows")]
    pub fn detect() -> Self {
        Self::x86_default()
    }

    /// Detecta automaticamente o modelo de CPU (fallback)
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    pub fn detect() -> Self {
        Self::x86_default()
    }

    /// Modelo para Apple Silicon (M1/M2/M3/M4)
    pub fn apple_silicon() -> Self {
        Self {
            name: "Apple Silicon (M-series)".into(),
            joules_per_cycle: constants::JOULES_PER_CYCLE_M1_PERF,
            frequency_hz: 3_200_000_000.0, // 3.2 GHz
            idle_watts: 0.5,
            max_watts: 15.0,
            efficiency: 0.95, // Alta eficiência
        }
    }

    /// Modelo para Apple Silicon efficiency cores
    pub fn apple_silicon_efficiency() -> Self {
        Self {
            name: "Apple Silicon E-core".into(),
            joules_per_cycle: constants::JOULES_PER_CYCLE_M1_EFF,
            frequency_hz: 2_000_000_000.0, // 2 GHz
            idle_watts: 0.1,
            max_watts: 2.0,
            efficiency: 0.98,
        }
    }

    /// Modelo padrão para x86-64
    pub fn x86_default() -> Self {
        Self {
            name: "x86-64 Generic".into(),
            joules_per_cycle: constants::JOULES_PER_CYCLE_X86,
            frequency_hz: 4_000_000_000.0, // 4 GHz
            idle_watts: 5.0,
            max_watts: 65.0,
            efficiency: 0.85,
        }
    }

    /// Modelo para ARM Cortex-M
    pub fn arm_cortex_m() -> Self {
        Self {
            name: "ARM Cortex-M".into(),
            joules_per_cycle: constants::JOULES_PER_CYCLE_ARM_M4,
            frequency_hz: 100_000_000.0, // 100 MHz
            idle_watts: 0.001,
            max_watts: 0.1,
            efficiency: 0.90,
        }
    }

    #[cfg(target_os = "linux")]
    fn linux_detect() -> Option<Self> {
        use std::fs;

        let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok()?;

        if cpuinfo.contains("Apple") {
            Some(Self::apple_silicon())
        } else if cpuinfo.contains("AMD") || cpuinfo.contains("Intel") {
            Some(Self::x86_default())
        } else {
            None
        }
    }
}

impl Default for CpuEnergyModel {
    fn default() -> Self {
        Self::detect()
    }
}

impl EnergyModel for CpuEnergyModel {
    fn name(&self) -> &str {
        &self.name
    }

    fn processor_type(&self) -> ProcessorType {
        ProcessorType::Cpu
    }

    fn joules_per_operation(&self) -> f64 {
        // Uma operação SIL típica = ~10 ciclos de CPU
        self.joules_per_cycle * 10.0 / self.efficiency
    }

    fn joules_per_cycle(&self) -> f64 {
        self.joules_per_cycle / self.efficiency
    }

    fn idle_watts(&self) -> f64 {
        self.idle_watts
    }

    fn max_watts(&self) -> f64 {
        self.max_watts
    }

    fn estimate_joules(
        &self,
        duration: Duration,
        operations: u64,
        utilization: f32,
    ) -> f64 {
        let seconds = duration.as_secs_f64();

        // Modelo refinado para CPU
        // E = P_base * t + E_dynamic * ops
        let base_power = self.idle_watts + (self.max_watts - self.idle_watts) * utilization as f64;
        let base_energy = base_power * seconds;

        // Energia dinâmica das operações
        let dynamic_energy = operations as f64 * self.joules_per_operation();

        // Soma com overhead do sistema
        (base_energy + dynamic_energy) * constants::OS_OVERHEAD_FACTOR / constants::DC_DC_EFFICIENCY
    }

    fn clone_box(&self) -> Box<dyn EnergyModel> {
        Box::new(self.clone())
    }
}

/// Modelo de energia para GPU
#[derive(Debug, Clone)]
pub struct GpuEnergyModel {
    /// Nome do modelo
    name: String,
    /// Joules por operação de shader
    joules_per_shader_op: f64,
    /// Potência idle (W)
    idle_watts: f64,
    /// Potência máxima (W)
    max_watts: f64,
    /// Overhead de transferência de memória (fator)
    memory_transfer_overhead: f64,
}

impl GpuEnergyModel {
    /// Cria modelo customizado
    pub fn new(
        name: impl Into<String>,
        joules_per_shader_op: f64,
        idle_watts: f64,
        max_watts: f64,
    ) -> Self {
        Self {
            name: name.into(),
            joules_per_shader_op,
            idle_watts,
            max_watts,
            memory_transfer_overhead: 1.2, // 20% overhead
        }
    }

    /// Modelo para GPU integrada (Apple/Intel)
    pub fn integrated() -> Self {
        Self {
            name: "Integrated GPU".into(),
            joules_per_shader_op: 1e-9,
            idle_watts: constants::GPU_IDLE_WATTS,
            max_watts: 15.0,
            memory_transfer_overhead: 1.1, // Memória compartilhada
        }
    }

    /// Modelo para GPU discreta (NVIDIA/AMD)
    pub fn discrete() -> Self {
        Self {
            name: "Discrete GPU".into(),
            joules_per_shader_op: 0.5e-9,
            idle_watts: 10.0,
            max_watts: 200.0,
            memory_transfer_overhead: 1.5, // PCIe transfer
        }
    }
}

impl Default for GpuEnergyModel {
    fn default() -> Self {
        Self::integrated()
    }
}

impl EnergyModel for GpuEnergyModel {
    fn name(&self) -> &str {
        &self.name
    }

    fn processor_type(&self) -> ProcessorType {
        ProcessorType::Gpu
    }

    fn joules_per_operation(&self) -> f64 {
        self.joules_per_shader_op * self.memory_transfer_overhead
    }

    fn joules_per_cycle(&self) -> f64 {
        // GPU não tem conceito de "ciclo" como CPU
        self.joules_per_shader_op
    }

    fn idle_watts(&self) -> f64 {
        self.idle_watts
    }

    fn max_watts(&self) -> f64 {
        self.max_watts
    }

    fn clone_box(&self) -> Box<dyn EnergyModel> {
        Box::new(self.clone())
    }
}

/// Modelo de energia para NPU (Neural Processing Unit)
#[derive(Debug, Clone)]
pub struct NpuEnergyModel {
    /// Nome do modelo
    name: String,
    /// TOPS (Tera Operations Per Second)
    tops: f64,
    /// Watts por TOP
    watts_per_top: f64,
    /// Potência idle (W)
    idle_watts: f64,
}

impl NpuEnergyModel {
    /// Cria modelo customizado
    pub fn new(name: impl Into<String>, tops: f64, watts_per_top: f64) -> Self {
        Self {
            name: name.into(),
            tops,
            watts_per_top,
            idle_watts: 0.1,
        }
    }

    /// Modelo para Apple Neural Engine
    pub fn apple_neural_engine() -> Self {
        Self {
            name: "Apple Neural Engine".into(),
            tops: 15.8, // M1 Neural Engine
            watts_per_top: 0.5,
            idle_watts: 0.05,
        }
    }

    /// Modelo para Qualcomm Hexagon
    pub fn qualcomm_hexagon() -> Self {
        Self {
            name: "Qualcomm Hexagon DSP".into(),
            tops: 10.0,
            watts_per_top: 0.8,
            idle_watts: 0.1,
        }
    }
}

impl Default for NpuEnergyModel {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self::apple_neural_engine()
        }
        #[cfg(not(target_os = "macos"))]
        {
            Self::qualcomm_hexagon()
        }
    }
}

impl EnergyModel for NpuEnergyModel {
    fn name(&self) -> &str {
        &self.name
    }

    fn processor_type(&self) -> ProcessorType {
        ProcessorType::Npu
    }

    fn joules_per_operation(&self) -> f64 {
        // Uma operação SIL no NPU ≈ 1000 MACs
        constants::JOULES_PER_NPU_INFERENCE / 1000.0
    }

    fn joules_per_cycle(&self) -> f64 {
        self.joules_per_operation()
    }

    fn idle_watts(&self) -> f64 {
        self.idle_watts
    }

    fn max_watts(&self) -> f64 {
        self.tops * self.watts_per_top
    }

    fn estimate_joules(
        &self,
        duration: Duration,
        operations: u64,
        utilization: f32,
    ) -> f64 {
        let seconds = duration.as_secs_f64();

        // NPU é muito eficiente quando em uso
        let power = self.idle_watts + (self.max_watts() - self.idle_watts) * utilization as f64;
        let time_energy = power * seconds;

        // Energia por inferências (cada operação SIL = micro-inferência)
        let op_energy = operations as f64 * self.joules_per_operation();

        time_energy + op_energy
    }

    fn clone_box(&self) -> Box<dyn EnergyModel> {
        Box::new(self.clone())
    }
}

/// Modelo híbrido que combina múltiplos processadores
#[derive(Debug, Clone)]
pub struct HybridEnergyModel {
    /// Modelos componentes
    cpu: CpuEnergyModel,
    gpu: GpuEnergyModel,
    npu: NpuEnergyModel,
    /// Pesos para cada tipo (deve somar 1.0)
    weights: (f64, f64, f64), // (cpu, gpu, npu)
}

impl HybridEnergyModel {
    /// Cria modelo híbrido com pesos customizados
    pub fn new(
        cpu: CpuEnergyModel,
        gpu: GpuEnergyModel,
        npu: NpuEnergyModel,
        weights: (f64, f64, f64),
    ) -> Self {
        let total = weights.0 + weights.1 + weights.2;
        let normalized = (weights.0 / total, weights.1 / total, weights.2 / total);
        Self {
            cpu,
            gpu,
            npu,
            weights: normalized,
        }
    }

    /// Detecta e cria modelo híbrido automaticamente
    pub fn detect() -> Self {
        Self {
            cpu: CpuEnergyModel::detect(),
            gpu: GpuEnergyModel::default(),
            npu: NpuEnergyModel::default(),
            weights: (0.6, 0.3, 0.1), // CPU dominante por padrão
        }
    }

    /// Define pesos dinamicamente baseado no workload
    pub fn with_workload_weights(mut self, cpu: f64, gpu: f64, npu: f64) -> Self {
        let total = cpu + gpu + npu;
        self.weights = (cpu / total, gpu / total, npu / total);
        self
    }
}

impl Default for HybridEnergyModel {
    fn default() -> Self {
        Self::detect()
    }
}

impl EnergyModel for HybridEnergyModel {
    fn name(&self) -> &str {
        "Hybrid (CPU+GPU+NPU)"
    }

    fn processor_type(&self) -> ProcessorType {
        ProcessorType::Hybrid
    }

    fn joules_per_operation(&self) -> f64 {
        self.cpu.joules_per_operation() * self.weights.0
            + self.gpu.joules_per_operation() * self.weights.1
            + self.npu.joules_per_operation() * self.weights.2
    }

    fn joules_per_cycle(&self) -> f64 {
        self.cpu.joules_per_cycle() * self.weights.0
    }

    fn idle_watts(&self) -> f64 {
        self.cpu.idle_watts() + self.gpu.idle_watts() + self.npu.idle_watts()
    }

    fn max_watts(&self) -> f64 {
        self.cpu.max_watts() + self.gpu.max_watts() + self.npu.max_watts()
    }

    fn estimate_joules(
        &self,
        duration: Duration,
        operations: u64,
        utilization: f32,
    ) -> f64 {
        // Distribui operações pelos pesos
        let cpu_ops = (operations as f64 * self.weights.0) as u64;
        let gpu_ops = (operations as f64 * self.weights.1) as u64;
        let npu_ops = (operations as f64 * self.weights.2) as u64;

        self.cpu.estimate_joules(duration, cpu_ops, utilization)
            + self.gpu.estimate_joules(duration, gpu_ops, utilization)
            + self.npu.estimate_joules(duration, npu_ops, utilization)
    }

    fn clone_box(&self) -> Box<dyn EnergyModel> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_model_detect() {
        let model = CpuEnergyModel::detect();
        assert!(!model.name.is_empty());
        assert!(model.joules_per_cycle > 0.0);
    }

    #[test]
    fn test_cpu_estimate_joules() {
        let model = CpuEnergyModel::apple_silicon();
        let joules = model.estimate_joules(
            Duration::from_millis(100),
            10_000,
            0.5,
        );
        assert!(joules > 0.0);
        assert!(joules < 10.0); // Sanity check
    }

    #[test]
    fn test_gpu_model() {
        let model = GpuEnergyModel::integrated();
        assert!(model.joules_per_operation() < 1e-6);
        assert!(model.max_watts() > model.idle_watts());
    }

    #[test]
    fn test_npu_model() {
        let model = NpuEnergyModel::default();
        assert!(model.tops > 0.0);
        assert!(model.max_watts() > 0.0);
    }

    #[test]
    fn test_hybrid_model() {
        let model = HybridEnergyModel::detect();
        let joules = model.estimate_joules(
            Duration::from_millis(50),
            5_000,
            0.7,
        );
        assert!(joules > 0.0);
    }

    #[test]
    fn test_hybrid_weights_normalize() {
        let model = HybridEnergyModel::new(
            CpuEnergyModel::detect(),
            GpuEnergyModel::default(),
            NpuEnergyModel::default(),
            (6.0, 3.0, 1.0), // Total = 10
        );

        let (c, g, n) = model.weights;
        assert!((c + g + n - 1.0).abs() < 0.001);
    }
}
