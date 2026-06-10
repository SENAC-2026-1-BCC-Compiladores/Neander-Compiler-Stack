# 💻 Ecossistema Neander (Compilador, Assembler e VM)

Este repositório contém um ecossistema completo de compilação e execução para a arquitetura [**Neander**](https://www.inf.ufrgs.br/arq/wiki/doku.php?id=neander), desenvolvido 100% em **Rust**. 

O projeto foi construído desenhando um pipeline *End-to-End* que transforma expressões matemáticas em código de máquina, dividido em três camadas principais:

1. **Frontend (Calculadora/Parser):** Lê expressões matemáticas (suportando precedência e parênteses), gera uma Árvore Sintática (AST) e compila isso para código Assembly Neander, fazendo o gerenciamento automático de zonas reservadas da memória.
2. **Middle-end (Assembler):** Montador de duas passagens que resolve variáveis, *labels* nativos, lida com aninhamentos invisíveis via *Backpatching* e converte o texto Assembly em *bytecode* (bytes raw). Também suporta expansão de macros como `MUL` e `SUB`.
3. **Backend (Interpretador/VM):** Uma Máquina Virtual que carrega o binário na memória e executa as instruções simulando os ciclos do hardware real, atualizando *flags* e entregando o resultado final direto do Acumulador.

---

## ⚙️ A Máquina NEANDER

A Neander é uma arquitetura de processador hipotética e pedagógica. Ela possui uma estrutura de dados de 8 bits e é baseada no modelo de **Acumulador (ACC)** — ou seja, todas as operações aritméticas e lógicas utilizam o Acumulador como um de seus operandos e também como destino do resultado.

Para suportar nosso projeto, reservarmos os endereços finais da memória (251 a 255) para atuarem como pseudo-registradores de sistema (`T0` a `T4`), permitindo a execução de macros sem sujar a memória do usuário.

### Condition Codes (Flags)
A máquina reage aos resultados matemáticos atualizando duas *flags* essenciais de controle:
* **N (Negative):** Acende se o resultado da última operação no ACC for negativo.
* **Z (Zero):** Acende se o resultado da última operação no ACC for estritamente zero.

### Tabela de Opcodes (Instruções Nativas)

Abaixo está o conjunto de instruções nativas suportadas pelo processador:

| Instrução | Opcode (Dec) | Tamanho | Descrição |
| :--- | :--- | :--- | :--- |
| **NOP** | 0 | 1 byte | Nenhuma operação (*No Operation*). |
| **STA** | 16 | 2 bytes | Armazena o valor do ACC no endereço de memória especificado. |
| **LDA** | 32 | 2 bytes | Carrega o valor do endereço de memória para o ACC. |
| **ADD** | 48 | 2 bytes | Soma o valor da memória ao ACC. |
| **OR** | 64 | 2 bytes | Operação lógica OR bit a bit com o ACC. |
| **AND** | 80 | 2 bytes | Operação lógica AND bit a bit com o ACC. |
| **NOT** | 96 | 1 byte | Inverte todos os bits do ACC. |
| **JMP** | 128 | 2 bytes | Salto incondicional (PC = Endereço). |
| **JN** | 144 | 2 bytes | Salta para o endereço se a flag **N** (Negativo) estiver ligada. |
| **JZ** | 160 | 2 bytes | Salta para o endereço se a flag **Z** (Zero) estiver ligada. |
| **HLT** | 240 | 1 byte | Para a execução da máquina (*Halt*). |

---

## 🧩 Pontos Legais do Projeto

Cada módulo foi pensado para ser executado de forma independente. No entanto, quando integrados, o pipeline se torna elegante e completo. Cada módulo possui sua própria biblioteca, disponibilizando funções e estruturas para os módulos vizinhos.

Essa flexibilidade permite, de forma muito fluida, adaptar o cliente que chama essas *libs*. Neste projeto, optei por construir ferramentas de linha de comando (CLI - *Command Line Interface*) com suporte a parâmetros no estilo UNIX. Porém, graças a essa arquitetura modular, no futuro o cliente poderia facilmente ser uma interface de terminal (TUI - *Terminal User Interface*) ou até mesmo uma interface gráfica (GUI - *Graphical User Interface*).

### ⚡ Alta Performance: Zero-Copy & Lazy Lexing

O núcleo do nosso analisador léxico (*Lexer*) foi desenhado para ser extremamente rápido e eficiente no consumo de memória, tirando proveito de duas abordagens avançadas:

* **Zero-Copy com *Lifetimes* (`&'a str`):** Em vez de clonar textos e alocar novas `Strings` no *heap* para cada token encontrado, o Lexer utiliza o poderoso sistema de *lifetimes* do Rust. Ele opera exclusivamente com referências (*slices*) apontando diretamente para a string original do código-fonte em memória. O resultado é um processo de tokenização sem nenhuma alocação dinâmica desnecessária.
* **Avaliação Preguiçosa (*Lazy Evaluation*):** O Lexer não varre o arquivo inteiro para gerar uma lista gigante de tokens de uma só vez. Ele opera sob demanda. A cada ciclo, o *Parser* solicita o próximo token necessário (via `next_token()`). Isso significa que, se houver um erro de sintaxe logo na primeira linha, o compilador aborta a execução instantaneamente, sem desperdiçar ciclos de CPU tentando processar o resto do arquivo.
---

## 🚀 Como Executar o Projeto

> **Nota:** Este passo a passo foi pensado para ser executado em um ambiente Linux.

Como o projeto foi inteiramente construído em Rust, a forma mais prática, elegante e recomendada de compilar e instalar os binários globalmente no seu sistema é utilizando o próprio `cargo`.

Clone o repositório, navegue até a raiz do projeto e execute:

```bash
cargo install --path .
```

O que isso faz?
O Cargo fará a compilação de todo o projeto em modo release (máxima otimização) e colocará os executáveis compilados na sua pasta ~/.cargo/bin. A partir de agora, se esse caminho estiver no seu PATH, você poderá rodar as ferramentas do projeto no seu terminal a partir de qualquer diretório da sua máquina!

Ou se preferir pode executar os binários com

```bash
cargo run --bin <binario_alvo>
```

O projeto foi idealizado para que cada binário seja modular, o que permite executar cada um de forma separada com:

```bash
cargo run --bin cacl --help
cargo run --bin assembler --help
cargo run --bin interpreter --help
```

Ou, se tiver instalado os binários via cargo install, você pode fazer:

```bash
calc --help
assembler --help
interpreter --help
```

Cada programa possui sua documentação e forma de uso descritas. 

Os binários também podem ser usados em conjunto, uma vez que utilizam entrada e saída padrão (stdin e stdout):

```bash
calc --path exemplo.txt | assembler | interpreter
```

## 📝 To-Do List

Este é um projeto contínuo para a disciplina de Compiladores. Os próximos passos mapeados para a evolução da arquitetura são:

    [ ] Documentar o código: Adicionar docstrings padronizados em Rust e detalhar o funcionamento interno dos parsers.

    [ ] Implementar Divisão: Expandir o macro de compilação matemática para suportar operações de divisão inteira, controlando as variáveis temporárias e laços de subtração.

    [ ] Implementar Potenciação: Criar a expansão de expoentes (XY) garantindo a segurança no aninhamento de múltiplos registradores temporários e loops.

---

## Referências

* [NEANDER](https://www.inf.ufrgs.br/arq/wiki/doku.php?id=neander)
* [Rust By Example](https://doc.rust-lang.org/rust-by-example/index.html)
* [Crafting interpreters](https://craftinginterpreters.com/)
