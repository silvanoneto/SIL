# SIL - Diagramas de Arquitetura

> Visualização completa da arquitetura do projeto SIL/LIS usando diagramas Mermaid

---

## 1. Arquitetura Geral do Sistema

```mermaid
flowchart TB
    subgraph USER["👤 Usuário"]
        CLI[lis-cli]
        API[lis-api]
        VSCODE[sil-vscode]
    end

    subgraph COMPILER["🔧 Compilador LIS"]
        LEXER[Lexer]
        PARSER[Parser]
        TYPECK[TypeChecker]
        CODEGEN[Compiler]
    end

    subgraph RUNTIME["⚙️ Runtime SIL"]
        ASM[Assembler]
        VSP[VSP VM]
        STATE[SilState]
        BYTESIL[ByteSil]
    end

    subgraph BACKENDS["🖥️ Backends"]
        CPU[CPU/SIMD]
        GPU[GPU/wgpu]
        NPU[NPU/CoreML]
    end

    subgraph MODALITIES["📡 Modalidades L0-LF"]
        SENSE[Sensores L0-L4]
        PROC[Processadores L5-L7]
        INTER[Interação L8-LA]
        EMERGE[Emergência LB-LF]
    end

    subgraph ML["🧠 Paebiru ML"]
        CORE[Core Ops]
        LAYERS[Layers]
        ARCH[Architectures]
        DIST[Distributed]
        EDGE[Edge Routing]
    end

    CLI --> LEXER
    API --> LEXER
    VSCODE --> LEXER

    LEXER --> PARSER
    PARSER --> TYPECK
    TYPECK --> CODEGEN
    CODEGEN --> ASM
    ASM --> VSP

    VSP --> CPU
    VSP --> GPU
    VSP --> NPU

    VSP --> STATE
    STATE --> BYTESIL

    STATE --> SENSE
    STATE --> PROC
    STATE --> INTER
    STATE --> EMERGE

    ML --> CODEGEN
```

---

## 2. Pipeline de Compilação

```mermaid
flowchart LR
    subgraph INPUT["📝 Entrada"]
        LIS[".lis<br/>Código Fonte"]
    end

    subgraph FRONTEND["Frontend"]
        LEX["Lexer<br/>(Logos)"]
        PARSE["Parser<br/>(Chumsky)"]
        AST["AST<br/>Abstract Syntax Tree"]
    end

    subgraph MIDDLE["Análise"]
        TYPE["Type Checker<br/>Inferência de Tipos"]
        RESOLVE["Module Resolver<br/>Dependências"]
    end

    subgraph BACKEND["Backend"]
        COMP["Compiler<br/>Geração de Código"]
        ASSEM["Assembler<br/>Bytecode"]
    end

    subgraph OUTPUT["📦 Saída"]
        SIL[".sil<br/>Assembly"]
        SILC[".silc<br/>Bytecode"]
    end

    LIS --> LEX
    LEX -->|Tokens| PARSE
    PARSE --> AST
    AST --> TYPE
    TYPE --> RESOLVE
    RESOLVE --> COMP
    COMP --> SIL
    SIL --> ASSEM
    ASSEM --> SILC

    style LIS fill:#e1f5fe
    style SIL fill:#fff3e0
    style SILC fill:#e8f5e9
```

---

## 3. Modelo de 16 Camadas (L0-LF)

