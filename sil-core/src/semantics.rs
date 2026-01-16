//! # LayerSemantics — Interpretacao Semantica por Camada
//!
//! Cada uma das 16 camadas do SIL interpreta ρ (magnitude) e θ (fase)
//! de forma unica, conferindo significado especifico ao ByteSil.
//!
//! ## Grupos de Camadas
//!
//! | Grupo | Camadas | Funcao |
//! |-------|---------|--------|
//! | Percepcao | L0-L4 | Sensores (luz, som, cheiro, sabor, tato) |
//! | Processamento | L5-L7 | Computacao (hardware, motor, ambiente) |
//! | Interacao | L8-LA | Comunicacao (controle, soberania, etica) |
//! | Emergencia | LB-LC | Padroes (complexidade, coerencia) |
//! | Meta | LD-LF | Controle (superposicao, emaranhamento, colapso) |

use std::fmt;

// =============================================================================
// Trait Principal
// =============================================================================

/// Trait para interpretacao semantica de uma camada SIL
pub trait LayerSemantics: Send + Sync {
    /// Nome da camada
    fn name(&self) -> &'static str;

    /// Indice hexadecimal (0x0 a 0xF)
    fn index(&self) -> u8;

    /// Grupo funcional da camada
    fn group(&self) -> LayerGroup;

    /// Interpreta ρ (magnitude) no contexto desta camada
    fn interpret_rho(&self, rho: i8) -> RhoInterpretation;

    /// Interpreta θ (fase) no contexto desta camada
    fn interpret_theta(&self, theta: u8) -> ThetaInterpretation;

    /// Cor associada a camada (wavelength em nm)
    fn wavelength(&self) -> u16 {
        // Espectro de 700nm (vermelho) a 380nm (UV)
        700 - (self.index() as u16 * 20)
    }
}

// =============================================================================
// Grupos de Camadas
// =============================================================================

/// Grupo funcional de uma camada
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LayerGroup {
    /// L0-L4: Sensores (percepcao do mundo fisico)
    Perception = 0,
    /// L5-L7: Computacao local (processamento)
    Processing = 1,
    /// L8-LA: Comunicacao (interacao com outros)
    Interaction = 2,
    /// LB-LC: Padroes emergentes
    Emergence = 3,
    /// LD-LF: Controle de fluxo meta
    Meta = 4,
}

impl LayerGroup {
    /// Retorna o grupo para um indice de camada
    pub fn from_index(index: u8) -> Self {
        match index {
            0x0..=0x4 => Self::Perception,
            0x5..=0x7 => Self::Processing,
            0x8..=0xA => Self::Interaction,
            0xB..=0xC => Self::Emergence,
            0xD..=0xF => Self::Meta,
            _ => Self::Meta,
        }
    }

    /// Nome do grupo
    pub fn name(&self) -> &'static str {
        match self {
            Self::Perception => "Perception",
            Self::Processing => "Processing",
            Self::Interaction => "Interaction",
            Self::Emergence => "Emergence",
            Self::Meta => "Meta",
        }
    }

    /// Camadas pertencentes ao grupo
    pub fn layers(&self) -> &'static [u8] {
        match self {
            Self::Perception => &[0, 1, 2, 3, 4],
            Self::Processing => &[5, 6, 7],
            Self::Interaction => &[8, 9, 10],
            Self::Emergence => &[11, 12],
            Self::Meta => &[13, 14, 15],
        }
    }
}

// =============================================================================
// Interpretacao de ρ (Magnitude)
// =============================================================================

/// Interpretacao semantica de ρ (log-magnitude) por camada
#[derive(Clone, Debug, PartialEq)]
pub enum RhoInterpretation {
    // === Percepcao (L0-L4) ===
    /// L0: Intensidade luminosa (log lux)
    LightIntensity(f64),
    /// L1: Amplitude sonora (dB)
    SoundAmplitude(f64),
    /// L2: Concentracao de VOC (log ppm)
    ChemicalConcentration(f64),
    /// L3: Intensidade de sabor (log)
    TasteIntensity(f64),
    /// L4: Intensidade tatil (log)
    TouchIntensity(f64),

    // === Processamento (L5-L7) ===
    /// L5: Carga computacional (log FLOPS)
    ComputationalLoad(f64),
    /// L6: Intensidade de acao (log forca/torque)
    ActionIntensity(f64),
    /// L7: Confianca do contexto (0-1)
    ContextConfidence(f64),

    // === Interacao (L8-LA) ===
    /// L8: Erro do setpoint (log |erro|)
    SetpointError(f64),
    /// L9: Forca soberana (log |poder|)
    SovereigntyStrength(f64),
    /// LA: Alcance etico (log |escopo|)
    EthicalScope(f64),

