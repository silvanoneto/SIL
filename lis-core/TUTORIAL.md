# LIS Tutorial - Getting Started

Bem-vindo ao LIS (Language for Intelligent Systems)! Este tutorial vai te guiar pelos conceitos fundamentais da linguagem.

## Instalação

```bash
# Clone o repositório
git clone https://github.com/silvanoneto/
cd 

# Build o compilador LIS
cargo build --release -p lis-cli

# Adicione ao PATH (opcional)
export PATH="$PATH:$(pwd)/target/release"
```

## Seu Primeiro Programa LIS

Crie um arquivo `hello.lis`:

```lis
fn main() {
    let x = 42;
    let y = x + 1;
    println("Hello, SIL!");
    print_int(y);
}
```

Compile para VSP assembly:

```bash
lis compile hello.lis
# Gera hello.sil
```

Ver o assembly gerado:

```bash
lis compile hello.lis --print
```

---

## Conceitos Fundamentais

### 1. Funções

```lis
fn add(a: Int, b: Int) {
    return a + b;
}

fn main() {
    let result = add(10, 32);
    print_int(result);
}
```

### 2. Transforms

Transforms são funções puras que operam sobre estados:

```lis
transform normalize(input: State) {
    let scaled = input.L0 * 0.5;
    return scaled;
}

fn main() {
    let state = state_neutral();
    let normalized = normalize(state);
}
```

### 3. Complex Numbers (ByteSil)

LIS usa encoding log-polar: `(rho, theta)` onde `z = e^rho * e^(i*theta*pi/128)`

```lis
fn complex_demo() {
    // Criar número complexo
    let z1 = bytesil_new(7, 0);     // magnitude máxima, fase 0
    let z2 = bytesil_new(5, 64);    // magnitude média, fase 90°

    // Multiplicação em O(1)!
    let product = bytesil_mul(z1, z2);

    // Conjugado
    let conj = bytesil_conj(z1);

    // Magnitude
    let mag = bytesil_magnitude(z1);

    print_bytesil(product);
}
```

**Por que log-polar?**
- Multiplicação/divisão são O(1) (soma/subtração de logs)
- Ideal para processamento de sinais
- Natural para hardware fotônico

### 4. SilState - 16 Layers

O estado em LIS é um vetor de 16 layers (L0-LF), cada um sendo um ByteSil:

```lis
fn layer_demo() {
    // Construir estado
    let state = state_neutral();

    // Modificar layers individuais
    let photon = bytesil_new(7, 0);
    let state = state_set_layer(state, 0, photon);

    // Acessar layers
    let l0 = state_get_layer(state, 0);    // Photonic
    let l1 = state_get_layer(state, 1);    // Acoustic
    let lf = state_get_layer(state, 15);   // Collapse

    print_state(state);
}
```

**Organização das 16 Layers:**

```
L0-L4  | PERCEPÇÃO     | Photonic, Acoustic, Olfactory, Gustatory, Haptic
L5-L7  | PROCESSAMENTO | Electronic, Psychomotor, Environmental
L8-LA  | INTERAÇÃO     | Cybernetic, Geopolitical, Cosmopolitical
LB-LC  | EMERGÊNCIA    | Synergic, Quantum
LD-LF  | META          | Superposition, Entanglement, Collapse
```

### 5. Control Flow

```lis
fn control_demo() {
    let x = 15;

    // If/Else
    if x > 10 {
        println("x is large");
    } else {
        println("x is small");
    }

    // Loop
    let i = 0;
    loop {
        if i > 5 {
            break;
        }
        print_int(i);
        i = i + 1;
    }
}
```

### 6. Pipeline Operator

O operador `|>` permite compor transforms:

```lis
transform step1(x: State) { return x; }
transform step2(x: State) { return x; }
transform step3(x: State) { return x; }

fn pipeline_demo() {
    let input = state_neutral();

    let result = input
        |> step1
        |> step2
        |> step3;

    print_state(result);
}
```

### 7. Feedback Loops

LIS suporta feedback como primitiva:

```lis
fn feedback_demo() {
    let state = state_neutral();
    let gain = 0.9;

    // Aplicar feedback
    let result = apply_feedback(state, gain);

    // Detectar emergência
    let threshold = 0.5;
    let emerged = detect_emergence(result, threshold);

    print_bool(emerged);
}
```

### 8. Hardware Hints

Guie a compilação para hardware específico:

```lis
// Executa em CPU
@cpu
fn sequential_work(data: State) {
    return data;
}

// Executa em GPU
@gpu
fn parallel_batch(batch: State) {
    return batch;
}

// Executa em NPU (Neural Processing Unit)
@npu
fn neural_inference(input: State) {
    return input;
}

// Usa instruções SIMD
@simd
fn vector_math(a: State, b: State) {
    return a;
}
```

