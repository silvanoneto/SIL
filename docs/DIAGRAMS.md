# 📊 Diagramas da Arquitetura 

Visualizações da arquitetura do ecossistema SIL usando Mermaid.

---

## 🌀 1. Visão Geral do Sistema

```mermaid
graph TB
    subgraph "PERCEPÇÃO (L0-L4)"
        L0[L0: Fotônica<br/>Camera, Light]
        L1[L1: Acústica<br/>Microphone, Audio]
        L2[L2: Olfativa<br/>Gas Sensors]
        L3[L3: Gustativa<br/>Taste Sensors]
        L4[L4: Háptica<br/>Pressure, Touch]
    end

    subgraph "PROCESSAMENTO (L5-L7)"
        L5[L5: Eletrônica<br/>CPU/GPU/NPU]
        L6[L6: Atuação<br/>Motors, Servos]
        L7[L7: Ambiente<br/>Climate, Fusion]
    end

    subgraph "INTERAÇÃO (L8-LA)"
        L8[L8: Rede P2P<br/>Mesh Network]
        L9[L9-LA: Governança<br/>Voting, Consensus]
    end

    subgraph "EMERGÊNCIA & META (LB-LF)"
        LB[LB: Enxame<br/>Swarm Intelligence]
        LC[LC: Quântico<br/>Superposition]
        LD[LD: Fork/Merge<br/>State Branching]
        LE[LE: Emaranhamento<br/>Entanglement]
        LF[LF: Colapso<br/>Checkpoint/Reset]
    end

    L0 --> L5
    L1 --> L5
    L2 --> L5
    L3 --> L5
    L4 --> L5

    L5 --> L6
    L5 --> L7
    L7 --> L5

    L5 --> L8
    L8 --> L9

    L9 --> LB
    LB --> LC
    LC --> LD
    LD --> LE
    LE --> LF

    LF -->|Feedback Loop| L0

    style L0 fill:#ff6b6b
    style L1 fill:#ffd93d
    style L2 fill:#6bcf7f
    style L3 fill:#4ecdc4
    style L4 fill:#a29bfe
    style L5 fill:#fd79a8
    style L6 fill:#fdcb6e
    style L7 fill:#55efc4
    style L8 fill:#74b9ff
    style L9 fill:#a29bfe
    style LB fill:#fd79a8
    style LC fill:#fab1a0
    style LD fill:#ff7675
    style LE fill:#e17055
    style LF fill:#d63031
```

---

## 🎭 2. Arquitetura do Orchestrator

```mermaid
graph TB
    subgraph "Orchestrator"
        ORCH[Orchestrator Core]

        subgraph "Component Registry"
            SENS[Sensors<br/>L0-L4]
            PROC[Processors<br/>L5, L7]
            ACT[Actuators<br/>L6]
            NET[Network Nodes<br/>L8]
            GOV[Governors<br/>L9-LA]
            SWARM[Swarm Agents<br/>LB]
        end

        subgraph "Event Bus"
            EB[Event Bus<br/>Pub/Sub]
            EH[Event History<br/>Circular Buffer]
            FILT[Filters<br/>Layer/Type/Source]
        end

        subgraph "Pipeline Executor"
            PIPE[Pipeline<br/>7 Stages]
            EXEC[Component Executor<br/>Per Stage]
        end

        subgraph "Scheduler"
            SCHED[Scheduler<br/>Rate Control]
            TIMER[Timer<br/>High Resolution]
            METRICS[Metrics<br/>Min/Max/Avg]
        end

        STATE[Global State<br/>Arc&lt;RwLock&lt;SilState&gt;&gt;]
    end

    ORCH --> SENS
    ORCH --> PROC
    ORCH --> ACT
    ORCH --> NET
    ORCH --> GOV
    ORCH --> SWARM

    ORCH --> EB
    EB --> EH
    EB --> FILT

    ORCH --> PIPE
    PIPE --> EXEC

    ORCH --> SCHED
    SCHED --> TIMER
    SCHED --> METRICS

    ORCH --> STATE

    EXEC -.reads.-> STATE
    EXEC -.writes.-> STATE
    EXEC -.emits.-> EB

    style ORCH fill:#4ecdc4,stroke:#333,stroke-width:4px
    style STATE fill:#ff6b6b,stroke:#333,stroke-width:2px
    style EB fill:#ffd93d
    style PIPE fill:#6bcf7f
    style SCHED fill:#a29bfe
```