    // === Emergencia (LB-LC) ===
    /// LB: Grau de emergencia (log |novidade|)
    EmergenceDegree(f64),
    /// LC: Grau de coerencia (log |ψ|²)
    CoherenceDegree(f64),

    // === Meta (LD-LF) ===
    /// LD: Numero de branches (log₂ |forks|)
    BranchCount(u32),
    /// LE: Grau de emaranhamento (log |concorrencia|)
    EntanglementDegree(f64),
    /// LF: Irreversibilidade (log |entropia|)
    Irreversibility(f64),

    /// Valor generico quando camada nao e especificada
    Generic(f64),
}

impl RhoInterpretation {
    /// Converte ρ bruto para valor da escala log
    pub fn from_raw(rho: i8) -> f64 {
        (rho as f64).exp()
    }

    /// Retorna o valor numerico subjacente
    pub fn value(&self) -> f64 {
        match self {
            Self::LightIntensity(v) => *v,
            Self::SoundAmplitude(v) => *v,
            Self::ChemicalConcentration(v) => *v,
            Self::TasteIntensity(v) => *v,
            Self::TouchIntensity(v) => *v,
            Self::ComputationalLoad(v) => *v,
            Self::ActionIntensity(v) => *v,
            Self::ContextConfidence(v) => *v,
            Self::SetpointError(v) => *v,
            Self::SovereigntyStrength(v) => *v,
            Self::EthicalScope(v) => *v,
            Self::EmergenceDegree(v) => *v,
            Self::CoherenceDegree(v) => *v,
            Self::BranchCount(v) => *v as f64,
            Self::EntanglementDegree(v) => *v,
            Self::Irreversibility(v) => *v,
            Self::Generic(v) => *v,
        }
    }
}

impl fmt::Display for RhoInterpretation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LightIntensity(v) => write!(f, "{:.2} lux", v),
            Self::SoundAmplitude(v) => write!(f, "{:.1} dB", v),
            Self::ChemicalConcentration(v) => write!(f, "{:.2} ppm", v),
            Self::TasteIntensity(v) => write!(f, "taste={:.2}", v),
            Self::TouchIntensity(v) => write!(f, "touch={:.2}", v),
            Self::ComputationalLoad(v) => write!(f, "{:.2} FLOPS", v),
            Self::ActionIntensity(v) => write!(f, "{:.2} N", v),
            Self::ContextConfidence(v) => write!(f, "{:.1}% conf", v * 100.0),
            Self::SetpointError(v) => write!(f, "err={:.3}", v),
            Self::SovereigntyStrength(v) => write!(f, "sov={:.2}", v),
            Self::EthicalScope(v) => write!(f, "scope={:.2}", v),
            Self::EmergenceDegree(v) => write!(f, "emerge={:.2}", v),
            Self::CoherenceDegree(v) => write!(f, "coh={:.3}", v),
            Self::BranchCount(v) => write!(f, "{} branches", v),
            Self::EntanglementDegree(v) => write!(f, "entangle={:.3}", v),
            Self::Irreversibility(v) => write!(f, "entropy={:.2}", v),
            Self::Generic(v) => write!(f, "ρ={:.3}", v),
        }
    }
}

// =============================================================================
// Interpretacao de θ (Fase)
// =============================================================================

/// Interpretacao semantica de θ (fase) por camada
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThetaInterpretation {
    // === Percepcao (L0-L4) ===
    /// L0: Matiz/cor (16 valores no circulo cromatico)
    Hue(Hue),
    /// L1: Classe de frequencia (16 bandas)
    FrequencyClass(FrequencyClass),
    /// L2: Classe quimica (gradiente olfativo)
    ChemicalClass(ChemicalClass),
    /// L3: Modalidade de sabor
    TasteMode(TasteMode),
    /// L4: Modalidade tatil
    TouchMode(TouchMode),

    // === Processamento (L5-L7) ===
    /// L5: Modo de operacao
    OperationMode(OperationMode),
    /// L6: Primitiva motora
    MotorPrimitive(MotorPrimitive),
    /// L7: Tipo de contexto
    ContextType(ContextType),

    // === Interacao (L8-LA) ===
    /// L8: Modo de controle cibernetico
    ControlMode(ControlMode),
    /// L9: Regime de governanca
    GovernanceMode(GovernanceMode),
    /// LA: Framework etico
    EthicalMode(EthicalMode),

    // === Emergencia (LB-LC) ===
    /// LB: Tipo de auto-organizacao
    OrgType(OrgType),
    /// LC: Regime quantico
    QuantumRegime(QuantumRegime),

    // === Meta (LD-LF) ===
    /// LD: Estrategia de superposicao
    SuperStrategy(SuperStrategy),
    /// LE: Tipo de correlacao
    CorrelationType(CorrelationType),
    /// LF: Tipo de colapso
    CollapseType(CollapseType),

    /// Valor generico (0-15)
    Generic(u8),
}