---

## Standard Library

### ByteSil - Operações Log-Polar

| Função | Descrição |
|:-------|:----------|
| `bytesil_new(rho, theta)` | Cria ByteSil de componentes raw |
| `bytesil_from_complex(re, im)` | Cria de número complexo cartesiano |
| `bytesil_to_complex(b)` | Converte para (re, im) |
| `bytesil_null()` | Valor NULL (0, 0) |
| `bytesil_one()` | Valor ONE (1 + 0i) |
| `bytesil_i()` | Valor I (0 + 1i) |
| `bytesil_neg_one()` | Valor -1 |
| `bytesil_neg_i()` | Valor -i |
| `bytesil_max()` | Valor máximo |
| `bytesil_mul(a, b)` | Multiplicação O(1) |
| `bytesil_div(a, b)` | Divisão O(1) |
| `bytesil_pow(b, n)` | Potência O(1) |
| `bytesil_root(b, n)` | Raiz O(1) |
| `bytesil_inv(b)` | Inverso (1/b) |
| `bytesil_conj(b)` | Conjugado |
| `bytesil_xor(a, b)` | XOR binário |
| `bytesil_magnitude(b)` | Magnitude |
| `bytesil_rho(b)` | Componente rho |
| `bytesil_theta(b)` | Componente theta |
| `bytesil_is_null(b)` | Verifica se é null |
| `bytesil_is_real(b)` | Verifica se é real |

### State - Manipulação de 16 Camadas

| Função | Descrição |
|:-------|:----------|
| `state_vacuum()` | Estado vazio (todos NULL) |
| `state_neutral()` | Estado neutro (todos ONE) |
| `state_maximum()` | Estado máximo (todos MAX) |
| `state_from_bytes(bytes)` | Cria de array de bytes |
| `state_to_bytes(s)` | Converte para bytes |
| `state_get_layer(s, idx)` | Obtém layer [0-15] |
| `state_set_layer(s, idx, val)` | Define layer |
| `state_xor(s)` | XOR de todas as layers |
| `state_hash(s)` | Hash do estado |
| `state_equals(a, b)` | Compara estados |
| `state_is_vacuum(s)` | Verifica se é vazio |
| `state_is_neutral(s)` | Verifica se é neutro |
| `state_count_null_layers(s)` | Conta layers NULL |
| `state_count_active_layers(s)` | Conta layers ativas |

### Math - Funções Matemáticas

| Função | Descrição |
|:-------|:----------|
| `complex_add(a, b)` | Soma complexa |
| `complex_sub(a, b)` | Subtração complexa |
| `complex_scale(b, factor)` | Escala por fator real |
| `complex_rotate(b, deg)` | Rotação em graus |
| `complex_lerp(a, b, t)` | Interpolação linear |
| `sin(x)`, `cos(x)`, `tan(x)` | Trigonométricas |
| `asin(x)`, `acos(x)`, `atan(x)` | Inversas |
| `atan2(y, x)` | Atan2 |
| `sqrt(x)` | Raiz quadrada |
| `pow_float(x, n)` | Potência |
| `exp(x)`, `ln(x)` | Exponencial e log natural |
| `log10(x)`, `log2(x)` | Logaritmos |
| `abs_int(x)`, `abs_float(x)` | Valor absoluto |
| `min_int(a,b)`, `max_int(a,b)` | Mínimo/máximo |
| `clamp_int(x, min, max)` | Limita valor |
| `floor(x)`, `ceil(x)`, `round(x)` | Arredondamento |
| `pi()`, `tau()`, `e()`, `phi()` | Constantes |

### String - Manipulação de Texto

| Função | Descrição |
|:-------|:----------|
| `string_length(s)` | Comprimento |
| `string_concat(a, b)` | Concatenação |
| `string_slice(s, start, end)` | Substring |
| `string_to_upper(s)` | Maiúsculas |
| `string_to_lower(s)` | Minúsculas |
| `string_contains(s, sub)` | Contém substring |
| `string_starts_with(s, prefix)` | Começa com |
| `string_ends_with(s, suffix)` | Termina com |
| `string_trim(s)` | Remove espaços |
| `string_replace(s, old, new)` | Substitui |
| `int_to_string(x)` | Int para string |
| `float_to_string(x)` | Float para string |
| `string_to_int(s)` | String para int |
| `string_to_float(s)` | String para float |

### I/O - Entrada e Saída

| Função | Descrição |
|:-------|:----------|
| `print_int(x)` | Imprime inteiro |
| `print_float(x)` | Imprime float |
| `print_string(s)` | Imprime string |
| `print_bool(b)` | Imprime boolean |
| `print_bytesil(b)` | Imprime ByteSil |
| `print_state(s)` | Imprime State |
| `println(s)` | Imprime com newline |
| `read_line()` | Lê linha |
| `read_int()` | Lê inteiro |
| `read_float()` | Lê float |