```mermaid
flowchart TB
    subgraph PERCEPTION["🎯 PERCEPÇÃO (L0-L4)"]
        direction LR
        L0["L0<br/>Photonic<br/>👁️ Visual"]
        L1["L1<br/>Acoustic<br/>👂 Audio"]
        L2["L2<br/>Olfactory<br/>👃 Olfato"]
        L3["L3<br/>Gustatory<br/>👅 Gustativo"]
        L4["L4<br/>Dermic<br/>✋ Tátil"]
    end

    subgraph PROCESSING["⚡ PROCESSAMENTO (L5-L7)"]
        direction LR
        L5["L5<br/>Electronic<br/>🔌 Sinais"]
        L6["L6<br/>Psychomotor<br/>🦾 Atuação"]
        L7["L7<br/>Environmental<br/>🌍 Contexto"]
    end

    subgraph INTERACTION["🌐 INTERAÇÃO (L8-LA)"]
        direction LR
        L8["L8<br/>Cybernetic<br/>🔗 Rede"]
        L9["L9<br/>Geopolitical<br/>🏛️ Governança"]
        LA["LA<br/>Cosmopolitical<br/>⚖️ Ética"]
    end

    subgraph EMERGENCE["✨ EMERGÊNCIA (LB-LC)"]
        direction LR
        LB["LB<br/>Synergic<br/>🐝 Enxame"]
        LC["LC<br/>Quantum<br/>⚛️ Quântico"]
    end

    subgraph META["🔮 META (LD-LF)"]
        direction LR
        LD["LD<br/>Superposition<br/>📊 Multi-estado"]
        LE["LE<br/>Entanglement<br/>🔀 Correlação"]
        LF["LF<br/>Collapse<br/>🎯 Resolução"]
    end

    PERCEPTION --> PROCESSING
    PROCESSING --> INTERACTION
    INTERACTION --> EMERGENCE
    EMERGENCE --> META
    META -->|"Feedback Loop"| PERCEPTION

    style PERCEPTION fill:#e3f2fd
    style PROCESSING fill:#fff3e0
    style INTERACTION fill:#e8f5e9
    style EMERGENCE fill:#f3e5f5
    style META fill:#fce4ec
```

---

## 4. ByteSil - Representação Log-Polar

```mermaid
flowchart LR
    subgraph BYTESIL["ByteSil (8 bits)"]
        direction TB
        RHO["ρ (rho)<br/>4 bits<br/>Magnitude<br/>-8 a +7"]
        THETA["θ (theta)<br/>4 bits<br/>Fase<br/>0 a 2π"]
    end

    subgraph COMPLEX["Número Complexo"]
        MAG["Magnitude = 2^ρ"]
        PHASE["Fase = θ × (2π/16)"]
        CART["z = r × e^(iθ)"]
    end

    subgraph OPS["Operações"]
        MUL["mul: ρ₁+ρ₂, θ₁+θ₂"]
        DIV["div: ρ₁-ρ₂, θ₁-θ₂"]
        POW["pow: ρ×n, θ×n"]
        CONJ["conj: ρ, -θ"]
    end

    RHO --> MAG
    THETA --> PHASE
    MAG --> CART
    PHASE --> CART

    BYTESIL --> OPS

    style BYTESIL fill:#e8eaf6
    style COMPLEX fill:#fff8e1
    style OPS fill:#e0f2f1
```

---

## 5. SilState - Container de 16 Camadas

```mermaid
flowchart TB
    subgraph SILSTATE["SilState (128 bits = 16 × 8 bits)"]
        direction LR
        B0["L0<br/>8b"]
        B1["L1<br/>8b"]
        B2["L2<br/>8b"]
        B3["L3<br/>8b"]
        B4["L4<br/>8b"]
        B5["L5<br/>8b"]
        B6["L6<br/>8b"]
        B7["L7<br/>8b"]
        B8["L8<br/>8b"]
        B9["L9<br/>8b"]
        BA["LA<br/>8b"]
        BB["LB<br/>8b"]
        BC["LC<br/>8b"]
        BD["LD<br/>8b"]
        BE["LE<br/>8b"]
        BF["LF<br/>8b"]
    end

    subgraph OPERATIONS["Operações"]
        ADD["state_add(s1, s2)"]
        XOR["state_xor(s1, s2)"]
        TENSOR["state_tensor(s1, s2)"]
        GET["state_get_layer(s, idx)"]
        SET["state_set_layer(s, idx, val)"]
    end

    SILSTATE --> OPERATIONS

    style B0 fill:#bbdefb
    style B1 fill:#bbdefb
    style B2 fill:#bbdefb
    style B3 fill:#bbdefb
    style B4 fill:#bbdefb
    style B5 fill:#ffe0b2
    style B6 fill:#ffe0b2
    style B7 fill:#ffe0b2
    style B8 fill:#c8e6c9
    style B9 fill:#c8e6c9
    style BA fill:#c8e6c9
    style BB fill:#e1bee7
    style BC fill:#e1bee7
    style BD fill:#f8bbd9
    style BE fill:#f8bbd9
    style BF fill:#f8bbd9
```