---

## 🔄 3. Pipeline de Execução

```mermaid
stateDiagram-v2
    [*] --> Sense

    Sense --> Process: Sensors Read<br/>(L0-L4)
    Process --> Actuate: Transform State<br/>(L5, L7)
    Actuate --> Network: Motor Commands<br/>(L6)
    Network --> Govern: P2P Messages<br/>(L8)
    Govern --> Swarm: Consensus<br/>(L9-LA)
    Swarm --> Quantum: Flocking<br/>(LB)
    Quantum --> Sense: Collapse<br/>(LC-LF)

    note right of Sense
        Read all sensors
        Update L0-L4
    end note

    note right of Process
        Execute processors
        Transform state
    end note

    note right of Actuate
        Send commands
        to motors/servos
    end note

    note right of Quantum
        Superposition
        Fork/Merge
        Collapse
    end note
```

---

## 📡 4. Padrões de Comunicação

```mermaid
graph LR
    subgraph "Padrão 1: SilState (Síncrono)"
        S1[Sensor] -->|read_to_state| ST1[SilState]
        ST1 -->|execute| P1[Processor]
        P1 -->|result| ST2[SilState]
    end

    subgraph "Padrão 2: Channels (Assíncrono)"
        S2[Sensor] -->|send| CH[Channel<br/>mpsc]
        CH -->|recv| P2[Processor]
    end

    subgraph "Padrão 3: Events (Pub/Sub)"
        PUB[Publisher] -->|emit| EB[Event Bus]
        EB -->|filter| SUB1[Subscriber 1]
        EB -->|filter| SUB2[Subscriber 2]
        EB -->|filter| SUB3[Subscriber N]
    end

    subgraph "Padrão 4: Network (P2P)"
        N1[Node A] -->|broadcast| MESH[Mesh Network]
        MESH -->|receive| N2[Node B]
        MESH -->|receive| N3[Node C]
    end

    style ST1 fill:#ff6b6b
    style ST2 fill:#ff6b6b
    style CH fill:#ffd93d
    style EB fill:#6bcf7f
    style MESH fill:#4ecdc4
```

---

## 🧮 5. ByteSil: Representação Log-Polar

```mermaid
graph TB
    subgraph "Representação Cartesiana (Tradicional)"
        C1[z = a + bi]
        C2[Multiplicação:<br/>4 mults + 2 adds<br/>Custo: Alto]
    end

    subgraph "Representação Log-Polar (SIL)"
        LP1["ByteSil(ρ, θ)<br/>ρ ∈ [0,15] (4 bits)<br/>θ ∈ [0,255] (8 bits)"]
        LP2["z = e^ρ · e^(iθ·2π/256)"]
        LP3["Multiplicação:<br/>(ρ₁+ρ₂, θ₁+θ₂)<br/>2 adds<br/>Custo: O(1)"]
    end

    C1 --> C2
    LP1 --> LP2
    LP2 --> LP3

    LP3 -.->|200× mais rápido| C2

    style LP1 fill:#ff6b6b
    style LP2 fill:#ffd93d
    style LP3 fill:#6bcf7f,stroke:#333,stroke-width:3px
    style C2 fill:#ddd
```

---

## 🔀 6. Computação Quântica (Fork/Merge)

```mermaid
graph TB
    START[Estado Inicial<br/>SilState]

    subgraph "Superposição"
        SP1[Estado 1<br/>peso: 0.5]
        SP2[Estado 2<br/>peso: 0.3]
        SP3[Estado 3<br/>peso: 0.2]
        SUPER[Estado Superposto<br/>Σ(wᵢ·sᵢ)]
    end

    subgraph "Fork/Merge"
        SUPER2[Estado Superposto]
        FORK1[Fork 1<br/>Estratégia A]
        FORK2[Fork 2<br/>Estratégia B]
        MERGE[Merge<br/>Max/Min/Avg/XOR]
        RESULT[Estado Mesclado]
    end

    subgraph "Colapso"
        RESULT2[Estado Final]
        COLLAPSE[Collapse<br/>seed: 42]
        COLLAPSED[Estado Colapsado<br/>Escolhe 1 estado]
    end

    START --> SP1
    START --> SP2
    START --> SP3

    SP1 --> SUPER
    SP2 --> SUPER
    SP3 --> SUPER

    SUPER --> SUPER2

    SUPER2 --> FORK1
    SUPER2 --> FORK2

    FORK1 --> MERGE
    FORK2 --> MERGE

    MERGE --> RESULT

    RESULT --> RESULT2
    RESULT2 --> COLLAPSE
    COLLAPSE --> COLLAPSED

    style START fill:#a29bfe
    style SUPER fill:#fab1a0
    style SUPER2 fill:#fab1a0
    style MERGE fill:#ff7675
    style COLLAPSED fill:#d63031,stroke:#333,stroke-width:3px
```