impl ThetaInterpretation {
    /// Retorna o valor numerico (0-15)
    pub fn value(&self) -> u8 {
        match self {
            Self::Hue(h) => *h as u8,
            Self::FrequencyClass(f) => *f as u8,
            Self::ChemicalClass(c) => *c as u8,
            Self::TasteMode(t) => *t as u8,
            Self::TouchMode(t) => *t as u8,
            Self::OperationMode(o) => *o as u8,
            Self::MotorPrimitive(m) => *m as u8,
            Self::ContextType(c) => *c as u8,
            Self::ControlMode(c) => *c as u8,
            Self::GovernanceMode(g) => *g as u8,
            Self::EthicalMode(e) => *e as u8,
            Self::OrgType(o) => *o as u8,
            Self::QuantumRegime(q) => *q as u8,
            Self::SuperStrategy(s) => *s as u8,
            Self::CorrelationType(c) => *c as u8,
            Self::CollapseType(c) => *c as u8,
            Self::Generic(v) => *v,
        }
    }
}

// =============================================================================
// Enums de θ — Percepcao (L0-L4)
// =============================================================================

/// L0: Matiz no circulo cromatico (16 cores)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Hue {
    Red = 0,
    RedOrange = 1,
    Orange = 2,
    YellowOrange = 3,
    Yellow = 4,
    YellowGreen = 5,
    Green = 6,
    BlueGreen = 7,
    Cyan = 8,
    BlueCyan = 9,
    Blue = 10,
    BlueViolet = 11,
    Violet = 12,
    Magenta = 13,
    RedMagenta = 14,
    Pink = 15,
}

impl Hue {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Red,
            1 => Self::RedOrange,
            2 => Self::Orange,
            3 => Self::YellowOrange,
            4 => Self::Yellow,
            5 => Self::YellowGreen,
            6 => Self::Green,
            7 => Self::BlueGreen,
            8 => Self::Cyan,
            9 => Self::BlueCyan,
            10 => Self::Blue,
            11 => Self::BlueViolet,
            12 => Self::Violet,
            13 => Self::Magenta,
            14 => Self::RedMagenta,
            _ => Self::Pink,
        }
    }
}

/// L1: Classe de frequencia sonora (16 bandas)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FrequencyClass {
    SubBass = 0,       // 20-60 Hz
    Bass = 1,          // 60-120 Hz
    LowMid = 2,        // 120-250 Hz
    Mid = 3,           // 250-500 Hz
    UpperMid = 4,      // 500-1k Hz
    Presence = 5,      // 1-2k Hz
    Brilliance = 6,    // 2-4k Hz
    HighBrilliance = 7, // 4-8k Hz
    Air = 8,           // 8-16k Hz
    UltraHigh = 9,     // 16-20k Hz
    Infrasound = 10,   // < 20 Hz
    Ultrasound = 11,   // > 20k Hz
    Noise = 12,        // Ruido branco
    Click = 13,        // Transiente
    Tone = 14,         // Tom puro
    Complex = 15,      // Harmonicos
}

impl FrequencyClass {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::SubBass,
            1 => Self::Bass,
            2 => Self::LowMid,
            3 => Self::Mid,
            4 => Self::UpperMid,
            5 => Self::Presence,
            6 => Self::Brilliance,
            7 => Self::HighBrilliance,
            8 => Self::Air,
            9 => Self::UltraHigh,
            10 => Self::Infrasound,
            11 => Self::Ultrasound,
            12 => Self::Noise,
            13 => Self::Click,
            14 => Self::Tone,
            _ => Self::Complex,
        }
    }
}

/// L2: Classe quimica (olfato)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ChemicalClass {
    Floral = 0,
    Fruity = 1,
    Citrus = 2,
    Green = 3,
    Woody = 4,
    Spicy = 5,
    Sulfurous = 6,
    Earthy = 7,
    Smoky = 8,
    Metallic = 9,
    Chemical = 10,
    Putrid = 11,
    Musky = 12,
    Minty = 13,
    Sweet = 14,
    Neutral = 15,
}