---

## 6. Estrutura de Crates do Workspace

```mermaid
flowchart TB
    subgraph WORKSPACE["Cargo Workspace"]
        subgraph CORE_CRATES["Core"]
            SIL_CORE["sil-core<br/>VM, State, VSP"]
            LIS_CORE["lis-core<br/>Compiler"]
        end

        subgraph CLI_CRATES["CLI & API"]
            LIS_CLI["lis-cli"]
            LIS_API["lis-api"]
            LIS_FMT["lis-format"]
            LIS_RT["lis-runtime"]
        end

        subgraph MODALITY_CRATES["Modalidades"]
            direction LR
            PHOTONIC["sil-photonic"]
            ACOUSTIC["sil-acoustic"]
            OLFACTORY["sil-olfactory"]
            GUSTATORY["sil-gustatory"]
            HAPTIC["sil-haptic"]
            ELECTRONIC["sil-electronic"]
            ACTUATOR["sil-actuator"]
            ENVIRONMENT["sil-environment"]
            NETWORK["sil-network"]
            GOVERNANCE["sil-governance"]
            COSMOPOLITAN["sil-cosmopolitan"]
            SWARM["sil-swarm"]
            QUANTUM["sil-quantum"]
            SUPERPOSITION["sil-superposition"]
            ENTANGLEMENT["sil-entanglement"]
            COLLAPSE["sil-collapse"]
        end

        subgraph INFRA_CRATES["Infraestrutura"]
            ORCH["sil-orchestration"]
            ENERGY["sil-energy"]
        end

        subgraph ML_CRATES["ML Library"]
            PAEBIRU["paebiru<br/>(46 arquivos .lis)"]
        end
    end

    LIS_CLI --> LIS_CORE
    LIS_API --> LIS_CORE
    LIS_CORE --> SIL_CORE

    ORCH --> SIL_CORE
    ORCH --> MODALITY_CRATES

    PAEBIRU --> LIS_CORE

    style CORE_CRATES fill:#e3f2fd
    style CLI_CRATES fill:#fff3e0
    style MODALITY_CRATES fill:#e8f5e9
    style INFRA_CRATES fill:#f3e5f5
    style ML_CRATES fill:#fce4ec
```

---

## 7. Ciclo Fechado de Execução

```mermaid
flowchart TB
    START((Início))

    subgraph SENSE["1. SENSE"]
        S1["Ler sensores"]
        S2["Atualizar L0-L4"]
    end

    subgraph PROCESS["2. PROCESS"]
        P1["Computar transformações"]
        P2["Atualizar L5-L7"]
    end

    subgraph ACTUATE["3. ACTUATE"]
        A1["Enviar comandos"]
        A2["Atualizar L6"]
    end

    subgraph NETWORK["4. NETWORK"]
        N1["Comunicar peers"]
        N2["Atualizar L8-LA"]
    end

    subgraph GOVERN["5. GOVERN"]
        G1["Aplicar regras"]
        G2["Consenso"]
    end

    subgraph EMERGE["6. EMERGE"]
        E1["Comportamento coletivo"]
        E2["Atualizar LB-LC"]
    end

    subgraph META["7. META"]
        M1["Reflexão"]
        M2["Checkpoint"]
        M3["Atualizar LD-LF"]
    end

    HALT{Parar?}
    END((Fim))

    START --> SENSE
    SENSE --> PROCESS
    PROCESS --> ACTUATE
    ACTUATE --> NETWORK
    NETWORK --> GOVERN
    GOVERN --> EMERGE
    EMERGE --> META
    META --> HALT
    HALT -->|Não| SENSE
    HALT -->|Sim| END

    style START fill:#4caf50
    style END fill:#f44336
    style HALT fill:#ff9800
```

