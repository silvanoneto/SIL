# LIS - Language for Intelligent Systems

**LIS** é uma linguagem de programação para modelagem de sistemas não-lineares, capazes de autocompilar informação e atuar com inteligência, escalonando toda a complexidade pro hardware com extrema eficiência e sucesso.

## 🌀 Filosofia

LIS é construída sobre o paradigma SIL (Superposition Intelligence Layer), mas oferece abstrações de alto nível para expressar:

- **Feedback loops e ciclos causais**: Recursão autopoiética como primitiva
- **Topologia e transformações contínuas**: Operações sobre espaços topológicos
- **Emergência e auto-organização**: Comportamentos emergentes de sistemas adaptativos
- **Metaprogramação reflexiva**: AST acessível como dados, runtime recompilation
- **Consciente de hardware**: Tipos que mapeiam para CPU/GPU/NPU/SIMD/Photonic

## 🎯 Objetivos

1. **Não-linearidade nativa**: Expressar sistemas não-lineares naturalmente
2. **Autocompilação**: Programas que se analisam, modificam e otimizam
3. **Escalonamento inteligente**: Compilação adapta-se ao hardware disponível
4. **Integração com SIL**: Compila para bytecode VSP (.silc)

## 📐 Arquitetura

```text
LIS Source (.lis)
    ↓ Lexer (logos)
Token Stream
    ↓ Parser (chumsky)
Abstract Syntax Tree
    ↓ Compiler
VSP Assembly (.sil)
    ↓ Assembler (sil-core)
Bytecode (.silc)
    ↓ VSP Runtime
Execution (CPU/GPU/NPU)
```

## 🚀 Exemplo

```lis
// Função simples
fn main() {
    let state = sense();           // Captura entrada sensorial (L0-L4)
    let processed = transform(state);  // Processa (L5-L7)
    act(processed);                // Atua (L5-L7)
}

// Transform com feedback
transform autopoietic(input: State) {
    let output = process(input);
    feedback output |> autopoietic;  // Ciclo fechado
    return output;
}

// Acesso a layers
fn layer_ops() {
    let state = sense();
    let photonic = state.L0;       // Acessa layer L0 (photonic)
    let quantum = state.LC;        // Acessa layer LC (quantum)
}

// Construção de estado
fn build_state() {
    let state = State {
        L0: (1.0, 0.0),            // Photonic (rho, theta)
        L1: (0.5, 1.57),           // Acoustic
        LF: (0.0, 0.0),            // Collapse
    };
}

// Operações complexas (log-polar)
fn complex_ops() {
    let z1 = (2.0, 1.57);          // e^2 * e^(i*π/2)
    let z2 = (1.0, 0.78);
    let product = z1 * z2;         // Multiplica em O(1)
    let conjugate = ~z1;           // Conjugado complexo
}

// Pipeline de transformações
fn pipeline() {
    let input = sense();
    let result = input
        |> normalize
        |> detect_patterns
        |> emerge;                 // Detecta emergência
}

// Hardware hints
@gpu
fn parallel_process(data: State) {
    // Executa na GPU automaticamente
    let result = batch_transform(data);
    return result;
}

@npu
fn classify(input: State) -> Int {
    // Executa no Neural Engine
    return neural_inference(input);
}

// Loops e controle
fn control_flow() {
    let x = 0;
    loop {
        x = x + 1;
        if x > 10 {
            break;
        }
    }
}
```

## 📚 Características da Linguagem

### Tipos Primitivos

- `ByteSil`: Valor complexo (log-polar encoding)
- `State`: Estado de 16 layers (L0-LF)
- `Layer(n)`: Layer específico (L0, L5, LC, etc.)
- `Int`, `Float`, `Bool`, `String`

### Operadores

#### Aritméticos
- `+`, `-`, `*`, `/`: Operações aritméticas
- `**`: Exponenciação
- `~`: Conjugado complexo
- `|x|`: Magnitude

#### Lógicos
- `&&`, `||`, `!`: Lógica booleana
- `==`, `!=`, `<`, `>`, `<=`, `>=`: Comparação

#### Layer Operations
- `^`: XOR entre layers
- `&`: AND bitwise
- `|`: OR bitwise

#### Pipeline
- `|>`: Pipe (aplica transform)

