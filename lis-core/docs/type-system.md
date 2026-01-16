# LIS Type System

**Status**: ✅ Implemented (Fase 1 Completa)
**Version**: 2026.1.16
**Last Updated**: 2026-01-12

---

## Overview

O LIS implementa um **sistema de tipos com inferência bidirecional** que combina type checking com suporte nativo para:

- **Hardware hints** (@gpu, @npu, @cpu, @simd, @photonic)
- **Números complexos** (representação log-polar: rho, theta)
- **Layers SIL** (L0-LF para 16 camadas)
- **State types** (16-layer state com ByteSil)

### Key Features

✅ **Type Inference**: Inferência automática na maioria dos casos
✅ **Type Safety**: Erros capturados em compile-time
✅ **Hardware Awareness**: Tipos podem carregar hints de execução
✅ **Rich Error Messages**: Mensagens com localização e sugestões

---

## Type Hierarchy

```
         Any
        /   \
     Num    State
    / | \     |
 Int Float Complex  (com 16 subtipos Layer L0-LF)
   \  |  /
    ByteSil  (representação runtime de 8 bits)
```

### Primitive Types

| Type | Description | Example |
|------|-------------|---------|
| `Int` | Inteiros 64-bit | `42`, `-100` |
| `Float` | Ponto flutuante 64-bit | `3.14`, `-0.5` |
| `Bool` | Booleano | `true`, `false` |
| `String` | Cadeia de caracteres | `"hello"` |
| `Unit` | Tipo vazio (void) | `()` |

### Complex Types

| Type | Description | Components |
|------|-------------|------------|
| `Complex` | Número complexo log-polar | `(rho: Float, theta: Float)` |
| `ByteSil` | Complexo 8-bit (runtime) | 4 bits magnitude + 4 bits phase |
| `State` | 16-layer SIL state | `{ L0..LF: ByteSil }` |
| `Layer(n)` | Camada específica | `L0`, `L1`, ..., `LF` |

### Function Types

```rust
fn(Int, Int) -> Int        // Função binária
fn(State) -> ByteSil       // Transform
fn() -> Unit               // Procedimento
```

---

## Type Inference Algorithm

LIS usa **inferência bidirecional** (Dunfield & Krishnaswami):

### Synthesis (Bottom-Up)

Infere tipos de expressões:

```lis
let x = 42;          // Infere: Int
let y = 3.14;        // Infere: Float
let z = (1.0, 0.0);  // Infere: Complex
```

### Checking (Top-Down)

Valida contra tipos esperados:

```lis
let x: Int = 42;     // Check: 42 é Int? ✓
let y: Int = 3.14;   // Check: 3.14 é Int? ✗ Type Error!
```

### Type Promotion

Tipos numéricos são automaticamente promovidos:

```
Int → Float → Complex → ByteSil
```

**Exemplos:**

```lis
let a = 1 + 2.5;     // 1 é promovido para Float → resultado Float
let b = 3.14 + (1.0, 0.0);  // 3.14 promovido para Complex
```

---

## Type Checking Rules

### Binary Operations

| Operator | Left Type | Right Type | Result Type |
|----------|-----------|------------|-------------|
| `+`, `-`, `*`, `/` | Numeric | Numeric | Common supertype |
| `**` (pow) | Numeric | Numeric | Common supertype |
| `<`, `<=`, `>`, `>=` | Numeric | Numeric | `Bool` |
| `==`, `!=` | Any | Same type | `Bool` |
| `&&`, `\|\|` | `Bool` | `Bool` | `Bool` |
| `&`, `\|`, `^` | `Int` | `Int` | `Int` |
| `&`, `\|`, `^` | `Layer` | `Layer` | `ByteSil` |

### Unary Operations

| Operator | Input Type | Result Type |
|----------|------------|-------------|
| `-` (neg) | Numeric | Same |
| `!` (not) | `Bool` | `Bool` |
| `~` (conj) | `Complex\|ByteSil` | Same |
| `\|x\|` (mag) | `Complex\|ByteSil` | `Float` |

### Layer Access

```lis
state.L0  // State → ByteSil
state.L5  // State → ByteSil
state.LF  // State → ByteSil

state.L10 // ✗ Error: Layer out of bounds (max LF)
```

### State Construction

```lis
let s = State {
    L0: (1.0, 0.0),   // Complex → ByteSil
    L1: (2.0, 0.5),
    L2: 3.14,         // Float → ByteSil (promoted)
};
```

---

## Hardware Hints

Hardware hints são anotações de tipo que indicam o substrato de execução:

```lis
fn process_gpu(data: State) @gpu {
    // Código otimizado para GPU
}

fn process_npu(data: State) @npu {
    // Código otimizado para NPU
}
```

### Supported Hints

- `@cpu` - Processamento CPU (padrão)
- `@gpu` - Paralelismo massivo (WGPU)
- `@npu` - Redes neurais (acelerador dedicado)
- `@simd` - Vetorização SIMD (AVX, NEON)
- `@photonic` - Computação fotônica (experimental)