---

## 8. Virtual Sil Processor (VSP)

```mermaid
flowchart TB
    subgraph VSP["VSP - Virtual Sil Processor"]
        subgraph FETCH["Fetch"]
            PC["Program Counter"]
            BYTECODE["Bytecode .silc"]
        end

        subgraph DECODE["Decode"]
            OPCODE["Opcode Decoder<br/>(70+ instruções)"]
        end

        subgraph EXECUTE["Execute"]
            direction LR
            INTERP["Interpreter<br/>(Universal)"]
            DYNASM["DynASM JIT<br/>(ARM64)"]
            CRANE["Cranelift JIT<br/>(Cross-platform)"]
        end

        subgraph MEMORY["Memory"]
            REGS["16 Registradores"]
            STACK["Stack"]
            HEAP["Heap"]
        end

        subgraph BACKEND["Backend Selector"]
            CPU_B["CPU Backend"]
            GPU_B["GPU Backend<br/>(wgpu)"]
            NPU_B["NPU Backend<br/>(CoreML)"]
        end
    end

    PC --> BYTECODE
    BYTECODE --> OPCODE
    OPCODE --> INTERP
    OPCODE --> DYNASM
    OPCODE --> CRANE

    EXECUTE --> MEMORY
    EXECUTE --> BACKEND

    style FETCH fill:#e3f2fd
    style DECODE fill:#fff3e0
    style EXECUTE fill:#e8f5e9
    style MEMORY fill:#f3e5f5
    style BACKEND fill:#fce4ec
```

---

## 9. Arquitetura Paebiru ML

```mermaid
flowchart TB
    subgraph PAEBIRU["Paebiru ML Library"]
        subgraph CORE_ML["core/"]
            BYTESIL_OPS["bytesil.lis<br/>Operações ByteSil"]
            STATE_OPS["state.lis<br/>Operações State"]
            ACTIVATIONS["activations.lis<br/>ReLU, Sigmoid, GELU, Softmax"]
            LOSS["loss.lis<br/>MSE, CrossEntropy, Cosine"]
            OPTIM["optim.lis<br/>SGD, Gradient Clipping"]
            LINALG["linalg.lis<br/>Dot, Norms"]
        end

        subgraph LAYERS_ML["layers/"]
            DENSE["dense.lis<br/>Fully Connected"]
            NORM["norm.lis<br/>LayerNorm, RMSNorm"]
            DROPOUT["dropout.lis<br/>Regularização"]
            RESIDUAL["residual.lis<br/>Skip Connections"]
        end

        subgraph ARCH_ML["arch/"]
            ATTENTION["attention.lis<br/>Scaled Dot-Product"]
            FFN["ffn.lis<br/>Feed Forward"]
            TRANSFORMER["transformer.lis<br/>Encoder/Decoder"]
            KAN["kan.lis<br/>Kolmogorov-Arnold"]
            SSM["ssm.lis<br/>Mamba/S4"]
            LNN["lnn.lis<br/>Liquid Neural Nets"]
            RNN["recurrent.lis<br/>RNN/LSTM/GRU"]
            SNN["snn.lis<br/>Spiking Neural Nets"]
        end

        subgraph DIST_ML["distributed/"]
            FEDAVG["fedavg.lis<br/>FedAvg Aggregation"]
            BYZANTINE["byzantine.lis<br/>Byzantine Robust"]
            COMPRESS["compress.lis<br/>Top-K, Quantization"]
            PRIVACY["privacy.lis<br/>Differential Privacy"]
        end

        subgraph EDGE_ML["edge/"]
            DEVICE["device.lis<br/>Device Detection"]
            RHO_SIL["rho_sil.lis<br/>Complexity Metric"]
            ROUTER["router.lis<br/>Auto-routing"]
        end
    end

    CORE_ML --> LAYERS_ML
    LAYERS_ML --> ARCH_ML
    ARCH_ML --> DIST_ML
    DIST_ML --> EDGE_ML

    style CORE_ML fill:#e3f2fd
    style LAYERS_ML fill:#fff3e0
    style ARCH_ML fill:#e8f5e9
    style DIST_ML fill:#f3e5f5
    style EDGE_ML fill:#fce4ec
```