impl ChemicalClass {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Floral,
            1 => Self::Fruity,
            2 => Self::Citrus,
            3 => Self::Green,
            4 => Self::Woody,
            5 => Self::Spicy,
            6 => Self::Sulfurous,
            7 => Self::Earthy,
            8 => Self::Smoky,
            9 => Self::Metallic,
            10 => Self::Chemical,
            11 => Self::Putrid,
            12 => Self::Musky,
            13 => Self::Minty,
            14 => Self::Sweet,
            _ => Self::Neutral,
        }
    }
}

/// L3: Modalidade de sabor
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TasteMode {
    Sweet = 0,
    SweetMild = 1,
    SweetIntense = 2,
    Salty = 3,
    SaltyMild = 4,
    Sour = 5,
    SourMild = 6,
    SourIntense = 7,
    Bitter = 8,
    BitterMild = 9,
    Umami = 10,
    UmamiIntense = 11,
    Astringent = 12,
    Pungent = 13,
    Cooling = 14,
    Neutral = 15,
}

impl TasteMode {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Sweet,
            1 => Self::SweetMild,
            2 => Self::SweetIntense,
            3 => Self::Salty,
            4 => Self::SaltyMild,
            5 => Self::Sour,
            6 => Self::SourMild,
            7 => Self::SourIntense,
            8 => Self::Bitter,
            9 => Self::BitterMild,
            10 => Self::Umami,
            11 => Self::UmamiIntense,
            12 => Self::Astringent,
            13 => Self::Pungent,
            14 => Self::Cooling,
            _ => Self::Neutral,
        }
    }
}

/// L4: Modalidade tatil
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TouchMode {
    Pressure = 0,
    PressureLight = 1,
    PressureHeavy = 2,
    Vibration = 3,
    VibrationFine = 4,
    VibrationCoarse = 5,
    Temperature = 6,
    Cold = 7,
    Hot = 8,
    Pain = 9,
    PainSharp = 10,
    PainDull = 11,
    Proprioception = 12,
    Position = 13,
    Movement = 14,
    Neutral = 15,
}

impl TouchMode {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Pressure,
            1 => Self::PressureLight,
            2 => Self::PressureHeavy,
            3 => Self::Vibration,
            4 => Self::VibrationFine,
            5 => Self::VibrationCoarse,
            6 => Self::Temperature,
            7 => Self::Cold,
            8 => Self::Hot,
            9 => Self::Pain,
            10 => Self::PainSharp,
            11 => Self::PainDull,
            12 => Self::Proprioception,
            13 => Self::Position,
            14 => Self::Movement,
            _ => Self::Neutral,
        }
    }
}

// =============================================================================
// Enums de θ — Processamento (L5-L7)
// =============================================================================

/// L5: Modo de operacao computacional
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OperationMode {
    Idle = 0,
    IdleLowPower = 1,
    Sensing = 2,
    SensingActive = 3,
    Processing = 4,
    ProcessingBatch = 5,
    Inference = 6,
    InferenceStream = 7,
    Training = 8,
    TrainingDistributed = 9,
    Compression = 10,
    Decompression = 11,
    Communication = 12,
    CommunicationSecure = 13,
    Critical = 14,
    Emergency = 15,
}

impl OperationMode {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Idle,
            1 => Self::IdleLowPower,
            2 => Self::Sensing,
            3 => Self::SensingActive,
            4 => Self::Processing,
            5 => Self::ProcessingBatch,
            6 => Self::Inference,
            7 => Self::InferenceStream,
            8 => Self::Training,
            9 => Self::TrainingDistributed,
            10 => Self::Compression,
            11 => Self::Decompression,
            12 => Self::Communication,
            13 => Self::CommunicationSecure,
            14 => Self::Critical,
            _ => Self::Emergency,
        }
    }
}

/// L6: Primitiva motora
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MotorPrimitive {
    Idle = 0,
    IdleReady = 1,
    Reach = 2,
    ReachExtend = 3,
    Grasp = 4,
    GraspPinch = 5,
    Manipulate = 6,
    ManipulateRotate = 7,
    Release = 8,
    ReleaseGentle = 9,
    Locomote = 10,
    LocomoteFast = 11,
    Orient = 12,
    OrientPrecise = 13,
    Emergency = 14,
    EmergencyStop = 15,
}

impl MotorPrimitive {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Idle,
            1 => Self::IdleReady,
            2 => Self::Reach,
            3 => Self::ReachExtend,
            4 => Self::Grasp,
            5 => Self::GraspPinch,
            6 => Self::Manipulate,
            7 => Self::ManipulateRotate,
            8 => Self::Release,
            9 => Self::ReleaseGentle,
            10 => Self::Locomote,
            11 => Self::LocomoteFast,
            12 => Self::Orient,
            13 => Self::OrientPrecise,
            14 => Self::Emergency,
            _ => Self::EmergencyStop,
        }
    }
}