### Type Specialization

Hardware hints não mudam o tipo base, mas podem influenciar código gerado:

```lis
fn add(a: Float, b: Float) -> Float {  // Versão CPU
    return a + b;
}

fn add(a: Float, b: Float) @simd -> Float {  // Versão SIMD
    // Vetorizado automaticamente
    return a + b;
}
```

---

## Error Messages

### Type Mismatch

```
error[E0308]: type mismatch at 12:18
   expected: Int
      found: Float
    context: let binding

   help: cast to Int using `as`: `3.14 as Int`
```

### Undefined Variable

```
error[E0425]: undefined variable 'x' at 5:13
```

### Invalid Layer Access

```
error[E0516]: invalid layer access L10 at 8:20 (valid: L0-LF)
```

### Hardware Conflict

```
error[E0601]: hardware hint conflict at 15:5
   required: @gpu
      found: @cpu

   help: ensure function is annotated with @gpu
```

### Argument Count Mismatch

```
error[E0061]: argument count mismatch at 10:15
   function 'add' expects 2 arguments, got 1
```

### Invalid Operation

```
error[E0369]: cannot apply operator '+' to types Bool and Int
```

---

## Implementation Details

### Type Context

O `TypeContext` mantém:

- **Bindings**: `HashMap<String, Type>` - variáveis → tipos
- **Constraints**: `Vec<Constraint>` - restrições coletadas
- **Fresh counter**: gerador de type variables

### Constraint Solving

Constraints são validados após inferência:

```rust
enum Constraint {
    Equal(Type, Type, Span),      // T1 = T2
    Subtype(Type, Type, Span),    // T1 <: T2
    Hardware(Type, Hint, Span),   // T tem hint
}
```

### Type Compatibility

Dois tipos são compatíveis se:

1. São exatamente iguais
2. Um pode ser promovido ao outro
3. Um deles é `Unknown` (type variable)

---

## Examples

### Basic Inference

```lis
fn main() {
    let x = 42;              // x: Int
    let y = x + 10;          // y: Int
    let z = 3.14;            // z: Float
    let w = x + z;           // w: Float (x promoted)
}
```

### Complex Numbers

```lis
fn main() {
    let z1 = (1.0, 0.0);     // z1: Complex
    let z2 = (2.0, 1.57);    // z2: Complex
    let sum = z1 + z2;       // sum: Complex
    let mag = |z1|;          // mag: Float
}
```

### Layer Operations

```lis
fn process(s: State) {
    let l0 = s.L0;           // l0: ByteSil
    let l1 = s.L1;           // l1: ByteSil
    let combined = l0 ^ l1;  // combined: ByteSil (XOR)
}
```

### Function Types

```lis
fn add(a: Int, b: Int) -> Int {
    return a + b;
}

fn main() {
    let result = add(1, 2);  // result: Int
}
```

---

## Test Coverage

**Status**: 38/39 testes passando (97.4%)

### Test Categories

- ✅ Basic type inference (4 tests)
- ✅ Complex numbers (2 tests)
- ✅ Binary operations (5 tests)
- ✅ Unary operations (2 tests)
- ✅ Variable references (2 tests)
- ✅ Type annotations (2 tests)
- ✅ Assignments (2 tests)
- ✅ Control flow (6 tests)
- ✅ Functions (2 tests)
- ✅ Layers and State (3 tests)
- ✅ Integration tests (3 tests)
- ✅ Error messages (1 test)
- ✅ Stress tests (2 tests)
- ✅ Edge cases (3 tests)

### Known Limitations

1. **Loop type checking**: Um edge case em loops precisa refinamento
2. **Generic types**: Não implementado (planejado para v0.2)
3. **Type aliases**: Parsing não completo
4. **Pattern matching**: Não implementado

---

## Performance

- **Type checking overhead**: ~5-10% do tempo total de compilação
- **Inference speed**: O(n) onde n = nodes no AST
- **Constraint solving**: O(c) onde c = número de constraints

### Benchmarks

```
Type check time for 100-line program: ~0.5ms
Type check time for 1000-line program: ~5ms
```

---

## Future Work

### Phase 2 Integration

Com a Standard Library (Fase 2), o type checker registrará:

- `sin()`, `cos()`, `exp()` → `Float -> Float`
- `sense_photonic()` → `() -> ByteSil`
- `act_motor()` → `ByteSil -> Unit`
- `collapse()` → `State -> ByteSil`

### Phase 3+ Enhancements

- **Generics**: `fn identity<T>(x: T) -> T`
- **Trait system**: Similar ao Rust
- **Effect system**: Track de side effects
- **Dependent types**: Tipos que dependem de valores

---

## References

- Dunfield & Krishnaswami (2013): "Complete and Easy Bidirectional Typechecking for Higher-Rank Polymorphism"
- SIL Core Types: [`sil-core/src/state/`](../../sil-core/src/state/)
- Type System Implementation: [`lis-core/src/types/`](../src/types/)

---

**Next**: [Standard Library](stdlib.md) | **Previous**: [Syntax](syntax.md)