---

## 10. Sistema de Traits

```mermaid
classDiagram
    class Sensor {
        <<trait>>
        +read_to_state() Result~SilState~
        +read_layer(layer: LayerId) Result~ByteSil~
    }

    class Processor {
        <<trait>>
        +process(input: SilState) Result~SilState~
    }

    class Actuator {
        <<trait>>
        +act(state: SilState) Result~ActuatorStatus~
    }

    class NetworkNode {
        <<trait>>
        +send(peer: PeerInfo, state: SilState) Result
        +recv() Result~SilState~
    }

    class Governor {
        <<trait>>
        +apply_rules(state: SilState) Result~SilState~
        +vote(proposal: Proposal) Result~Vote~
    }

    class SwarmAgent {
        <<trait>>
        +emit_signal(state: SilState) Result
        +aggregate(signals: Vec~SilState~) Result~SilState~
    }

    class QuantumState {
        <<trait>>
        +superpose(states: Vec~SilState~) Result~SilState~
        +measure() Result~SilState~
    }

    class Forkable {
        <<trait>>
        +fork() Result~SilState~
    }

    class Entangled {
        <<trait>>
        +entangle(other: SilState) Result
        +correlate() Result~SilState~
    }

    class Collapsible {
        <<trait>>
        +collapse() Result~SilState~
    }

    Sensor <|.. PhotonicSensor : L0
    Sensor <|.. AcousticSensor : L1
    Sensor <|.. HapticSensor : L4

    Processor <|.. ElectronicProcessor : L5
    Processor <|.. EnvironmentProcessor : L7

    Actuator <|.. MotorActuator : L6

    NetworkNode <|.. P2PNode : L8
    Governor <|.. GeopoliticalGovernor : L9
    Governor <|.. CosmopoliticalGovernor : LA

    SwarmAgent <|.. SwarmNode : LB
    QuantumState <|.. QuantumProcessor : LC

    Forkable <|.. SuperpositionState : LD
    Entangled <|.. EntanglementState : LE
    Collapsible <|.. CollapseState : LF
```

---

## 11. Backends de Hardware

```mermaid
flowchart TB
    subgraph SELECTION["Backend Selection"]
        AUTO["Auto-detect<br/>Capabilities"]
    end

    subgraph CPU_BACKEND["CPU Backend"]
        SCALAR["Scalar<br/>(Fallback)"]
        SIMD["SIMD<br/>(AVX2/NEON)"]
        MULTI["Multi-thread<br/>(Rayon)"]
    end

    subgraph GPU_BACKEND["GPU Backend"]
        WGPU["wgpu<br/>(WebGPU)"]
        COMPUTE["Compute Shaders<br/>(WGSL)"]
    end

    subgraph NPU_BACKEND["NPU Backend"]
        COREML["CoreML<br/>(Apple Silicon)"]
        ANE["Apple Neural Engine"]
    end

    subgraph JIT_BACKEND["JIT Compilation"]
        DYNASM["DynASM<br/>(ARM64 native)"]
        CRANELIFT["Cranelift<br/>(Cross-platform)"]
    end

    AUTO --> CPU_BACKEND
    AUTO --> GPU_BACKEND
    AUTO --> NPU_BACKEND
    AUTO --> JIT_BACKEND

    style CPU_BACKEND fill:#e3f2fd
    style GPU_BACKEND fill:#fff3e0
    style NPU_BACKEND fill:#e8f5e9
    style JIT_BACKEND fill:#f3e5f5
```

---

## 12. Roteamento Edge (ρ_sil)