/// L7: Tipo de contexto ambiental
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ContextType {
    Unknown = 0,
    UnknownDangerous = 1,
    Indoor = 2,
    IndoorResidential = 3,
    Outdoor = 4,
    OutdoorUrban = 5,
    Transit = 6,
    TransitVehicle = 7,
    Social = 8,
    SocialCrowded = 9,
    Work = 10,
    WorkIndustrial = 11,
    Rest = 12,
    RestSleep = 13,
    Emergency = 14,
    EmergencyCritical = 15,
}

impl ContextType {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Unknown,
            1 => Self::UnknownDangerous,
            2 => Self::Indoor,
            3 => Self::IndoorResidential,
            4 => Self::Outdoor,
            5 => Self::OutdoorUrban,
            6 => Self::Transit,
            7 => Self::TransitVehicle,
            8 => Self::Social,
            9 => Self::SocialCrowded,
            10 => Self::Work,
            11 => Self::WorkIndustrial,
            12 => Self::Rest,
            13 => Self::RestSleep,
            14 => Self::Emergency,
            _ => Self::EmergencyCritical,
        }
    }
}

// =============================================================================
// Enums de θ — Interacao (L8-LA)
// =============================================================================

/// L8: Modo de controle cibernetico
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ControlMode {
    /// Setpoint atingido
    Equilibrium = 0,
    EquilibriumStable = 1,
    /// Correcao ativa
    Correcting = 2,
    CorrectingFast = 3,
    /// Ajuste de parametros
    Adapting = 4,
    AdaptingSlow = 5,
    /// Aprendizado de modelo
    Learning = 6,
    LearningDeep = 7,
    /// Exploracao do espaco
    Exploring = 8,
    ExploringRandom = 9,
    /// Competicao (jogo)
    Competing = 10,
    CompetingAggressive = 11,
    /// Cooperacao (jogo)
    Cooperating = 12,
    CooperatingAltruistic = 13,
    /// Modo de falha segura
    Emergency = 14,
    EmergencyShutdown = 15,
}

impl ControlMode {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Equilibrium,
            1 => Self::EquilibriumStable,
            2 => Self::Correcting,
            3 => Self::CorrectingFast,
            4 => Self::Adapting,
            5 => Self::AdaptingSlow,
            6 => Self::Learning,
            7 => Self::LearningDeep,
            8 => Self::Exploring,
            9 => Self::ExploringRandom,
            10 => Self::Competing,
            11 => Self::CompetingAggressive,
            12 => Self::Cooperating,
            13 => Self::CooperatingAltruistic,
            14 => Self::Emergency,
            _ => Self::EmergencyShutdown,
        }
    }
}

/// L9: Regime de governanca
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GovernanceMode {
    /// Auto-governanca plena
    Autonomous = 0,
    AutonomousSovereign = 1,
    /// Federacao com delegacao
    Federated = 2,
    FederatedLoose = 3,
    /// Alianca com parceiros
    Allied = 4,
    AlliedStrong = 5,
    /// Neutralidade declarada
    Neutral = 6,
    NeutralArmed = 7,
    /// Territorio em disputa
    Disputed = 8,
    DisputedActive = 9,
    /// Sob controle externo
    Occupied = 10,
    OccupiedResisting = 11,
    /// Regime transitorio
    Transitional = 12,
    TransitionalDemocratic = 13,
    /// Estado falido
    Collapsed = 14,
    CollapsedAnarchy = 15,
}

impl GovernanceMode {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Autonomous,
            1 => Self::AutonomousSovereign,
            2 => Self::Federated,
            3 => Self::FederatedLoose,
            4 => Self::Allied,
            5 => Self::AlliedStrong,
            6 => Self::Neutral,
            7 => Self::NeutralArmed,
            8 => Self::Disputed,
            9 => Self::DisputedActive,
            10 => Self::Occupied,
            11 => Self::OccupiedResisting,
            12 => Self::Transitional,
            13 => Self::TransitionalDemocratic,
            14 => Self::Collapsed,
            _ => Self::CollapsedAnarchy,
        }
    }
}