---

## 🐝 7. Swarm Intelligence (Flocking)

```mermaid
graph TB
    subgraph "Agent Local"
        A1[Agente 1<br/>Estado Local]
        A2[Agente 2<br/>Estado Local]
        A3[Agente 3<br/>Estado Local]
        AN[Agente N<br/>Estado Local]
    end

    subgraph "Spatial Partitioning"
        GRID[Grid Espacial<br/>Cell Size: r]
        NEIGHBORS[Vizinhos Próximos<br/>k ≈ 30-50]
    end

    subgraph "Comportamento Emergente"
        ALIGN[Alignment<br/>Mesma direção]
        COHESION[Cohesion<br/>Centro de massa]
        SEPARATION[Separation<br/>Evitar colisão]
        FLOCK[Flocking<br/>Comportamento Global]
    end

    A1 --> GRID
    A2 --> GRID
    A3 --> GRID
    AN --> GRID

    GRID --> NEIGHBORS

    NEIGHBORS --> ALIGN
    NEIGHBORS --> COHESION
    NEIGHBORS --> SEPARATION

    ALIGN --> FLOCK
    COHESION --> FLOCK
    SEPARATION --> FLOCK

    FLOCK -.update.-> A1
    FLOCK -.update.-> A2
    FLOCK -.update.-> A3
    FLOCK -.update.-> AN

    style GRID fill:#ffd93d
    style FLOCK fill:#6bcf7f,stroke:#333,stroke-width:3px
    style ALIGN fill:#ff6b6b
    style COHESION fill:#4ecdc4
    style SEPARATION fill:#a29bfe
```

---

## 🌐 8. Rede P2P Mesh

```mermaid
graph TB
    subgraph "Topologia Mesh"
        N1((Node 1<br/>Sensor))
        N2((Node 2<br/>Sensor))
        N3((Node 3<br/>Sensor))
        N4((Node 4<br/>Actuator))
        N5((Node 5<br/>Gateway))
    end

    subgraph "Governança Distribuída"
        PROP[Proposta<br/>Alerta Global?]
        V1[Voto: Sim]
        V2[Voto: Sim]
        V3[Voto: Não]
        V4[Voto: Sim]
        V5[Voto: Sim]
        CONS[Consenso<br/>4/5 = Aceito]
    end

    N1 <-->|broadcast| N2
    N1 <-->|broadcast| N3
    N2 <-->|broadcast| N3
    N2 <-->|broadcast| N4
    N3 <-->|broadcast| N5
    N4 <-->|broadcast| N5
    N1 <-->|shortcut| N4
    N1 <-->|shortcut| N5

    N1 --> PROP

    N1 --> V1
    N2 --> V2
    N3 --> V3
    N4 --> V4
    N5 --> V5

    V1 --> CONS
    V2 --> CONS
    V3 --> CONS
    V4 --> CONS
    V5 --> CONS

    CONS -.ação global.-> N1
    CONS -.ação global.-> N2
    CONS -.ação global.-> N3
    CONS -.ação global.-> N4
    CONS -.ação global.-> N5

    style N1 fill:#ff6b6b
    style N2 fill:#ffd93d
    style N3 fill:#6bcf7f
    style N4 fill:#4ecdc4
    style N5 fill:#a29bfe
    style CONS fill:#e17055,stroke:#333,stroke-width:3px
```

---

## 🔧 9. VSP (Virtual Sil Processor)

