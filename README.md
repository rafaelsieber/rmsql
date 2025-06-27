# RMSQL - Cliente MySQL Interativo esti## 📁 Sistema de Arquivos

O RMSQL organiza seus dados em:

### Configurações (`~/.config/rmsql/`)
- **`connections.json`**: Conexões salvas
- **`user_config.json`**: Configurações do usuário e bancos cadastrados

### Cache (`~/.cache/rmsql/`)
- **`sql_history.json`**: Histórico completo de comandos SQL cliente MySQL moderno e interativo com interface de terminal, inspirado no Vim, construído em Rust com sistema avançado de configuração e histórico.SQL - Cliente MySQL Interativo estilo Vim

Um cliente MySQL moderno e interativo com interface de terminal, inspirado no Vim, construído em Rust com sistema avançado de configuração e histórico.

## 🚀 Funcionalidades

### ✨ Principais Recursos

#### 1. Sistema de Configuração Inteligente
- **Auto-descoberta** de bancos de dados
- **Histórico persistente** de comandos SQL
- **Favoritos** e acesso rápido a bancos frequentes
- **Configurações personalizáveis** por usuário

#### 2. Editor SQL Interativo
- **Tecla**: `i` (para entrar no modo editor)
- Digite consultas SQL personalizadas
- **Recursos:**
  - Histórico de consultas persistente entre sessões
  - Navegação no histórico com `↑`/`↓`
  - Execução com `Enter`
  - Suporte a todas as consultas SQL (SELECT, INSERT, UPDATE, DELETE, etc.)
  - Métricas de performance (tempo de execução)
  - Mensagens de erro detalhadas

#### 3. Expansão Dinâmica de Colunas
- **Tecla**: `Espaço` (no modo visualização de dados)
- Alterna entre visualização normal e expandida das colunas
- Na visualização expandida:
  - Navegação horizontal com `←`/`→`
  - Melhor para visualizar dados com texto longo

#### 4. Gerenciamento Avançado de Conexões
- Salva automaticamente conexões utilizadas
- Interface intuitiva para gerenciar múltiplas conexões
- Reconexão automática em caso de perda de conexão

## 📁 Sistema de Arquivos

O RMSQL organiza seus dados em:

### Configurações (`~/.config/rmsql/`)
- **`connections.json`**: Conexões salvas
- **`user_config.json`**: Configurações do usuário e bancos cadastrados

### Cache (`~/.cache/rmsql/`)
- **`sql_history.json`**: Histórico completo de comandos SQL

## Pré-requisitos

1. **MySQL Server** rodando na máquina
2. **Rust** instalado (versão 1.70 ou superior)
3. **Conexão MySQL configurada**

## Configuração do MySQL (se necessário)

Para permitir conexão como root sem senha localmente:

```sql
-- Conecte no MySQL como root
mysql -u root -p

-- Execute os comandos:
ALTER USER 'root'@'localhost' IDENTIFIED WITH mysql_native_password BY '';
FLUSH PRIVILEGES;
```

## Instalação

1. Clone o repositório:
   ```bash
   git clone https://github.com/rafaelsieber/rmsql.git
   cd rmsql
   ```

2. Compile o projeto:
   ```bash
   cargo build --release
   ```

## Uso

### Executar como root (recomendado)
```bash
sudo ./target/release/rmsql
```

### Executar com usuário específico
```bash
./target/release/rmsql -u seu_usuario -p sua_senha
```

### Opções de linha de comando

```bash
./target/release/rmsql [OPTIONS]

Options:
  -h, --host <HOST>          MySQL host [default: localhost]
  -P, --port <PORT>          MySQL port [default: 3306]
  -u, --username <USERNAME>  MySQL username (default: root when running with sudo)
  -p, --password <PASSWORD>  MySQL password
  -d, --database <DATABASE>  Initial database to connect to
      --help                 Print help
```

## Interface e Navegação

### Comandos Vim-Inspired

| Tecla | Ação |
|-------|------|
| `j` ou `↓` | Mover para baixo |
| `k` ou `↑` | Mover para cima |
| `h` ou `←` | Voltar/Navegar para trás |
| `l` ou `→` ou `Enter` | Avançar/Entrar |
| `g` | Ir para o topo |
| `G` | Ir para o fim |
| `r` | Atualizar vista atual |
| `i` | Entrar no editor SQL |
| `Espaço` | Alternar expansão de colunas (modo dados) |
| `q` | Sair |
| `?` | Mostrar ajuda |

### Modos de Visualização

| Tecla | Modo | Descrição |
|-------|------|-----------|
| `1` | Connections | Gerenciar conexões MySQL |
| `2` | Databases | Lista bancos de dados da conexão ativa |
| `3` | Tables | Lista tabelas do banco selecionado |
| `4` | Data | Mostra dados da tabela selecionada |

### Editor SQL

No modo editor SQL (`i`):
- Digite suas consultas SQL
- `Enter`: Executar consulta
- `↑`/`↓`: Navegar no histórico de comandos
- `Esc`: Sair do modo editor
- Todas as consultas são automaticamente salvas no histórico