/// LA: Framework etico
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EthicalMode {
    /// Regras universais (Kant)
    Deontological = 0,
    DeontologicalStrict = 1,
    /// Maximizar bem-estar (Mill)
    Consequentialist = 2,
    ConsequentialistRule = 3,
    /// Carater e excelencia (Aristoteles)
    Virtue = 4,
    VirtueAncient = 5,
    /// Acordo mutuo (Rawls)
    Contractual = 6,
    ContractualSocial = 7,
    /// Relacoes e responsabilidade (Gilligan)
    Care = 8,
    CareRelational = 9,
    /// "Sou porque somos" (Filosofia Africana)
    Ubuntu = 10,
    UbuntuCommunal = 11,
    /// Etica planetaria (Lovelock)
    Gaian = 12,
    GaianDeep = 13,
    /// Etica inter-estelar
    Cosmic = 14,
    CosmicUniversal = 15,
}

impl EthicalMode {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Deontological,
            1 => Self::DeontologicalStrict,
            2 => Self::Consequentialist,
            3 => Self::ConsequentialistRule,
            4 => Self::Virtue,
            5 => Self::VirtueAncient,
            6 => Self::Contractual,
            7 => Self::ContractualSocial,
            8 => Self::Care,
            9 => Self::CareRelational,
            10 => Self::Ubuntu,
            11 => Self::UbuntuCommunal,
            12 => Self::Gaian,
            13 => Self::GaianDeep,
            14 => Self::Cosmic,
            _ => Self::CosmicUniversal,
        }
    }
}

// =============================================================================
// Enums de θ — Emergencia (LB-LC)
// =============================================================================

/// LB: Tipo de auto-organizacao
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OrgType {
    /// Ordem periodica
    Crystalline = 0,
    CrystallineSymmetric = 1,
    /// Estruturas longe do equilibrio
    Dissipative = 2,
    DissipativeOscillating = 3,
    /// Auto-producao
    Autopoietic = 4,
    AutopoieticClosed = 5,
    /// Inteligencia de enxame
    Swarm = 6,
    SwarmDistributed = 7,
    /// Redes complexas
    Network = 8,
    NetworkScaleFree = 9,
    /// Selecao e variacao
    Evolutionary = 10,
    EvolutionaryDarwinian = 11,
    /// Emergencia mental
    Cognitive = 12,
    CognitiveConscious = 13,
    /// Meta-emergencia
    Transcendent = 14,
    TranscendentSingular = 15,
}

impl OrgType {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Crystalline,
            1 => Self::CrystallineSymmetric,
            2 => Self::Dissipative,
            3 => Self::DissipativeOscillating,
            4 => Self::Autopoietic,
            5 => Self::AutopoieticClosed,
            6 => Self::Swarm,
            7 => Self::SwarmDistributed,
            8 => Self::Network,
            9 => Self::NetworkScaleFree,
            10 => Self::Evolutionary,
            11 => Self::EvolutionaryDarwinian,
            12 => Self::Cognitive,
            13 => Self::CognitiveConscious,
            14 => Self::Transcendent,
            _ => Self::TranscendentSingular,
        }
    }
}

/// LC: Regime quantico
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum QuantumRegime {
    Classical = 0,
    ClassicalLimit = 1,
    Semiclassical = 2,
    SemiclassicalWKB = 3,
    Coherent = 4,
    CoherentGlauber = 5,
    Superposed = 6,
    SuperposedCat = 7,
    Entangled = 8,
    EntangledBell = 9,
    Tunneling = 10,
    TunnelingResonant = 11,
    Interfering = 12,
    InterferingMach = 13,
    Collapsed = 14,
    CollapsedMeasured = 15,
}

impl QuantumRegime {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Classical,
            1 => Self::ClassicalLimit,
            2 => Self::Semiclassical,
            3 => Self::SemiclassicalWKB,
            4 => Self::Coherent,
            5 => Self::CoherentGlauber,
            6 => Self::Superposed,
            7 => Self::SuperposedCat,
            8 => Self::Entangled,
            9 => Self::EntangledBell,
            10 => Self::Tunneling,
            11 => Self::TunnelingResonant,
            12 => Self::Interfering,
            13 => Self::InterferingMach,
            14 => Self::Collapsed,
            _ => Self::CollapsedMeasured,
        }
    }
}

// =============================================================================
// Enums de θ — Meta (LD-LF)
// =============================================================================

/// LD: Estrategia de superposicao
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SuperStrategy {
    Collapsed = 0,
    CollapsedFinal = 1,
    Binary = 2,
    BinarySymmetric = 3,
    Ternary = 4,
    TernaryBalanced = 5,
    Quaternary = 6,
    QuaternaryQubit = 7,
    Exponential = 8,
    ExponentialGrowth = 9,
    Continuous = 10,
    ContinuousSmooth = 11,
    Hierarchical = 12,
    HierarchicalTree = 13,
    Infinite = 14,
    InfiniteCountable = 15,
}