```mermaid
graph TB
    subgraph "LIS Source"
        LIS[LIS Code<br/>.lis]
    end

    subgraph "Compilation Pipeline"
        LEX[Lexer<br/>Tokens]
        PARSE[Parser<br/>AST]
        COMP[Compiler<br/>Code Gen]
        ASM[Assembly<br/>.sil]
        BINARY[Bytecode<br/>.silc]
    end

    subgraph "VSP Runtime"
        REG[Registers<br/>R0-RF]
        STATE[State<br/>L0-LF]
        MEM[Memory<br/>Program]

        subgraph "Backends"
            CPU[CPU<br/>Interpreted]
            GPU[GPU<br/>WGPU Batch]
            NPU[NPU<br/>Neural Accel]
        end
    end

    LIS --> LEX
    LEX --> PARSE
    PARSE --> COMP
    COMP --> ASM
    ASM --> BINARY

    BINARY --> REG
    BINARY --> STATE
    BINARY --> MEM

    REG --> CPU
    STATE --> CPU
    MEM --> CPU

    REG --> GPU
    STATE --> GPU
    MEM --> GPU

    REG --> NPU
    STATE --> NPU
    MEM --> NPU

    style LIS fill:#a29bfe
    style ASM fill:#ffd93d
    style BINARY fill:#ff6b6b
    style CPU fill:#6bcf7f
    style GPU fill:#4ecdc4
    style NPU fill:#fd79a8
```

---

## 📊 10. Performance (Latency Budget)

```mermaid
gantt
    title Pipeline Latency Budget (100 Hz = 10ms)
    dateFormat X
    axisFormat %L

    section Stages
    Sense (L0-L4)     :done, s1, 0, 50
    Process (L5,L7)   :done, s2, 50, 150
    Actuate (L6)      :done, s3, 150, 170
    Network (L8)      :done, s4, 170, 180
    Govern (L9-LA)    :done, s5, 180, 185
    Swarm (LB)        :done, s6, 185, 187
    Quantum (LC-LF)   :done, s7, 187, 190
    Orchestrator      :done, s8, 190, 190

    section Budget
    Slack (98.1%)     :crit, slack, 190, 10000
```

---

## 🚀 11. Roadmap de Desenvolvimento

```mermaid
timeline
    title Development Roadmap

    2026-Q1 : Core + Traits Fundamentais
           : sil-network + sil-governance
           : Percepção (L0-L4)
           : Processamento (L5-L7)

    2026-Q2 : Emergência & Meta (LB-LF)
           : sil-orchestration completo
           : LIS language + compiler
           : JSIL format

    2026-Q3 : VSP JIT compilation
           : GPU backend (WGPU)
           : Python/JavaScript bindings
           : Documentation v1.0

    2026-Q4 : NPU backend (CoreML)
           : Distributed orchestration
           : Edge deployment
           : Version 1.0 Release
```

---

## 📈 12. Scaling Behavior

```mermaid
xychart-beta
    title "ByteSil Operations: O(1) Constant Time"
    x-axis [1, 10, 100, 1000, 10000, 100000]
    y-axis "Time (ns)" 0 --> 3
    line [1.21, 1.22, 1.21, 1.23, 1.22, 1.21]
```

```mermaid
xychart-beta
    title "Swarm Scaling: Spatial Partitioning vs Naive"
    x-axis [10, 50, 100, 500, 1000, 5000, 10000]
    y-axis "Time (µs)" 0 --> 3000
    line "Naive O(N×16)" [28, 132, 261, 1320, 2610, 13050, 26100]
    line "Spatial O(k×16)" [14, 15, 15, 15, 16, 16, 17]
```

---

## 🎯 Como Usar Estes Diagramas

### Renderizar no GitHub

Os diagramas Mermaid são renderizados automaticamente no GitHub:
- Abra este arquivo no GitHub
- Os diagramas aparecem como gráficos interativos

### Renderizar Localmente

1. **VSCode**: Instale extensão "Markdown Preview Mermaid Support"
2. **CLI**: Use `mmdc` (mermaid-cli)
   ```bash
   npm install -g @mermaid-js/mermaid-cli
   mmdc -i DIAGRAMS.md -o diagrams.pdf
   ```

### Exportar para Imagens

```bash
# SVG
mmdc -i DIAGRAMS.md -o diagrams/ -t svg

# PNG
mmdc -i DIAGRAMS.md -o diagrams/ -t png
```

---

## 📚 Recursos

- [Mermaid Documentation](https://mermaid.js.org/)
- [ARCHITECTURE.md](ARCHITECTURE.md) — Arquitetura completa (texto)
- [EXAMPLES.md](EXAMPLES.md) — Casos de uso práticos
- [PERFORMANCE.md](PERFORMANCE.md) — Benchmarks detalhados

---

**⧑** *Uma imagem vale mil palavras. Um diagrama vale mil commits.*
