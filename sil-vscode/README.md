# 🎯 SIL & LIS Language Support for VS Code

**SIL** = **Symbolic Information Lattice** (inglês — perspectiva topológica) / **Sistema Informacional Lógico-matemático** (português — perspectiva descritiva)

**LIS** = **Language for Intelligent Systems** - Linguagem de alto nível que compila para SIL

Extensão completa para desenvolvimento em **SIL** (assembly) e **LIS** (linguagem de alto nível).

## ✨ Funcionalidades

### Para SIL (Assembly)
- **Syntax Highlighting** - Colorização semântica para 70+ opcodes
- **IntelliSense** - Auto-complete para opcodes, registradores e diretivas
- **Snippets** - 14+ templates de código prontos
- **Debugger** - Debug via DAP (Debug Adapter Protocol)
- **Assembler Integration** - Compilar .sil → .silc

### Para LIS (High-Level Language)
- **Syntax Highlighting** - Keywords, tipos, funções, operadores
- **IntelliSense** - Auto-complete para keywords, tipos, funções builtin
- **Snippets** - 20+ templates (funções, transforms, pipelines, etc.)
- **Compiler Integration** - Compilar .lis → .sil → .silc
- **Hardware Hints** - Suporte a @cpu, @gpu, @npu, @simd

### Recursos Compartilhados
- **Hover Info** - Documentação inline ao passar mouse
- **Diagnostics** - Erros e warnings em tempo real
- **Go to Definition** - Navegação para símbolos
- **Document Symbols** - Outline de símbolos (Ctrl+Shift+O)
- **Formatting** - Formatação automática de código

## 📦 Instalação

### Via VS Code Marketplace

1. Abra VS Code
2. Pressione `Ctrl+Shift+X`
3. Busque "SIL Language"
4. Clique em Install

### Via VSIX

```bash
code --install-extension sil-language-2026.1.16.vsix
```

### Desenvolvimento

```bash
cd sil-vscode
npm install
npm run compile
# F5 para abrir Extension Development Host
```

## 🚀 Uso

### Criar arquivo SIL (Assembly)

Crie arquivo com extensão `.sil`:

```sil
; Hello SIL
.mode SIL-128

.code
main:
    MOV R0, 0x42    ; Carregar valor
    TRANS R0, R1    ; Transformar
    HLT             ; Finalizar
```

### Criar arquivo LIS (High-Level)

Crie arquivo com extensão `.lis`:

```lis
// Hello LIS
fn main() {
    let state = sense();           // Captura sensorial
    let processed = transform(state);  // Processa
    act(processed);                // Atua
}

transform process_state(input: State) -> State {
    let photonic = input.L0;
    let acoustic = input.L1;
    let result = photonic * acoustic;
    return result;
}
```

### Comandos SIL

| Comando              | Descrição                    | Atalho          |
|:---------------------|:-----------------------------|:----------------|
| `SIL: New Program`   | Criar novo programa          | —               |
| `SIL: Assemble`      | Compilar .sil → .silc        | Ctrl+Shift+B    |
| `SIL: Run`           | Executar programa            | Ctrl+Shift+R    |
| `SIL: Debug`         | Iniciar debugger             | —               |
| `SIL: REPL`          | Abrir console interativo     | —               |
| `SIL: Disassemble`   | Desassemblar .silc           | —               |

### Comandos LIS

| Comando              | Descrição                    | Atalho          |
|:---------------------|:-----------------------------|:----------------|
| `LIS: New Program`   | Criar novo programa          | —               |
| `LIS: Compile to SIL`| Compilar .lis → .sil         | Ctrl+Shift+B    |
| `LIS: Build`         | Compilar .lis → .silc        | —               |
| `LIS: Run`           | Executar programa            | Ctrl+Shift+R    |

### Snippets SIL

| Prefixo      | Snippet                        |
|:-------------|:-------------------------------|
| `sil-prog`   | Template programa completo     |
| `sil-fn`     | Template função                |
| `sil-loop`   | Template loop                  |
| `sil-data`   | Template seção de dados        |

### Snippets LIS

| Prefixo          | Snippet                              |
|:-----------------|:-------------------------------------|
| `lis-prog`       | Programa básico                      |
| `lis-fn`         | Definição de função                  |
| `lis-transform`  | Transform com feedback loop          |
| `lis-state`      | Construção de State com layers       |
| `lis-pipeline`   | Pipeline de transformações           |
| `lis-if`         | Condicional if/else                  |
| `lis-loop`       | Loop infinito com break              |
| `lis-complex`    | Operações complexas (log-polar)      |
| `lis-gpu`        | Função com hint @gpu                 |
| `lis-npu`        | Função com hint @npu                 |
| `lis-spa`        | Sense-Process-Act control loop       |

## ⚙️ Configuração

### Configurações SIL

```json
{
  "sil.mode": "SIL-128",
  "sil.lsp.enabled": true,
  "sil.debug.stopOnEntry": false,
  "sil.format.alignOperands": true,
  "sil.format.uppercaseOpcodes": true
}
```

### Configurações LIS

```json
{
  "lis.compiler.path": "lis",
  "lis.silMode": "SIL-128",
  "lis.optimizationLevel": "O2",
  "lis.format.indentSize": 4,
  "lis.lsp.enabled": true
}
```

## 📁 Arquivos

```text
-vscode/
├── package.json                    # Manifest da extensão
├── language-configuration.json     # Config para SIL
├── language-configuration-lis.json # Config para LIS
├── syntaxes/
│   ├── sil.tmLanguage.json        # Grammar SIL
│   └── lis.tmLanguage.json        # Grammar LIS
├── snippets/
│   ├── sil.json                   # Snippets SIL (14)
│   └── lis.json                   # Snippets LIS (20)
├── src/
│   ├── extension.ts               # Entry point (SIL + LIS)
│   └── debugAdapter.ts            # DAP adapter
└── README.md
```

## 📜 License

AGPL-3.0