impl SuperStrategy {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Collapsed,
            1 => Self::CollapsedFinal,
            2 => Self::Binary,
            3 => Self::BinarySymmetric,
            4 => Self::Ternary,
            5 => Self::TernaryBalanced,
            6 => Self::Quaternary,
            7 => Self::QuaternaryQubit,
            8 => Self::Exponential,
            9 => Self::ExponentialGrowth,
            10 => Self::Continuous,
            11 => Self::ContinuousSmooth,
            12 => Self::Hierarchical,
            13 => Self::HierarchicalTree,
            14 => Self::Infinite,
            _ => Self::InfiniteCountable,
        }
    }
}

/// LE: Tipo de correlacao
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CorrelationType {
    Separable = 0,
    SeparableProduct = 1,
    Classical = 2,
    ClassicalCorrelated = 3,
    Discord = 4,
    DiscordQuantum = 5,
    Bipartite = 6,
    BipartiteMaximal = 7,
    Multipartite = 8,
    MultipartiteGenuine = 9,
    GHZ = 10,
    GHZState = 11,
    WState = 12,
    WStateBalanced = 13,
    Cluster = 14,
    ClusterGraph = 15,
}

impl CorrelationType {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Separable,
            1 => Self::SeparableProduct,
            2 => Self::Classical,
            3 => Self::ClassicalCorrelated,
            4 => Self::Discord,
            5 => Self::DiscordQuantum,
            6 => Self::Bipartite,
            7 => Self::BipartiteMaximal,
            8 => Self::Multipartite,
            9 => Self::MultipartiteGenuine,
            10 => Self::GHZ,
            11 => Self::GHZState,
            12 => Self::WState,
            13 => Self::WStateBalanced,
            14 => Self::Cluster,
            _ => Self::ClusterGraph,
        }
    }
}

/// LF: Tipo de colapso
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CollapseType {
    Null = 0,
    NullVoid = 1,
    SoftReset = 2,
    SoftResetPartial = 3,
    HardReset = 4,
    HardResetFull = 5,
    Measurement = 6,
    MeasurementStrong = 7,
    Observation = 8,
    ObservationWeak = 9,
    Termination = 10,
    TerminationGraceful = 11,
    Death = 12,
    DeathPermanent = 13,
    EOF = 14,
    EOFCycle = 15,
}

impl CollapseType {
    pub fn from_theta(theta: u8) -> Self {
        match theta & 0x0F {
            0 => Self::Null,
            1 => Self::NullVoid,
            2 => Self::SoftReset,
            3 => Self::SoftResetPartial,
            4 => Self::HardReset,
            5 => Self::HardResetFull,
            6 => Self::Measurement,
            7 => Self::MeasurementStrong,
            8 => Self::Observation,
            9 => Self::ObservationWeak,
            10 => Self::Termination,
            11 => Self::TerminationGraceful,
            12 => Self::Death,
            13 => Self::DeathPermanent,
            14 => Self::EOF,
            _ => Self::EOFCycle,
        }
    }
}

// =============================================================================
// Funcoes Utilitarias
// =============================================================================

/// Interpreta ρ para uma camada especifica
pub fn interpret_rho_for_layer(layer: u8, rho: i8) -> RhoInterpretation {
    let value = (rho as f64).exp();
    match layer {
        0x0 => RhoInterpretation::LightIntensity(value),
        0x1 => RhoInterpretation::SoundAmplitude(rho as f64 * 6.0), // aproximadamente dB
        0x2 => RhoInterpretation::ChemicalConcentration(value),
        0x3 => RhoInterpretation::TasteIntensity(value),
        0x4 => RhoInterpretation::TouchIntensity(value),
        0x5 => RhoInterpretation::ComputationalLoad(value),
        0x6 => RhoInterpretation::ActionIntensity(value),
        0x7 => RhoInterpretation::ContextConfidence((rho as f64 + 8.0) / 15.0),
        0x8 => RhoInterpretation::SetpointError(value),
        0x9 => RhoInterpretation::SovereigntyStrength(value),
        0xA => RhoInterpretation::EthicalScope(value),
        0xB => RhoInterpretation::EmergenceDegree(value),
        0xC => RhoInterpretation::CoherenceDegree((rho as f64 + 8.0) / 15.0),
        0xD => RhoInterpretation::BranchCount(1u32 << (rho.max(0) as u32)),
        0xE => RhoInterpretation::EntanglementDegree((rho as f64 + 8.0) / 15.0),
        0xF => RhoInterpretation::Irreversibility(value),
        _ => RhoInterpretation::Generic(value),
    }
}

