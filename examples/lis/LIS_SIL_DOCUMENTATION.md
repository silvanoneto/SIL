# LIS & SIL Language Documentation

> **LIS** (Language for Intelligent Systems) e **SIL** (Signal Intermediate Language) - Linguagens para sistemas não-lineares e processamento de sinais complexos.

---

## Índice

1. [Visão Geral](#1-visão-geral)
2. [Arquitetura de Compilação](#2-arquitetura-de-compilação)
3. [Estrutura de Projeto](#3-estrutura-de-projeto)
4. [Sistema de Módulos](#4-sistema-de-módulos)
5. [Lexer e Tokens](#5-lexer-e-tokens)
6. [Sistema de Tipos](#6-sistema-de-tipos)
7. [Sintaxe LIS](#7-sintaxe-lis)
8. [Expressões](#8-expressões)
9. [Statements](#9-statements)
10. [Funções Intrínsecas](#10-funções-intrínsecas)
11. [Linguagem SIL (Assembly VSP)](#11-linguagem-sil-assembly-vsp)
12. [Tratamento de Erros](#12-tratamento-de-erros)
13. [Integração com Rust](#13-integração-com-rust)
14. [CLI - Linha de Comando](#14-cli---linha-de-comando)
15. [Exemplos Práticos](#15-exemplos-práticos)

---

## 1. Visão Geral

### 1.1 Filosofia

**LIS** é uma linguagem de programação projetada para modelagem de sistemas não-lineares. Seus princípios fundamentais são:

| Princípio | Descrição |
|-----------|-----------|
| **Não-linear por design** | Suporte nativo a feedback loops, topologia e emergência |
| **Auto-compilável** | Metaprogramação reflexiva e otimização adaptativa |
| **Hardware-aware** | Sistema de tipos que reflete o substrato computacional (CPU/GPU/NPU) |
| **Edge-native** | Execução distribuída para swarm computing |

### 1.2 ByteSil - A Unidade Fundamental

O `ByteSil` é a unidade computacional fundamental de LIS:

- **8 bits** representando um número complexo em coordenadas **log-polar**
- **4 bits signed** para magnitude (rho): -8 a +7 (logaritmo da magnitude)
- **4 bits unsigned** para fase (theta): 0-15 mapeando 0 a 2π radianos
- Permite operações complexas eficientes em hardware restrito

**Importante:** A VM SIL **NÃO** usa tipos tradicionais como i16/i32/i64/i128 ou f16/f32/f64/f128. Internamente, **todos os valores** são representados como ByteSil em coordenadas log-polares:

- `Int` em LIS → instrução `MOVI` com valor imediato
- `Float` em LIS → instrução `COMPLEX` com rho/theta escalado (×1000)

### 1.3 State - O Container de 16 Camadas

O `State` é uma estrutura de 16 camadas (L0-LF), cada uma contendo um `ByteSil`:

```
L0-L3    Camada de Percepção (input sensorial)
L4       Fronteira de Percepção
L5-L7    Camada de Processamento (computação)
L8-LA    Camada de Interação (comunicação)
LB-LC    Camada de Emergência (comportamento de swarm)
LD-LF    Camada Meta (reflexão/checkpoint)
```

### 1.4 Modos SIL

O modo SIL determina quantas camadas do State são **ativas**:

| Modo   | Camadas Ativas | Bits Totais |
|--------|----------------|-------------|
| Sil8   | 1 (L0)         | 8           |
| Sil16  | 2 (L0-L1)      | 16          |
| Sil32  | 4 (L0-L3)      | 32          |
| Sil64  | 8 (L0-L7)      | 64          |
| Sil128 | 16 (L0-LF)     | 128         |

**Nota:** O modo afeta apenas o número de camadas consideradas nas operações, **não** a largura de tipos primitivos. O compilador LIS atualmente emite sempre `.mode SIL-128`.

#### Promoção de Modo

Quando um valor de modo menor é usado em contexto de modo maior, camadas extras são preenchidas com:
- **Vacuum** (zeros) para dados
- **Neutral** (1, 0) para multiplicadores

#### Demoção de Modo

Quando um State de modo maior precisa ser reduzido:

| Estratégia | Descrição |
|------------|-----------|
| Truncate   | Ignora camadas superiores |
| XOR        | XOR bit-a-bit de todas camadas |
| Average    | Média das camadas (domínio complexo) |
| Max        | Máxima magnitude entre camadas |

---

## 2. Arquitetura de Compilação

```
┌─────────────────┐
│  LIS Source     │  (.lis)
│  Código Fonte   │
└────────┬────────┘
         │ lexer
         ▼
┌─────────────────┐
│  Token Stream   │
│  Fluxo de Tokens│
└────────┬────────┘
         │ parser
         ▼
┌─────────────────┐
│  AST            │  (Abstract Syntax Tree)
│  Árvore Sintática│
└────────┬────────┘
         │ type checker + compiler
         ▼
┌─────────────────┐
│  VSP Assembly   │  (.sil)
│  Assembly SIL   │
└────────┬────────┘
         │ assembler (sil-core)
         ▼
┌─────────────────┐
│  Bytecode       │  (.silc)
│  Código Binário │
└────────┬────────┘
         │ VSP runtime
         ▼
┌─────────────────┐
│  Execution      │
│  Execução       │
└─────────────────┘
```

---

## 3. Estrutura de Projeto

### 3.1 Visão Geral

LIS suporta dois modos de operação:

| Modo | Descrição |
|------|-----------|
| **Standalone** | Arquivo `.lis` único, sem dependências |
| **Projeto** | Múltiplos arquivos com `lis.toml` |

### 3.2 Estrutura de Diretórios

```
meu-projeto/
├── lis.toml                # Manifest do projeto
├── src/
│   ├── main.lis            # Ponto de entrada
│   ├── lib.lis             # Biblioteca (opcional)
│   └── utils/
│       ├── mod.lis         # Módulo utils
│       └── math.lis        # Submódulo math
├── libs/                   # Dependências locais
│   └── neural/
│       ├── lis.toml
│       └── src/
│           └── lib.lis
└── target/                 # Output de compilação
    ├── debug/
    │   └── meu-projeto.sil
    └── release/
        └── meu-projeto.silc
```

### 3.3 Manifest: lis.toml

O arquivo `lis.toml` define a configuração do projeto:

```toml
[package]
name = "meu-projeto"
version = "2026.1.16"
authors = ["Nome <email@example.com>"]
description = "Descrição do projeto"
license = "MIT"
repository = "https://github.com/silvanoneto/"

[dependencies]
# Dependências locais (caminho relativo)
utils = { path = "./libs/utils" }
neural = { path = "../shared/neural" }

# Dependências de registry (futuro)
# math = "0.2.3"

[build]
entry = "src/main.lis"      # Ponto de entrada (padrão)
target_mode = "SIL-128"     # Modo SIL (padrão)
output = "target"           # Diretório de saída (padrão)
debug = false               # Incluir info de debug
```

### 3.4 Seções do Manifest

#### [package]

| Campo | Tipo | Obrigatório | Descrição |
|-------|------|-------------|-----------|
| `name` | String | Sim | Nome do pacote |
| `version` | String | Sim | Versão semântica |
| `authors` | Array | Não | Lista de autores |
| `description` | String | Não | Descrição |
| `license` | String | Não | Identificador SPDX |
| `repository` | String | Não | URL do repositório |

#### [dependencies]

Formatos suportados:

```toml
# Caminho local
utils = { path = "./libs/utils" }

# Com opções
neural = { path = "./neural", optional = true }

# Git (futuro)
# math = { git = "https://github.com/silvanoneto/", branch = "main" }
```

#### [build]

| Campo | Padrão | Descrição |
|-------|--------|-----------|
| `entry` | `src/main.lis` | Arquivo de entrada |
| `target_mode` | `SIL-128` | Modo SIL alvo |
| `output` | `target` | Diretório de saída |
| `debug` | `false` | Incluir debug info |

---

## 4. Sistema de Módulos

### 4.1 Use Statements

Importar módulos e símbolos:

```lis
// Import de módulo local (relativo a src/)
use utils::math;

// Import de dependência (definida em lis.toml)
use neural::layers;

// Import com alias
use utils::math as m;

// Import de itens específicos
use neural::layers::{Dense, Conv2D};

// Re-export público
pub use utils::math;
```

### 4.2 Declaração de Módulos

```lis
// Declara submódulo (carrega utils/mod.lis ou utils.lis)
mod utils;

// Módulo público
pub mod utils;
```

### 4.3 Visibilidade

Por padrão, todos os itens são **privados**. Use `pub` para exportar:

```lis
// Função privada (apenas neste módulo)
fn helper() {
    // ...
}

// Função pública (exportada)
pub fn process(x: Int) -> Int {
    return helper() + x;
}

// Transform público
pub transform normalize(s: State) -> State {
    return state_normalize(s);
}

// Type alias público
pub type Vector = State;
```

### 4.4 Resolução de Módulos

O resolver procura módulos na seguinte ordem:

1. **Dependências do manifest**: Se o primeiro segmento do path corresponde a uma dependência em `lis.toml`
2. **Módulos locais**: Relativo a `src/`
   - `use foo::bar` → `src/foo/bar.lis` ou `src/foo/bar/mod.lis`
3. **Relativo ao arquivo atual**: Se não encontrado em `src/`

### 4.5 Interoperabilidade com Rust (FFI)

Declarar funções externas implementadas em Rust:

```lis
// Declaração de função externa
extern fn my_rust_function(a: Int, b: Float) -> State;

// Uso normal
fn main() {
    let result = my_rust_function(42, 3.14);
    trace_state(result);
}
```

As funções `extern` são mapeadas para a instrução `SYSCALL` da VM, permitindo:

- Chamadas diretas a código Rust sem overhead de interpretação
- Extensibilidade sem modificar lis-core
- Type safety verificado em compile-time

### 4.6 Exemplo de Projeto Multi-Arquivo

**lis.toml:**
```toml
[package]
name = "neural-app"
version = "2026.1.16"

[dependencies]
layers = { path = "./libs/layers" }
```

**src/main.lis:**
```lis
use layers::dense;
use utils::math;

mod utils;

fn main() {
    let input = state_neutral();
    let weights = state_neutral();
    let output = dense::forward(input, weights);
    trace_state(output);
}
```

**src/utils/mod.lis:**
```lis
pub mod math;

pub fn log(msg: String) {
    print_string(msg);
}
```

**src/utils/math.lis:**
```lis
pub fn add(a: Int, b: Int) -> Int {
    return a + b;
}

pub fn multiply(a: Int, b: Int) -> Int {
    return a * b;
}
```

---

## 5. Lexer e Tokens

### 5.1 Palavras-chave

| Categoria | Palavras-chave |
|-----------|----------------|
| Definições | `fn`, `transform`, `type` |
| Variáveis | `let`, `return` |
| Controle de fluxo | `if`, `else`, `loop`, `break`, `continue` |
| Literais | `true`, `false` |
| Avançado | `feedback`, `emerge` |
| Módulos | `use`, `mod`, `pub`, `as`, `extern` |

### 5.2 Tipos Built-in

| Tipo | Descrição | Representação Interna |
|------|-----------|----------------------|
| `Int` | Inteiro (i64 no compilador) | Instrução MOVI com imediato |
| `Float` | Ponto flutuante (f64 no compilador) | COMPLEX com rho/theta escalado |
| `Bool` | Booleano | ByteSil (false=NULL, true=ONE) |
| `String` | Cadeia de caracteres | Referência a pool de strings |
| `ByteSil` | Complexo de 8 bits (log-polar) | 4 bits rho + 4 bits theta |
| `State` | Estado de 16 camadas | 16 x ByteSil = 128 bits |

> **Nota:** A VM usa representação log-polar unificada. Não existem tipos i16/i32/i64/i128 ou f16/f32/f64/f128 tradicionais na VM.

### 5.3 Hardware Hints

Anotações para direcionar execução a hardware específico:

```lis
@cpu      // Execução em CPU
@gpu      // Execução em GPU
@npu      // Neural Processing Unit
@simd     // Operações SIMD
@photonic // Computação fotônica
```

### 5.4 Referências de Camada

```lis
L0, L1, L2, L3    // Percepção
L4                 // Fronteira
L5, L6, L7        // Processamento
L8, L9, LA        // Interação
LB, LC            // Emergência
LD, LE, LF        // Meta
```

### 5.5 Operadores

| Categoria | Operadores | Descrição |
|-----------|------------|-----------|
| Aritméticos | `+`, `-`, `*`, `/`, `**` | Soma, subtração, multiplicação, divisão, potência |
| Comparação | `==`, `!=`, `<`, `<=`, `>`, `>=` | Igualdade, diferença, menor, menor-igual, maior, maior-igual |
| Lógicos | `&&`, `\|\|`, `!` | AND, OR, NOT |
| Bitwise | `^`, `&`, `\|` | XOR, AND, OR (também usados para camadas) |
| Pipeline | `\|>` | Encadeamento de transformações |
| Unários | `-`, `!`, `~`, `\|x\|` | Negação, NOT, conjugado complexo, magnitude |

### 5.6 Literais

```lis
// Inteiros
42
-17
0

// Floats
3.14
-2.5e-3
1.0

// Strings
"hello world"
"linha com \"aspas\""

// Booleanos
true
false
```

### 5.7 Comentários

```lis
// Comentário de linha

/*
   Comentário
   de bloco
*/
```

---

## 6. Sistema de Tipos

### 6.1 Tipos Disponíveis

| Tipo | Sintaxe | Exemplo |
|------|---------|---------|
| Inteiro | `Int` | `let x: Int = 42;` |
| Float | `Float` | `let pi: Float = 3.14159;` |
| Complexo | `Complex` | `let c: Complex = (1.0, 0.5);` |
| ByteSil | `ByteSil` | `let b: ByteSil = bytesil_new(8, 4);` |
| Estado | `State` | `let s: State = state_vacuum();` |
| Camada | `Layer(n)` | Acesso via `state.L0` |
| Booleano | `Bool` | `let flag: Bool = true;` |
| String | `String` | `let msg: String = "hello";` |
| Função | `(T1, T2) -> R` | `fn add(a: Int, b: Int) -> Int` |
| Tupla | `(T1, T2, T3)` | `let pair = (1, 2);` |
| Unit | `()` | Retorno vazio |

### 6.2 Regras de Compatibilidade

```
         ┌──────────────────────────────────────┐
         │          Promoção Numérica           │
         │                                      │
         │    Int ───────► Float ───────► Complex
         │                                      │
         └──────────────────────────────────────┘

         ┌──────────────────────────────────────┐
         │       Compatibilidade ByteSil        │
         │                                      │
         │    Complex ◄────────► ByteSil        │
         │                                      │
         └──────────────────────────────────────┘
```

**Regras:**
1. `Error` é compatível com tudo (recovery)
2. Inteiros promovem para Float e Complex
3. Complex e ByteSil são intercambiáveis
4. Hardware hints devem corresponder
5. Tipos Unknown são compatíveis com qualquer coisa (inferência)
6. Funções compatíveis se assinaturas correspondem
7. Tuplas compatíveis se elementos correspondem

### 6.3 Inferência de Tipos

LIS usa inferência de tipos bidirecional:

```lis
// Tipo inferido como Int
let x = 42;

// Tipo inferido como Float
let y = 3.14;

// Tipo explícito
let z: Float = 42;  // Int promovido para Float

// Inferência em funções
fn double(n) {      // n inferido pelo uso
    return n * 2;   // retorno inferido
}
```

---

## 7. Sintaxe LIS

### 7.1 Estrutura do Programa

```lis
// Definição de tipo (opcional)
type Vector = State;

// Definição de função
fn function_name(param1: Type, param2: Type) -> ReturnType {
    // corpo
}

// Definição de transform
transform transform_name(input: Type) -> Type {
    // corpo
}

// Ponto de entrada (convenção)
fn main() {
    // código principal
}
```

### 7.2 Definição de Funções

```lis
// Função simples
fn add(a: Int, b: Int) -> Int {
    return a + b;
}

// Função sem tipo de retorno (Unit)
fn print_hello() {
    print_string("Hello!");
}

// Função com tipos inferidos
fn multiply(a, b) {
    return a * b;
}

// Transform (especializado para transformações)
transform normalize(s: State) -> State {
    return state_normalize(s);
}
```

### 7.3 Definição de Tipos

```lis
type Tensor = State;
type Layer = ByteSil;
type Weights = State;
```

---

## 8. Expressões

### 8.1 Precedência de Operadores

(Do mais baixo para o mais alto)

| Nível | Operador | Associatividade |
|-------|----------|-----------------|
| 1 | `\|>` | Esquerda |
| 2 | `\|\|` | Esquerda |
| 3 | `&&` | Esquerda |
| 4 | `==`, `!=` | Esquerda |
| 5 | `<`, `<=`, `>`, `>=` | Esquerda |
| 6 | `^`, `&`, `\|` | Esquerda |
| 7 | `+`, `-` | Esquerda |
| 8 | `*`, `/` | Esquerda |
| 9 | `**` | Direita |
| 10 | `-`, `!`, `~`, `\|x\|` | Unário |
| 11 | Primário | - |

### 8.2 Expressões Literais

```lis
// Inteiros
42
-17

// Floats
3.14
-2.5e-3

// Strings
"texto"

// Booleanos
true
false
```

### 8.3 Expressões de Variáveis

```lis
let x = 42;
let y = x;      // Referência a variável
let z = x + y;  // Uso em expressão
```

### 8.4 Expressões Binárias

```lis
// Aritméticas
a + b
a - b
a * b
a / b
a ** b   // Potência

// Comparação
a == b
a != b
a < b
a <= b
a > b
a >= b

// Lógicas
a && b
a || b

// Bitwise/Layer
a ^ b    // XOR
a & b    // AND
a | b    // OR
```

### 8.5 Expressões Unárias

```lis
-x       // Negação numérica
!flag    // NOT lógico
~c       // Conjugado complexo
|x|      // Magnitude (valor absoluto)
```

### 8.6 Chamadas de Função

```lis
// Chamada simples
sin(x)
sqrt(y)

// Múltiplos argumentos
add(1, 2)
max(a, b, c)

// Chamada encadeada
sqrt(abs(x))
```

### 8.7 Acesso a Camadas

```lis
let s = state_vacuum();

// Acesso de leitura
let layer0 = s.L0;
let layer5 = s.L5;
let layerF = s.LF;

// Modificação via função
let s2 = state_set_layer(s, 0, new_value);
```

### 8.8 Construção de State

```lis
// Via vacuum (todas as camadas zeradas)
let s = state_vacuum();

// Via neutral (valor padrão)
let s = state_neutral();

// Construção manual
let s = state_vacuum();
let s = state_set_layer(s, 0, val0);
let s = state_set_layer(s, 1, val1);
// ... continua para todas as 16 camadas
```

### 8.9 Números Complexos

```lis
// Criação em log-polar (rho, theta como inteiros 0-15)
let c = bytesil_new(8, 4);

// Criação a partir de magnitude e fase
let c = from_mag_phase(1.0, 3.14159);

// Criação a partir de cartesiano
let c = from_cartesian(1.0, 0.5);
```

### 8.10 Tuplas

```lis
// Criação
let pair = (1, 2);
let triple = (1.0, "hello", true);

// Uso (desestruturação não suportada diretamente)
```

### 8.11 Pipeline (Pipe)

```lis
// Encadeamento de transformações
let result = input |> transform1 |> transform2 |> transform3;

// Equivalente a:
let temp1 = transform1(input);
let temp2 = transform2(temp1);
let result = transform3(temp2);
```

### 8.12 Feedback e Emergência

```lis
// Feedback loop
let stabilized = feedback(state);

// Emergência (comportamento de swarm)
let emerged = emerge(collective_state);
```

---

## 9. Statements

### 9.1 Let Binding

```lis
// Com inferência de tipo
let x = 42;

// Com tipo explícito
let y: Float = 3.14;

// Imutável por padrão (rebinding permitido)
let x = 1;
let x = x + 1;  // Shadowing
```

### 9.2 Assignment

```lis
// Atribuição a variável existente
x = 42;
y = x + 1;
```

### 9.3 Return

```lis
// Retorno com valor
fn add(a: Int, b: Int) -> Int {
    return a + b;
}

// Retorno implícito (Unit)
fn log_message(msg: String) {
    print_string(msg);
    return;
}
```

### 9.4 If/Else

```lis
// If simples
if condition {
    // then branch
}

// If/Else
if condition {
    // then branch
} else {
    // else branch
}

// If/Else If/Else
if condition1 {
    // branch 1
} else {
    if condition2 {
        // branch 2
    } else {
        // branch 3
    }
}
```

### 9.5 Loop

```lis
// Loop infinito (usar break para sair)
loop {
    // corpo
    if done {
        break;
    }
}

// Loop com contador
let i = 0;
loop {
    if i >= 10 {
        break;
    }
    // corpo
    let i = i + 1;
}
```

### 9.6 Break e Continue

```lis
loop {
    if should_skip {
        continue;  // Pula para próxima iteração
    }

    if should_stop {
        break;     // Sai do loop
    }

    // corpo normal
}
```

### 9.7 Expression Statement

```lis
// Expressão como statement (resultado descartado)
print_string("hello");
some_side_effect();
```

---

## 10. Funções Intrínsecas

### 10.1 Operações ByteSil

| Função | Assinatura | Descrição |
|--------|------------|-----------|
| `bytesil_new` | `(Int, Int) -> ByteSil` | Cria ByteSil de rho e theta |
| `bytesil_magnitude` | `(ByteSil) -> Float` | Retorna magnitude |
| `bytesil_phase_radians` | `(ByteSil) -> Float` | Retorna fase em radianos |
| `bytesil_mul` | `(ByteSil, ByteSil) -> ByteSil` | Multiplicação |
| `bytesil_div` | `(ByteSil, ByteSil) -> ByteSil` | Divisão |
| `bytesil_pow` | `(ByteSil, Float) -> ByteSil` | Potência |
| `bytesil_sqrt` | `(ByteSil) -> ByteSil` | Raiz quadrada (alias: `bytesil_root`) |
| `bytesil_conjugate` | `(ByteSil) -> ByteSil` | Conjugado complexo (alias: `bytesil_conj`) |
| `bytesil_rho` | `(ByteSil) -> Int` | Retorna índice rho (-8 a +7) |
| `bytesil_theta` | `(ByteSil) -> Int` | Retorna índice theta (0 a 15) |
| `bytesil_xor` | `(ByteSil, ByteSil) -> ByteSil` | XOR de ByteSils |

### 10.2 Matemática

| Função | Assinatura | Descrição |
|--------|------------|-----------|
| `sin` | `(Float) -> Float` | Seno |
| `cos` | `(Float) -> Float` | Cosseno |
| `tan` | `(Float) -> Float` | Tangente |
| `asin` | `(Float) -> Float` | Arco seno |
| `acos` | `(Float) -> Float` | Arco cosseno |
| `atan` | `(Float) -> Float` | Arco tangente |
| `atan2` | `(Float, Float) -> Float` | Arco tangente de y/x |
| `sqrt` | `(Float) -> Float` | Raiz quadrada |
| `exp` | `(Float) -> Float` | Exponencial |
| `ln` | `(Float) -> Float` | Logaritmo natural |
| `log2` | `(Float) -> Float` | Logaritmo base 2 |
| `log10` | `(Float) -> Float` | Logaritmo base 10 |
| `floor` | `(Float) -> Float` | Arredonda para baixo |
| `ceil` | `(Float) -> Float` | Arredonda para cima |
| `round` | `(Float) -> Float` | Arredonda |
| `abs` | `(Float) -> Float` | Valor absoluto |
| `min` | `(Float, Float) -> Float` | Mínimo |
| `max` | `(Float, Float) -> Float` | Máximo |
| `pi` | `() -> Float` | Constante π |

### 10.3 Operações de Estado

| Função | Assinatura | Descrição |
|--------|------------|-----------|
| `state_vacuum` | `() -> State` | Estado zerado |
| `state_neutral` | `() -> State` | Estado neutro |
| `state_get_layer` | `(State, Int) -> ByteSil` | Obtém camada |
| `state_set_layer` | `(State, Int, ByteSil) -> State` | Define camada |
| `state_xor` | `(State, State) -> State` | XOR de estados |
| `state_add` | `(State, State) -> State` | Soma de estados |
| `state_tensor` | `(State, State) -> State` | Produto tensorial |
| `state_normalize` | `(State) -> State` | Normaliza estado |
| `state_collapse_xor` | `(State) -> ByteSil` | Colapsa via XOR |
| `state_collapse_average` | `(State) -> ByteSil` | Colapsa via média |

### 10.4 Operações Complexas

| Função | Assinatura | Descrição |
|--------|------------|-----------|
| `complex_add` | `(ByteSil, ByteSil) -> ByteSil` | Soma complexa |
| `complex_sub` | `(ByteSil, ByteSil) -> ByteSil` | Subtração complexa |
| `complex_scale` | `(ByteSil, Float) -> ByteSil` | Escala magnitude |
| `complex_rotate` | `(ByteSil, Float) -> ByteSil` | Rotaciona fase |
| `complex_lerp` | `(ByteSil, ByteSil, Float) -> ByteSil` | Interpolação linear |

### 10.5 Operações de String

| Função | Assinatura | Descrição |
|--------|------------|-----------|
| `string_concat` | `(String, String) -> String` | Concatenação |
| `string_length` | `(String) -> Int` | Comprimento |
| `string_contains` | `(String, String) -> Bool` | Contém substring |
| `string_to_upper` | `(String) -> String` | Maiúsculas |
| `string_to_lower` | `(String) -> String` | Minúsculas |
| `string_trim` | `(String) -> String` | Remove espaços |

### 10.6 I/O

| Função | Assinatura | Descrição |
|--------|------------|-----------|
| `print_int` | `(Int) -> ()` | Imprime inteiro |
| `print_float` | `(Float) -> ()` | Imprime float |
| `print_string` | `(String) -> ()` | Imprime string |
| `read_line` | `() -> String` | Lê linha |
| `read_int` | `() -> Int` | Lê inteiro |

### 10.7 Debug

| Função | Assinatura | Descrição |
|--------|------------|-----------|
| `assert` | `(Bool, String) -> ()` | Asserção |
| `debug_print` | `(String) -> ()` | Print de debug |
| `trace_state` | `(State) -> ()` | Trace de estado |
| `timestamp_millis` | `() -> Int` | Timestamp em ms |

### 10.8 HTTP

| Função | Assinatura | Descrição |
|--------|------------|-----------|
| `http_get` | `(String) -> String` | GET request |
| `http_post` | `(String, String) -> String` | POST request |
| `http_put` | `(String, String) -> String` | PUT request |
| `http_patch` | `(String, String) -> String` | PATCH request |
| `http_delete` | `(String) -> String` | DELETE request |

### 10.9 Operações de Camadas

| Função | Assinatura | Descrição |
|--------|------------|-----------|
| `fuse_vision_audio` | `(State, State) -> State` | Fusão multimodal |
| `fuse_multimodal` | `(State, State, State) -> State` | Fusão de 3 modalidades |
| `shift_layers_up` | `(State, Int) -> State` | Desloca camadas para cima |
| `shift_layers_down` | `(State, Int) -> State` | Desloca camadas para baixo |
| `rotate_layers` | `(State, Int) -> State` | Rotaciona camadas |

### 10.10 Transformações

| Função | Assinatura | Descrição |
|--------|------------|-----------|
| `transform_phase_shift` | `(State, Float) -> State` | Desloca fase |
| `transform_magnitude_scale` | `(State, Float) -> State` | Escala magnitude |
| `apply_feedback` | `(State, State) -> State` | Aplica feedback |
| `detect_emergence` | `(State) -> Bool` | Detecta emergência |
| `relu_state` | `(State) -> State` | ReLU em estado |

---

## 11. Linguagem SIL (Assembly VSP)

### 11.1 Formato de Arquivo SIL

```sil
.mode SIL-128

.data
    ; Dados estáticos (opcional)

.code

label_name:
    INSTRUCTION operands    ; Comentário
    INSTRUCTION operands
    RET
```

### 11.2 Instruções

| Instrução | Sintaxe | Descrição |
|-----------|---------|-----------|
| `MOVI` | `MOVI Rx, imm` | Move valor imediato para registrador |
| `MOV` | `MOV Rx, Ry` | Copia registrador |
| `COMPLEX` | `COMPLEX Rx, Ry, Rz` | Constrói complexo de rho e theta |
| `ADD` | `ADD Rx, Ry, Rz` | Rx = Ry + Rz |
| `SUB` | `SUB Rx, Ry, Rz` | Rx = Ry - Rz |
| `MUL` | `MUL Rx, Ry, Rz` | Rx = Ry * Rz |
| `DIV` | `DIV Rx, Ry, Rz` | Rx = Ry / Rz |
| `CMP` | `CMP Rx, Ry, Rz` | Compara Ry com Rz, resultado em Rx |
| `JMP` | `JMP label` | Salto incondicional |
| `JZ` | `JZ Rx, label` | Salta se Rx == 0 |
| `JNZ` | `JNZ Rx, label` | Salta se Rx != 0 |
| `CALL` | `CALL function` | Chama função/intrínseca |
| `RET` | `RET` | Retorna |
| `PUSH` | `PUSH Rx` | Empilha registrador |
| `POP` | `POP Rx` | Desempilha para registrador |
| `NOP` | `NOP` | Nenhuma operação |

### 11.3 Registradores

```
R0-RF    16 registradores de propósito geral

Convenções:
- R0: Valor de retorno / primeiro argumento
- R1-R3: Argumentos adicionais
- R4-RF: Temporários / locais
```

### 11.4 Exemplo de Compilação LIS → SIL

**Código LIS:**
```lis
fn add(a: Int, b: Int) -> Int {
    return a + b;
}
```

**Código SIL gerado:**
```sil
.mode SIL-128

.code

add:
    ; a está em R0, b está em R1
    ADD R0, R0, R1    ; R0 = a + b
    RET               ; Retorna R0
```

### 11.5 Exemplo Complexo

**Código LIS:**
```lis
fn magnitude(s: State) -> Float {
    let layer0 = state_get_layer(s, 0);
    return bytesil_magnitude(layer0);
}
```

**Código SIL gerado:**
```sil
.mode SIL-128

.code

magnitude:
    ; s está em R0
    MOVI R1, 0                    ; índice da camada
    CALL state_get_layer          ; R0 = layer0
    CALL bytesil_magnitude        ; R0 = magnitude
    RET
```

---

## 12. Tratamento de Erros

### 12.1 Tipos de Erro

| Tipo | Código | Descrição |
|------|--------|-----------|
| `LexError` | - | Erro de tokenização |
| `ParseError` | - | Erro de sintaxe |
| `TypeError` | E03xx | Erro de tipo |
| `SemanticError` | E04xx | Erro semântico |
| `CodeGenError` | - | Erro de geração de código |

### 12.2 Erros de Tipo

| Código | Nome | Descrição |
|--------|------|-----------|
| E0308 | Mismatch | Tipo esperado diferente do encontrado |
| E0308 | InfiniteType | Tipo se refere a si mesmo |
| E0425 | UndefinedVariable | Variável não definida no escopo |
| E0516 | InvalidLayerAccess | Camada fora do range L0-LF |
| E0601 | HardwareConflict | Hardware hints incompatíveis |
| E0061 | ArgumentCountMismatch | Número errado de argumentos |
| E0369 | InvalidOperation | Operador inválido para tipos |

### 12.3 Formato de Mensagem de Erro

```
error[E0308]: type mismatch
  --> src/main.lis:10:5
   |
10 |     let x: Int = "hello";
   |     ^^^^^^^^^^^^^^^^^^^^
   |
   = expected: Int
   = found: String
   = note: cannot assign String to Int variable
```

---

## 13. Integração com Rust

### 13.1 Usando lis-core como Biblioteca

```rust
use lis_core::{parse, compile, Compiler};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        fn main() {
            let x = 42;
            print_int(x);
        }
    "#;

    // Parse para AST
    let ast = parse(source)?;

    // Compilar para assembly SIL
    let assembly = compile(source)?;

    println!("{}", assembly);
    Ok(())
}
```

### 13.2 API Pública

```rust
// Parsing
pub fn parse(source: &str) -> Result<Program, Error>

// Compilação para VSP Assembly
pub fn compile(source: &str) -> Result<String, Error>

// Compilação para bytecode (feature: bytecode)
#[cfg(feature = "bytecode")]
pub fn compile_to_bytecode(source: &str) -> Result<Vec<u8>, Error>

// Compilação para JSIL (feature: jsil)
#[cfg(feature = "jsil")]
pub fn compile_to_jsil(
    source: &str,
    output_path: &str,
    compression: Option<Compression>
) -> Result<JsilStats, Error>

// JIT execution (feature: llvm)
#[cfg(feature = "llvm")]
pub fn jit_execute(source: &str) -> Result<i64, Error>

// AOT compilation (feature: llvm)
#[cfg(feature = "llvm")]
pub fn compile_to_object(source: &str, output_path: &str) -> Result<(), Error>
```

### 13.3 Tipos Principais

```rust
// AST
pub struct Program {
    pub items: Vec<Item>,
}

pub enum Item {
    Function(Function),
    Transform(Transform),
    TypeAlias(TypeAlias),
    Use(UseStatement),
    Module(ModuleDecl),
    ExternFunction(ExternFn),
}

pub struct UseStatement {
    pub path: Vec<String>,       // ["neural", "layers"]
    pub alias: Option<String>,   // as nome
    pub items: Option<Vec<String>>, // {Dense, Conv2D}
    pub is_pub: bool,
}

pub struct ModuleDecl {
    pub name: String,
    pub is_pub: bool,
}

pub struct ExternFn {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
}

pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Stmt>,
    pub hardware_hint: Option<HardwareHint>,
}

// Tipos
pub enum Type {
    Int,
    Float,
    Complex,
    ByteSil,
    State,
    Layer(u8),
    Hardware(HardwareHint),
    Function(Vec<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Bool,
    String,
    Unit,
    Unknown(u32),
    Named(String),
    Error,
}

// Hardware hints
pub enum HardwareHint {
    Cpu,
    Gpu,
    Npu,
    Simd,
    Photonic,
}
```

---

## 14. CLI - Linha de Comando

### 14.1 Instalação

```bash
# Compilar e instalar lis-cli
cargo install --path lis-cli
```

### 14.2 Comandos Principais

| Comando | Descrição |
|---------|-----------|
| `lis new <nome>` | Cria novo projeto |
| `lis init` | Inicializa projeto no diretório atual |
| `lis build` | Compila projeto para bytecode |
| `lis run` | Compila e executa |
| `lis check` | Verifica sintaxe sem compilar |
| `lis fmt` | Formata código fonte |
| `lis compile` | Compila arquivo único |

### 14.3 Criar Novo Projeto

```bash
# Cria estrutura de projeto
lis new meu-projeto

# Estrutura criada:
# meu-projeto/
# ├── lis.toml
# ├── src/
# │   └── main.lis
# └── .gitignore
```

### 14.4 Inicializar Projeto Existente

```bash
# No diretório atual
lis init

# Cria lis.toml e src/main.lis se não existirem
```

### 14.5 Compilar Projeto

```bash
# Compilação debug (padrão)
lis build

# Compilação release (otimizada)
lis build --release

# Output:
# target/debug/projeto.sil    (debug)
# target/release/projeto.silc (release)
```

### 14.6 Executar Projeto

```bash
# Compila e executa
lis run

# Equivalente a: lis build && sil run target/debug/projeto.sil
```

### 14.7 Verificar Sintaxe

```bash
# Verifica sem compilar
lis check

# Útil para CI/CD e validação rápida
```

### 14.8 Compilar Arquivo Único

```bash
# Arquivo standalone (sem lis.toml)
lis compile programa.lis

# Com output específico
lis compile programa.lis -o output.sil

# Para bytecode
lis compile programa.lis --bytecode -o output.silc
```

### 14.9 Formatar Código

```bash
# Formata todos os arquivos do projeto
lis fmt

# Verifica formatação sem modificar
lis fmt --check

# Formata arquivo específico
lis fmt src/main.lis
```

### 14.10 Exemplos de Uso

```bash
# Workflow típico
lis new neural-network
cd neural-network
# ... editar src/main.lis ...
lis check
lis run

# Desenvolvimento iterativo
lis run              # Compila e executa
# ... editar código ...
lis run              # Recompila e executa

# Preparar para produção
lis build --release
```

---

## 15. Exemplos Práticos

### 15.1 Hello World

```lis
fn main() {
    print_string("Hello, World!");
}
```

### 15.2 Fibonacci

```lis
fn fibonacci(n: Int) -> Int {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() {
    let result = fibonacci(10);
    print_int(result);
}
```

### 15.3 Operações com ByteSil

```lis
fn magnitude(b: ByteSil) -> Float {
    return bytesil_magnitude(b);
}

fn phase(b: ByteSil) -> Float {
    return bytesil_phase_radians(b);
}

fn from_cartesian(real: Float, imag: Float) -> ByteSil {
    let mag = sqrt(real * real + imag * imag);
    let phase_rad = atan2(imag, real);
    return from_mag_phase(mag, phase_rad);
}

fn main() {
    let c = bytesil_new(8, 4);
    let m = magnitude(c);
    let p = phase(c);
    print_float(m);
    print_float(p);
}
```

### 15.4 Manipulação de Estado

```lis
fn initialize_state() -> State {
    let s = state_vacuum();
    let val = bytesil_new(8, 0);

    let s = state_set_layer(s, 0, val);
    let s = state_set_layer(s, 1, val);
    let s = state_set_layer(s, 2, val);
    // ... continua

    return s;
}

fn process_state(s: State) -> State {
    let layer0 = state_get_layer(s, 0);
    let processed = bytesil_mul(layer0, layer0);
    return state_set_layer(s, 0, processed);
}

fn main() {
    let s = initialize_state();
    let s = process_state(s);
    trace_state(s);
}
```

### 15.5 Pipeline de Transformação

```lis
transform relu(s: State) -> State {
    return relu_state(s);
}

transform normalize(s: State) -> State {
    return state_normalize(s);
}

transform scale(s: State) -> State {
    return transform_magnitude_scale(s, 0.5);
}

fn process(input: State) -> State {
    return input |> relu |> normalize |> scale;
}

fn main() {
    let s = state_neutral();
    let result = process(s);
    trace_state(result);
}
```

### 15.6 Dense Layer (Neural Network)

```lis
fn dense_forward(input: State, weights: State, bias: State) -> State {
    let weighted = state_tensor(input, weights);
    let with_bias = state_xor(weighted, bias);
    return with_bias;
}

fn dense_relu(input: State, weights: State, bias: State) -> State {
    let linear = dense_forward(input, weights, bias);
    return relu_state(linear);
}

fn main() {
    let input = state_neutral();
    let weights = state_neutral();
    let bias = state_vacuum();

    let output = dense_relu(input, weights, bias);
    trace_state(output);
}
```

### 15.7 Otimizador SGD

```lis
fn scale_state(s: State, factor: Float) -> State {
    return transform_magnitude_scale(s, factor);
}

fn subtract_states(a: State, b: State) -> State {
    return state_xor(a, b);  // Aproximação via XOR
}

fn sgd_step(params: State, gradients: State, lr: Float) -> State {
    let scaled_grad = scale_state(gradients, lr);
    return subtract_states(params, scaled_grad);
}

fn sgd_momentum(
    params: State,
    gradients: State,
    velocity: State,
    lr: Float,
    momentum: Float
) -> State {
    let v_momentum = scale_state(velocity, momentum);
    let grad_scaled = scale_state(gradients, lr);
    let new_velocity = subtract_states(v_momentum, grad_scaled);
    return subtract_states(params, new_velocity);
}
```

### 15.8 Learning Rate Scheduling

```lis
fn lr_cosine_annealing(
    initial_lr: Float,
    step: Int,
    total_steps: Int,
    min_lr: Float
) -> Float {
    let progress = step * 1.0 / total_steps * 1.0;
    let cos_val = cos(pi() * progress);
    return min_lr + 0.5 * (initial_lr - min_lr) * (1.0 + cos_val);
}

fn lr_linear_warmup(
    target_lr: Float,
    step: Int,
    warmup_steps: Int
) -> Float {
    if step >= warmup_steps {
        return target_lr;
    }
    return target_lr * step * 1.0 / warmup_steps * 1.0;
}
```

### 15.9 Loop com Condição

```lis
fn sum_to_n(n: Int) -> Int {
    let sum = 0;
    let i = 1;

    loop {
        if i > n {
            break;
        }
        let sum = sum + i;
        let i = i + 1;
    }

    return sum;
}

fn main() {
    let result = sum_to_n(100);
    print_int(result);  // 5050
}
```

### 15.10 Interpolação de Estados

```lis
fn clamp(x: Float, min_val: Float, max_val: Float) -> Float {
    if x < min_val {
        return min_val;
    }
    if x > max_val {
        return max_val;
    }
    return x;
}

fn interpolate_layer(a: ByteSil, b: ByteSil, t: Float) -> ByteSil {
    return complex_lerp(a, b, t);
}

fn interpolate_state(a: State, b: State, t: Float) -> State {
    let tc = clamp(t, 0.0, 1.0);
    let result = state_vacuum();

    let la0 = state_get_layer(a, 0);
    let lb0 = state_get_layer(b, 0);
    let result = state_set_layer(result, 0, interpolate_layer(la0, lb0, tc));

    // Repetir para todas as 16 camadas...

    return result;
}
```

---

## Apêndice A: Referência Rápida

### Keywords

```
fn        transform   type       let        return
if        else        loop       break      continue
true      false       feedback   emerge
use       mod         pub        as         extern
```

### Tipos Built-in

```
Int       Float       Bool       String     ByteSil    State
```

### Hardware Hints

```
@cpu      @gpu        @npu       @simd      @photonic
```

### Camadas

```
L0  L1  L2  L3  L4  L5  L6  L7  L8  L9  LA  LB  LC  LD  LE  LF
```

### Operadores por Precedência

```
|>                    (pipe)
||                    (logical or)
&&                    (logical and)
==  !=                (equality)
<   <=  >   >=        (comparison)
^   &   |             (bitwise)
+   -                 (additive)
*   /                 (multiplicative)
**                    (power)
-   !   ~   |x|       (unary)
```

---

## Apêndice B: Mensagens de Erro Comuns

| Erro | Causa | Solução |
|------|-------|---------|
| `undefined variable` | Variável não declarada | Declare com `let` |
| `type mismatch` | Tipos incompatíveis | Verifique tipos |
| `invalid layer access` | Camada fora de L0-LF | Use L0-LF apenas |
| `argument count mismatch` | Número errado de args | Verifique assinatura |
| `unexpected token` | Erro de sintaxe | Verifique sintaxe |

---

## Apêndice C: Arquivos de Referência

| Arquivo | Conteúdo |
|---------|----------|
| `lis-core/src/lexer.rs` | Tokenização |
| `lis-core/src/parser.rs` | Parser |
| `lis-core/src/ast/mod.rs` | Definições AST |
| `lis-core/src/types/mod.rs` | Sistema de tipos |
| `lis-core/src/types/checker.rs` | Verificação de tipos |
| `lis-core/src/compiler.rs` | Geração de código |
| `lis-core/src/resolver.rs` | Resolução de módulos |
| `lis-core/src/manifest.rs` | Parser de lis.toml |
| `lis-core/src/error.rs` | Tipos de erro |
| `lis-core/src/lib.rs` | API pública |
| `lis-cli/src/main.rs` | CLI e comandos |

---

*Documentação gerada para LIS/SIL - Language for Intelligent Systems*