### Transforms - Transformações de Estado

| Função | Descrição |
|:-------|:----------|
| `transform_phase_shift(s, n)` | Shift de fase |
| `transform_magnitude_scale(s, n)` | Escala magnitude |
| `transform_layer_swap(s, a, b)` | Troca layers |
| `transform_xor_layers(s, a, b, c)` | XOR layers |
| `transform_identity(s)` | Identidade |
| `apply_feedback(s, gain)` | Aplica feedback |
| `detect_emergence(s, threshold)` | Detecta emergência |

### Layers - Operações de Camadas

| Função | Descrição |
|:-------|:----------|
| `fuse_vision_audio(v, a)` | Funde L0 + L1 |
| `fuse_multimodal(layers)` | Funde percepção |
| `normalize_perception(s)` | Normaliza L0-L4 |
| `shift_layers_up(s)` | Rota layers para cima |
| `shift_layers_down(s)` | Rota layers para baixo |
| `rotate_layers(s, n)` | Rota por n posições |

### Debug - Depuração

| Função | Descrição |
|:-------|:----------|
| `assert(condition)` | Asserção |
| `assert_eq_int(a, b)` | Asserção de igualdade |
| `debug_print(msg)` | Print de debug |
| `trace_state(s)` | Trace de estado |
| `timestamp_millis()` | Timestamp em ms |
| `timestamp_micros()` | Timestamp em us |

---

## Exemplos Práticos

### Sensor Fusion

```lis
fn sensor_fusion() {
    // Criar estado com sensores
    let s = state_neutral();

    // Simular leituras sensoriais
    let vision = bytesil_new(7, 0);    // L0: Photonic
    let audio = bytesil_new(5, 32);    // L1: Acoustic
    let touch = bytesil_new(3, 64);    // L4: Haptic

    // Aplicar aos layers
    let s = state_set_layer(s, 0, vision);
    let s = state_set_layer(s, 1, audio);
    let s = state_set_layer(s, 4, touch);

    // Fusão multimodal
    let fused = fuse_vision_audio(vision, audio);

    print_bytesil(fused);
    print_state(s);
}
```

### Processamento de Sinais

```lis
fn signal_processing() {
    // Criar sinal
    let signal = bytesil_new(5, 0);

    // Filtro passa-baixa (escalar)
    let filtered = complex_scale(signal, 0.5);

    // Shift de fase (90°)
    let shifted = complex_rotate(filtered, 90.0);

    // Magnitude
    let mag = bytesil_magnitude(shifted);

    println("Signal processing:");
    print_bytesil(signal);
    print_bytesil(shifted);
    print_float(mag);
}
```

### Sistema Adaptativo

```lis
fn adaptive_system() {
    let state = state_neutral();
    let target = bytesil_new(7, 0);

    let iterations = 0;
    loop {
        if iterations > 100 {
            break;
        }

        // Obter layer atual
        let current = state_get_layer(state, 0);

        // Calcular erro (simplificado)
        let error = bytesil_magnitude(current);

        // Verificar convergência
        if error < 0.01 {
            println("Converged!");
            break;
        }

        // Aplicar feedback
        state = apply_feedback(state, 0.9);

        iterations = iterations + 1;
    }

    print_int(iterations);
    print_state(state);
}
```

---

## Compilação e Execução

### Workflow Completo

```bash
# 1. Escrever código LIS
vim program.lis

# 2. Verificar sintaxe
lis check program.lis

# 3. Compilar para assembly
lis compile program.lis

# 4. Ver assembly gerado
cat program.sil

# 5. Executar diretamente
lis run program.lis
```

### Debug

```bash
# Ver assembly com comentários
lis compile program.lis --print

# Ver apenas erros
lis check program.lis 2>&1 | grep error
```

---

## Próximos Passos

1. **Explore os exemplos**: Veja `examples/lis/` para programas completos
2. **Leia a documentação**: Veja `docs/ARCHITECTURE.md` para entender o paradigma
3. **Experimente**: Escreva seus próprios programas!

## Recursos

- [docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md) - Arquitetura completa do sistema
- [docs/EXAMPLES.md](../docs/EXAMPLES.md) - Casos de uso práticos
- [examples/lis/](../examples/lis/) - Exemplos em LIS
- [examples/sil/](../examples/sil/) - Exemplos em Rust/SIL

## Ajuda

```bash
# Ver comandos disponíveis
lis --help

# Informações sobre LIS
lis info

# Versão
lis --version
```

---

**"We are the swarm. We are the vapor. We are the edge."**

理信 (Lixin) - Where logic and information are indistinguishable.