```mermaid
flowchart TB
    INPUT["Input State"]

    CALC["Calcular ρ_sil(state)"]

    subgraph ZONES["Zonas Cromáticas"]
        ULTRA["UltraLocal<br/>ρ < 0.1<br/>🟢 Device"]
        LOCAL["Local<br/>0.1 ≤ ρ < 0.3<br/>🟡 Edge Node"]
        NEAR["Near<br/>0.3 ≤ ρ < 0.5<br/>🟠 Distributed"]
        FAR["Far<br/>0.5 ≤ ρ < 0.8<br/>🔴 Cloud"]
        HPC["HPC<br/>ρ ≥ 0.8<br/>🟣 Datacenter"]
    end

    DEVICE["Processar Local"]
    EDGE["Edge Node"]
    DISTRIBUTE["Mesh/Cluster"]
    CLOUD["Cloud Offload"]
    DATACENTER["HPC Cluster"]

    INPUT --> CALC
    CALC --> ULTRA
    CALC --> LOCAL
    CALC --> NEAR
    CALC --> FAR
    CALC --> HPC

    ULTRA --> DEVICE
    LOCAL --> EDGE
    NEAR --> DISTRIBUTE
    FAR --> CLOUD
    HPC --> DATACENTER

    style ULTRA fill:#c8e6c9
    style LOCAL fill:#fff9c4
    style NEAR fill:#ffe0b2
    style FAR fill:#ffcdd2
    style HPC fill:#e1bee7
```

---

## 13. Aprendizado Federado

```mermaid
flowchart TB
    subgraph CLIENTS["Clientes (Edge Devices)"]
        C1["Client 1<br/>Train Local"]
        C2["Client 2<br/>Train Local"]
        C3["Client 3<br/>Train Local"]
        C4["Client 4<br/>Train Local"]
    end

    subgraph PRIVACY["Camada de Privacidade"]
        DP["Differential Privacy<br/>add_gaussian_noise()"]
        COMPRESS["Compression<br/>top_k_sparsify()"]
        QUANT["Quantization<br/>quantize_8bit()"]
    end

    subgraph AGGREGATION["Agregação"]
        FEDAVG["FedAvg<br/>fedavg_4()"]
        BYZANTINE["Byzantine Robust<br/>krum_3(), trimmed_mean_5()"]
        WEIGHTED["Weighted<br/>fedavg_weighted_4()"]
    end

    subgraph GLOBAL["Modelo Global"]
        MODEL["Global Model<br/>Atualizado"]
    end

    C1 --> DP
    C2 --> DP
    C3 --> DP
    C4 --> DP

    DP --> COMPRESS
    COMPRESS --> QUANT
    QUANT --> AGGREGATION

    FEDAVG --> MODEL
    BYZANTINE --> MODEL
    WEIGHTED --> MODEL

    MODEL -->|"Broadcast"| CLIENTS

    style CLIENTS fill:#e3f2fd
    style PRIVACY fill:#f3e5f5
    style AGGREGATION fill:#e8f5e9
    style GLOBAL fill:#fff3e0
```

---

## 14. API REST (lis-api)

```mermaid
flowchart LR
    subgraph CLIENT["Cliente HTTP"]
        REQ["Request"]
    end

    subgraph MIDDLEWARE["Middleware"]
        AUTH["Auth<br/>X-API-Key"]
        RATE["Rate Limit<br/>10 req/s"]
        CORS["CORS<br/>(opcional)"]
    end

    subgraph ENDPOINTS["Endpoints"]
        COMPILE["/api/compile<br/>POST"]
        EXECUTE["/api/execute<br/>POST"]
        FORMAT["/api/format<br/>POST"]
        CHECK["/api/check<br/>POST"]
        INTRINSICS["/api/intrinsics<br/>GET"]
        INFO["/api/info<br/>GET"]
        HEALTH["/health<br/>GET"]
        DOCS["/docs<br/>Swagger UI"]
    end

    subgraph HANDLERS["Handlers"]
        H_COMP["compile_handler()"]
        H_EXEC["execute_handler()"]
        H_FMT["format_handler()"]
    end

    REQ --> AUTH
    AUTH --> RATE
    RATE --> CORS
    CORS --> ENDPOINTS

    COMPILE --> H_COMP
    EXECUTE --> H_EXEC
    FORMAT --> H_FMT

    style CLIENT fill:#e3f2fd
    style MIDDLEWARE fill:#fff3e0
    style ENDPOINTS fill:#e8f5e9
    style HANDLERS fill:#f3e5f5
```