/// Interpreta θ para uma camada especifica
pub fn interpret_theta_for_layer(layer: u8, theta: u8) -> ThetaInterpretation {
    match layer {
        0x0 => ThetaInterpretation::Hue(Hue::from_theta(theta)),
        0x1 => ThetaInterpretation::FrequencyClass(FrequencyClass::from_theta(theta)),
        0x2 => ThetaInterpretation::ChemicalClass(ChemicalClass::from_theta(theta)),
        0x3 => ThetaInterpretation::TasteMode(TasteMode::from_theta(theta)),
        0x4 => ThetaInterpretation::TouchMode(TouchMode::from_theta(theta)),
        0x5 => ThetaInterpretation::OperationMode(OperationMode::from_theta(theta)),
        0x6 => ThetaInterpretation::MotorPrimitive(MotorPrimitive::from_theta(theta)),
        0x7 => ThetaInterpretation::ContextType(ContextType::from_theta(theta)),
        0x8 => ThetaInterpretation::ControlMode(ControlMode::from_theta(theta)),
        0x9 => ThetaInterpretation::GovernanceMode(GovernanceMode::from_theta(theta)),
        0xA => ThetaInterpretation::EthicalMode(EthicalMode::from_theta(theta)),
        0xB => ThetaInterpretation::OrgType(OrgType::from_theta(theta)),
        0xC => ThetaInterpretation::QuantumRegime(QuantumRegime::from_theta(theta)),
        0xD => ThetaInterpretation::SuperStrategy(SuperStrategy::from_theta(theta)),
        0xE => ThetaInterpretation::CorrelationType(CorrelationType::from_theta(theta)),
        0xF => ThetaInterpretation::CollapseType(CollapseType::from_theta(theta)),
        _ => ThetaInterpretation::Generic(theta),
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_groups() {
        assert_eq!(LayerGroup::from_index(0), LayerGroup::Perception);
        assert_eq!(LayerGroup::from_index(4), LayerGroup::Perception);
        assert_eq!(LayerGroup::from_index(5), LayerGroup::Processing);
        assert_eq!(LayerGroup::from_index(8), LayerGroup::Interaction);
        assert_eq!(LayerGroup::from_index(0xB), LayerGroup::Emergence);
        assert_eq!(LayerGroup::from_index(0xF), LayerGroup::Meta);
    }

    #[test]
    fn test_rho_interpretation() {
        let light = interpret_rho_for_layer(0x0, 0);
        assert!(matches!(light, RhoInterpretation::LightIntensity(_)));
        assert!((light.value() - 1.0).abs() < 0.001); // e^0 = 1

        let coherence = interpret_rho_for_layer(0xC, 7);
        assert!(matches!(coherence, RhoInterpretation::CoherenceDegree(_)));
        assert!(coherence.value() > 0.9); // rho=7 -> alta coerencia
    }

    #[test]
    fn test_theta_interpretation() {
        let hue = interpret_theta_for_layer(0x0, 0);
        assert!(matches!(hue, ThetaInterpretation::Hue(Hue::Red)));

        let ctrl = interpret_theta_for_layer(0x8, 0);
        assert!(matches!(
            ctrl,
            ThetaInterpretation::ControlMode(ControlMode::Equilibrium)
        ));

        let ethics = interpret_theta_for_layer(0xA, 10);
        assert!(matches!(
            ethics,
            ThetaInterpretation::EthicalMode(EthicalMode::Ubuntu)
        ));
    }

    #[test]
    fn test_ethical_modes() {
        assert_eq!(EthicalMode::from_theta(0), EthicalMode::Deontological);
        assert_eq!(EthicalMode::from_theta(2), EthicalMode::Consequentialist);
        assert_eq!(EthicalMode::from_theta(10), EthicalMode::Ubuntu);
        assert_eq!(EthicalMode::from_theta(12), EthicalMode::Gaian);
        assert_eq!(EthicalMode::from_theta(14), EthicalMode::Cosmic);
    }

    #[test]
    fn test_governance_modes() {
        assert_eq!(GovernanceMode::from_theta(0), GovernanceMode::Autonomous);
        assert_eq!(GovernanceMode::from_theta(2), GovernanceMode::Federated);
        assert_eq!(GovernanceMode::from_theta(14), GovernanceMode::Collapsed);
    }

    #[test]
    fn test_collapse_types() {
        assert_eq!(CollapseType::from_theta(0), CollapseType::Null);
        assert_eq!(CollapseType::from_theta(6), CollapseType::Measurement);
        assert_eq!(CollapseType::from_theta(14), CollapseType::EOF);
    }
}