## 🔧 Recursos Avançados

### Histórico Inteligente
- **Persistência**: Comandos SQL salvos entre sessões
- **Métricas**: Tempo de execução de cada consulta
- **Contexto**: Database e conexão utilizados
- **Filtros**: Histórico por conexão ou database
- **Limite**: Controle automático do tamanho do histórico

### Configurações Personalizáveis
- **Auto-salvar histórico**: Ativado por padrão
- **Limite de entradas**: 1000 comandos (configurável)
- **Confirmação**: Para queries perigosas (DROP, DELETE)
- **Tempo de execução**: Exibição opcional de métricas

### Gerenciamento de Bancos
- **Auto-descoberta**: Bancos salvos automaticamente
- **Favoritos**: Marque bancos importantes
- **Último acesso**: Rastreamento automático
- **Acesso rápido**: Para bancos recentes e favoritos

## Recursos

- ✅ Interface vim-inspired intuitiva
- ✅ Navegação hierárquica (Connections → Databases → Tables → Data)
- ✅ Editor SQL interativo com histórico persistente
- ✅ Sistema de configuração avançado
- ✅ Métricas de performance para queries
- ✅ Gerenciamento de múltiplas conexões
- ✅ Auto-descoberta e favoritos de bancos
- ✅ Expansão dinâmica de colunas
- ✅ Histórico de comandos persistente
- ✅ Interface colorizada e responsiva

## Estrutura do Projeto

```
src/
├── main.rs              # Ponto de entrada e lógica principal
├── database.rs          # Gerenciamento de conexões e queries MySQL
├── navigation.rs        # Estado de navegação e controle de modos
├── ui.rs               # Interface do usuário com ratatui
├── connection_config.rs # Gerenciamento de configurações de conexão
├── connection_ui.rs    # Interface para gerenciamento de conexões
└── user_config.rs      # Sistema de configuração do usuário
```

## 📊 Benefícios

### Para Desenvolvedores
- **Histórico persistente**: Nunca perca comandos SQL importantes
- **Análise de performance**: Identifique queries lentas
- **Contexto preservado**: Saiba onde e quando executou cada comando
- **Interface produtiva**: Navegação rápida estilo vim

### Para DBAs
- **Auditoria**: Rastreamento completo de atividades
- **Debugging**: Histórico de erros com contexto completo
- **Eficiência**: Acesso rápido a bancos frequentes
- **Múltiplas conexões**: Gerencie vários ambientes

### Para Times
- **Consistência**: Configurações padronizadas
- **Colaboração**: Compartilhamento fácil de configurações
- **Segurança**: Controle sobre queries perigosas

## Exemplos de Uso

### Conectar como root via sudo:
```bash
sudo ./target/release/rmsql
```

### Conectar com credenciais específicas:
```bash
./target/release/rmsql -u admin -p minhasenha -h 192.168.1.100
```

### Navegar:
1. Inicie o programa
2. Use `j/k` para navegar pelos bancos
3. Pressione `Enter` em um banco para ver suas tabelas
4. Pressione `Enter` em uma tabela para ver seus dados
5. Use `h` para voltar ao nível anterior
6. Pressione `q` para sair

## Dependências

- `mysql` - Driver MySQL nativo para Rust
- `ratatui` - Framework moderno para interfaces de terminal
- `crossterm` - Controle de terminal multiplataforma
- `anyhow` - Tratamento robusto de erros
- `clap` - Parser elegante de argumentos CLI
- `serde` - Serialização/deserialização de dados
- `chrono` - Manipulação de datas e horários
- `dirs` - Acesso a diretórios do sistema
- `uuid` - Geração de identificadores únicos

## 🔐 Privacidade e Segurança

- **Local apenas**: Todos os dados ficam na máquina do usuário
- **Não compartilhado**: Nenhuma informação enviada para servidores externos
- **Controle total**: Usuário pode limpar histórico a qualquer momento
- **Separação clara**: Configurações e cache em diretórios apropriados
- **Senhas seguras**: Não armazenadas em texto plano

## Limitações Atuais

- Mostra até 100 linhas por padrão (configurável)
- Senhas não são salvas por segurança
- Interface otimizada para terminais com largura mínima de 80 caracteres

## Roadmap

Funcionalidades planejadas:
- [ ] Exportação de dados (CSV, JSON)
- [ ] Importação de dados
- [ ] Editor de esquemas
- [ ] Backup e restore
- [ ] Suporte a PostgreSQL
- [ ] Plugin system
- [ ] Temas customizáveis

## Troubleshooting

### Erro de conexão
- Verifique se o MySQL está rodando: `sudo systemctl status mysql`
- Teste a conexão: `mysql -u root`
- Verifique as credenciais

### Erro de permissão
- Execute com `sudo` para usar credenciais root
- Ou especifique usuário e senha: `-u usuario -p senha`

### Interface não funciona corretamente
- Certifique-se de que o terminal suporta cores
- Redimensione o terminal se necessário
- Alguns terminais podem não suportar todos os caracteres especiais