---

## 15. Orchestrator

```mermaid
flowchart TB
    subgraph ORCHESTRATOR["Sil Orchestrator"]
        REGISTRY["Component Registry<br/>HashMap por tipo"]
        EVENTBUS["Event Bus<br/>Pub/Sub async"]
        PIPELINE["Pipeline<br/>Sense→Process→Act"]
        SCHEDULER["Scheduler<br/>Timing/Priority"]
    end

    subgraph COMPONENTS["Componentes Registrados"]
        SENSORS["Sensors<br/>(L0-L4)"]
        PROCESSORS["Processors<br/>(L5-L7)"]
        ACTUATORS["Actuators<br/>(L6)"]
        NETWORK["Network Nodes<br/>(L8-LA)"]
        GOVERNORS["Governors<br/>(L9-LA)"]
        SWARM["Swarm Agents<br/>(LB)"]
        QUANTUM["Quantum States<br/>(LC-LF)"]
    end

    subgraph EVENTS["Eventos"]
        STATE_CHANGE["StateChanged"]
        THRESHOLD["ThresholdReached"]
        ERROR["ErrorOccurred"]
        TICK["ClockTick"]
    end

    REGISTRY --> COMPONENTS
    PIPELINE --> COMPONENTS
    EVENTBUS --> EVENTS
    SCHEDULER --> PIPELINE

    style ORCHESTRATOR fill:#e3f2fd
    style COMPONENTS fill:#e8f5e9
    style EVENTS fill:#fff3e0
```

---

## 16. Fluxo de Compilação CLI

```mermaid
sequenceDiagram
    participant User
    participant CLI as lis-cli
    participant Resolver as ModuleResolver
    participant Lexer
    participant Parser
    participant TypeChecker
    participant Compiler
    participant Assembler
    participant VSP

    User->>CLI: lis run program.lis
    CLI->>Resolver: resolve_input()
    Resolver-->>CLI: [modules]

    loop Para cada módulo
        CLI->>Lexer: tokenize(source)
        Lexer-->>CLI: tokens
        CLI->>Parser: parse(tokens)
        Parser-->>CLI: AST
        CLI->>TypeChecker: check(AST)
        TypeChecker-->>CLI: validated AST
        CLI->>Compiler: compile(AST)
        Compiler-->>CLI: assembly
    end

    CLI->>Assembler: assemble(all_assembly)
    Assembler-->>CLI: bytecode (.silc)
    CLI->>VSP: load(bytecode)
    VSP->>VSP: run()
    VSP-->>CLI: SilState (resultado)
    CLI-->>User: Output
```

---

## 17. Arquitetura de Rede Neural (Forward Pass)

```mermaid
flowchart LR
    INPUT["Input<br/>State"]

    subgraph LAYER1["Layer 1"]
        D1["dense_forward()"]
        N1["layer_norm()"]
        A1["relu()"]
    end

    subgraph LAYER2["Layer 2"]
        D2["dense_forward()"]
        N2["layer_norm()"]
        A2["relu()"]
        DROP["dropout()"]
    end

    subgraph OUTPUT_LAYER["Output Layer"]
        D3["dense_forward()"]
        SOFT["softmax()"]
    end

    OUTPUT["Output<br/>State"]

    INPUT --> D1
    D1 --> N1
    N1 --> A1
    A1 --> D2
    D2 --> N2
    N2 --> A2
    A2 --> DROP
    DROP --> D3
    D3 --> SOFT
    SOFT --> OUTPUT

    style INPUT fill:#e3f2fd
    style OUTPUT fill:#e8f5e9
    style LAYER1 fill:#fff3e0
    style LAYER2 fill:#f3e5f5
    style OUTPUT_LAYER fill:#fce4ec
```

---

## 18. Transformer Block