### Palavras-chave

- `fn`: Define função
- `transform`: Define transformação
- `type`: Alias de tipo
- `let`: Declaração de variável
- `return`: Retorna valor
- `if`, `else`: Condicional
- `loop`, `break`, `continue`: Loops
- `feedback`: Feedback loop (L(F) → L(0))
- `emerge`: Detecta emergência

### Hardware Hints

- `@cpu`: Força execução em CPU
- `@gpu`: Força execução em GPU
- `@npu`: Força execução em NPU
- `@simd`: Usa instruções SIMD
- `@photonic`: Hint para hardware fotônico (futuro)

## 🔧 Uso

### Como biblioteca Rust

```rust
use lis_core::{compile, Lexer, Parser, Compiler};

fn main() {
    let source = r#"
        fn main() {
            let x = 42;
        }
    "#;

    // Compilação completa
    let assembly = compile(source).unwrap();
    println!("{}", assembly);

    // Ou passo a passo
    let tokens = Lexer::new(source).tokenize().unwrap();
    let ast = Parser::new(tokens).parse().unwrap();
    let mut compiler = Compiler::new();
    let asm = compiler.compile(&ast).unwrap();
}
```

### CLI (futuro)

```bash
# Compilar para assembly
lis compile program.lis -o program.sil

# Compilar para bytecode
lis build program.lis -o program.silc

# Executar diretamente
lis run program.lis

# REPL interativo
lis repl
```

## 🏗️ Implementação

### Estrutura do Projeto

```
lis-core/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs          # API pública
    ├── error.rs        # Tipos de erro
    ├── ast/
    │   └── mod.rs      # Abstract Syntax Tree
    ├── lexer.rs        # Tokenização (logos)
    ├── parser.rs       # Parser (recursive descent)
    └── compiler.rs     # Gerador de código VSP
```

### Fases de Compilação

1. **Lexer**: `source` → `Vec<Token>`
   - Tokenização com `logos`
   - Suporta keywords, identifiers, operators, literals
   - Comments (linha e bloco)

2. **Parser**: `Vec<Token>` → `AST`
   - Recursive descent parser
   - Precedência de operadores
   - Validação sintática

3. **Compiler**: `AST` → `VSP Assembly`
   - Alocação de registradores (R0-RF)
   - Geração de código VSP
   - Labels e control flow

4. **Assembler** (sil-core): `Assembly` → `Bytecode`
   - Integração com sil-core existente
   - Produz .silc executável

## 🧪 Status de Implementação

### ✅ Implementado

- [x] Estrutura do projeto
- [x] Lexer completo com logos
- [x] Parser para subset da linguagem
- [x] AST com tipos principais
- [x] Compiler básico para VSP assembly
- [x] Suporte a funções, let, arithmetic
- [x] Suporte a control flow (if, loop)
- [x] Operações em layers (L0-LF)
- [x] Testes unitários

### 🚧 Em Progresso

- [ ] Integração com sil-core assembler
- [ ] Sistema de tipos completo
- [ ] Type checking
- [ ] Análise semântica
- [ ] Standard library

### 📋 Futuro

- [ ] Metaprogramação reflexiva
- [ ] Runtime recompilation
- [ ] Otimizações adaptativas
- [ ] Scheduling distribuído
- [ ] CLI tool
- [ ] LSP (Language Server Protocol)
- [ ] Debugger integration
- [ ] Self-hosting (compiler escrito em LIS)

## 🤝 Contribuindo

LIS é parte do projeto , licenciado sob AGPL-3.0. Contribuições são bem-vindas!

## 🔗 Relacionado

- **SIL**: Design pattern e paradigma de computação
- **VSP**: Virtual Sil Processor (bytecode VM)
- **sil-core**: Runtime e infraestrutura base

## 📖 Documentação

Para mais detalhes sobre o paradigma SIL:
- [SIL_CODE.md](../SIL/SIL_CODE.md) - Design pattern
- [SIL_VSP.md](../SIL/SIL_VSP.md) - Virtual machine
- [SIL_ARCHITECTURE.md](../SIL/SIL_ARCHITECTURE.md) - Arquitetura

---

**"We are the swarm. We are the vapor. We are the edge."**

理信 (Lǐxìn) - Where logic and information are indistinguishable.