```mermaid
flowchart TB
    INPUT["Input State"]

    subgraph ATTENTION["Multi-Head Attention"]
        QKV["Q, K, V Projection"]
        ATTN["scaled_dot_product_attention()"]
        PROJ["Output Projection"]
    end

    ADD1["+ Residual"]
    NORM1["layer_norm()"]

    subgraph FFN["Feed Forward Network"]
        FF1["dense_forward()"]
        GELU["gelu()"]
        FF2["dense_forward()"]
    end

    ADD2["+ Residual"]
    NORM2["layer_norm()"]

    OUTPUT["Output State"]

    INPUT --> QKV
    QKV --> ATTN
    ATTN --> PROJ
    PROJ --> ADD1
    INPUT --> ADD1
    ADD1 --> NORM1
    NORM1 --> FF1
    FF1 --> GELU
    GELU --> FF2
    FF2 --> ADD2
    NORM1 --> ADD2
    ADD2 --> NORM2
    NORM2 --> OUTPUT

    style ATTENTION fill:#e3f2fd
    style FFN fill:#fff3e0
```

---

## 19. Extensão VSCode

```mermaid
flowchart TB
    subgraph VSCODE["sil-vscode Extension"]
        subgraph SYNTAX["Syntax Support"]
            TM["TextMate Grammar<br/>(.lis, .sil)"]
            HIGHLIGHT["Syntax Highlighting"]
            THEMES["Themes"]
        end

        subgraph LSP_CLIENT["LSP Client"]
            LANG["LanguageClient"]
            DIAG["Diagnostics"]
            COMPLETE["Completions"]
            HOVER["Hover Info"]
        end

        subgraph DEBUG["Debugger (DAP)"]
            BREAKPOINTS["Breakpoints"]
            STEP["Step/Continue"]
            VARS["Variables View"]
            STACK["Call Stack"]
        end

        subgraph COMMANDS["Commands"]
            CMD_COMP["lis.compile"]
            CMD_RUN["lis.run"]
            CMD_FMT["lis.format"]
            CMD_NEW["lis.newProject"]
        end
    end

    subgraph EXTERNAL["External Processes"]
        LSP_SERVER["sil-lsp<br/>(Language Server)"]
        DEBUG_ADAPTER["vsp-debug<br/>(Debug Adapter)"]
        LIS_CLI["lis-cli"]
    end

    LSP_CLIENT <--> LSP_SERVER
    DEBUG <--> DEBUG_ADAPTER
    COMMANDS --> LIS_CLI

    style SYNTAX fill:#e3f2fd
    style LSP_CLIENT fill:#fff3e0
    style DEBUG fill:#e8f5e9
    style COMMANDS fill:#f3e5f5
```

---

## 20. Visão Geral Completa

```mermaid
mindmap
    root((SIL))
        LIS Language
            Lexer
            Parser
            Type System
            Compiler
        SIL Core
            ByteSil
            SilState
            VSP VM
            Traits
        Backends
            CPU/SIMD
            GPU/wgpu
            NPU/CoreML
            JIT/Cranelift
        16 Layers
            L0-L4 Perception
            L5-L7 Processing
            L8-LA Interaction
            LB-LC Emergence
            LD-LF Meta
        Paebiru ML
            Core Ops
            Layers
            Architectures
            Distributed
            Edge Routing
        Tools
            lis-cli
            lis-api
            sil-vscode
            sil-lsp
        Infrastructure
            Orchestration
            K8s
            Docker
```

---

## Referências

- [README.md](../README.md) - Visão geral do projeto
- [ARCHITECTURE.md](ARCHITECTURE.md) - Fundamentos filosóficos
- [LIS_SIL_DOCUMENTATION.md](LIS_SIL_DOCUMENTATION.md) - Documentação completa da linguagem
- [PERFORMANCE.md](PERFORMANCE.md) - Benchmarks e otimizações
- [PAEBIRU.md](PAEBIRU.md) - Documentação da biblioteca ML

---

*Diagramas gerados com Mermaid - SIL/LIS 2026*
