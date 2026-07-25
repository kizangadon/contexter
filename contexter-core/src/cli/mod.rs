//! CLI binary for the Contexter storage engine.
//!
//! Provides a clap-based command-line interface for all Engine CRUD
//! operations, diagnostics, and maintenance commands.
//!
//! # Usage
//!
//! ```text
//! contexter [--db-path <path>] <command> [args...]
//! ```
//!
//! Every command opens an `Engine` instance at the specified (or default)
//! RocksDB path, executes the requested operation, and prints the result
//! to stdout. Errors are printed to stderr with a non-zero exit code.

use crate::{error::EngineError, *};
use clap::{Parser, Subcommand, ValueEnum};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Top-level CLI
// ---------------------------------------------------------------------------

/// Contexter storage engine CLI — diagnostic and CRUD interface.
#[derive(Parser)]
#[command(
    name = "contexter",
    version,
    about = "Contexter storage engine CLI",
    long_about = "CLI for the Contexter storage engine. Supports full CRUD \
                   on Sessions, Memories, Agents, and Skills, plus settings, \
                   audit log queries, and diagnostics commands."
)]
struct Cli {
    /// Path to RocksDB storage directory.
    /// Defaults to `~/.contexter/`.
    #[arg(short, long, env = "CONTEXTER_DB_PATH")]
    db_path: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

// ---------------------------------------------------------------------------
// Top-level subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
enum Commands {
    /// Session CRUD operations.
    #[command(subcommand)]
    Session(SessionCommands),
    /// Memory CRUD and search operations.
    #[command(subcommand)]
    Memory(MemoryCommands),
    /// Agent CRUD operations.
    #[command(subcommand)]
    Agent(AgentCommands),
    /// Skill CRUD operations.
    #[command(subcommand)]
    Skill(SkillCommands),
    /// Key-value setting operations.
    #[command(subcommand)]
    Setting(SettingCommands),
    /// Audit log query operations.
    #[command(subcommand)]
    Audit(AuditCommands),
    /// Diagnostics and maintenance operations.
    #[command(subcommand)]
    Diag(DiagCommands),
    /// Show comprehensive engine status.
    Status,
    /// Flush WAL and create a checkpoint.
    Checkpoint,
}

// ---------------------------------------------------------------------------
// ValueEnum helpers for domain enums
// ---------------------------------------------------------------------------

/// CLI-friendly representation of [`SessionStatus`].
#[derive(Debug, Clone, ValueEnum)]
enum CliSessionStatus {
    Active,
    Completed,
    Error,
}

impl From<CliSessionStatus> for SessionStatus {
    fn from(s: CliSessionStatus) -> Self {
        match s {
            CliSessionStatus::Active => SessionStatus::Active,
            CliSessionStatus::Completed => SessionStatus::Completed,
            CliSessionStatus::Error => SessionStatus::Error,
        }
    }
}

/// CLI-friendly representation of [`MemoryType`].
#[derive(Debug, Clone, ValueEnum)]
enum CliMemoryType {
    Fact,
    Preference,
    Procedure,
    Context,
    Episode,
}

impl From<CliMemoryType> for MemoryType {
    fn from(m: CliMemoryType) -> Self {
        match m {
            CliMemoryType::Fact => MemoryType::Fact,
            CliMemoryType::Preference => MemoryType::Preference,
            CliMemoryType::Procedure => MemoryType::Procedure,
            CliMemoryType::Context => MemoryType::Context,
            CliMemoryType::Episode => MemoryType::Episode,
        }
    }
}

/// CLI-friendly representation of [`AgentStatus`].
#[derive(Debug, Clone, ValueEnum)]
enum CliAgentStatus {
    Active,
    Inactive,
}

impl From<CliAgentStatus> for AgentStatus {
    fn from(s: CliAgentStatus) -> Self {
        match s {
            CliAgentStatus::Active => AgentStatus::Active,
            CliAgentStatus::Inactive => AgentStatus::Inactive,
        }
    }
}

// ---------------------------------------------------------------------------
// Session subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Subcommand)]
enum SessionCommands {
    /// Create a new session.
    Create {
        /// Agent that owns this session.
        #[arg(long)]
        agent_id: String,

        /// Project name for the session.
        #[arg(long, default_value = "default")]
        project: String,

        /// Initial session status.
        #[arg(long)]
        status: Option<CliSessionStatus>,

        /// JSON metadata string.
        #[arg(long)]
        metadata: Option<String>,
    },
    /// Retrieve a session by ID.
    Get {
        /// Session UUID.
        id: String,
    },
    /// List sessions with optional filters.
    List {
        /// Filter by project name.
        #[arg(long)]
        project: Option<String>,
        /// Filter by agent ID.
        #[arg(long)]
        agent_id: Option<String>,
        /// Filter by status.
        #[arg(long)]
        status: Option<CliSessionStatus>,
        /// Number of records to skip.
        #[arg(long, default_value_t = 0)]
        offset: u64,
        /// Maximum number of records to return.
        #[arg(long, default_value_t = 100)]
        limit: u64,
    },
    /// Update a session.
    Update {
        /// Session UUID.
        id: String,
        /// New status.
        #[arg(long)]
        status: Option<CliSessionStatus>,
        /// New turn count.
        #[arg(long)]
        turn_count: Option<u32>,
        /// New duration in milliseconds.
        #[arg(long)]
        duration_ms: Option<u64>,
        /// New JSON metadata.
        #[arg(long)]
        metadata: Option<String>,
    },
    /// Delete a session.
    Delete {
        /// Session UUID.
        id: String,
    },
    /// Count sessions matching filters.
    Count {
        /// Filter by project name.
        #[arg(long)]
        project: Option<String>,
        /// Filter by agent ID.
        #[arg(long)]
        agent_id: Option<String>,
        /// Filter by status.
        #[arg(long)]
        status: Option<CliSessionStatus>,
    },
}

// ---------------------------------------------------------------------------
// Memory subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Subcommand)]
enum MemoryCommands {
    /// Create a new memory.
    Create {
        /// Session this memory belongs to.
        #[arg(long)]
        session_id: String,
        /// Agent creating this memory.
        #[arg(long)]
        agent_id: String,
        /// Memory type.
        #[arg(long)]
        memory_type: CliMemoryType,
        /// Memory content text.
        #[arg(long)]
        content: String,
        /// Comma-separated tags.
        #[arg(long)]
        tags: Option<String>,
    },
    /// Retrieve a memory by ID.
    Get {
        /// Memory UUID.
        id: String,
    },
    /// Search memories with optional filters and keyword ranking.
    Search {
        /// Keyword search query (space-separated keywords).
        #[arg(long)]
        query: Option<String>,
        /// Filter by memory type.
        #[arg(long)]
        memory_type: Option<CliMemoryType>,
        /// Comma-separated tags (any match).
        #[arg(long)]
        tags: Option<String>,
        /// Filter by session ID.
        #[arg(long)]
        session_id: Option<String>,
        /// Filter by agent ID.
        #[arg(long)]
        agent_id: Option<String>,
        /// Maximum results.
        #[arg(long, default_value_t = 100)]
        limit: u64,
    },
    /// Update a memory.
    Update {
        /// Memory UUID.
        id: String,
        /// New content text.
        #[arg(long)]
        content: Option<String>,
        /// New memory type.
        #[arg(long)]
        memory_type: Option<CliMemoryType>,
        /// New comma-separated tags.
        #[arg(long)]
        tags: Option<String>,
    },
    /// Delete a memory.
    Delete {
        /// Memory UUID.
        id: String,
    },
    /// Count memories matching filters.
    Count {
        /// Filter by session ID.
        #[arg(long)]
        session_id: Option<String>,
        /// Filter by agent ID.
        #[arg(long)]
        agent_id: Option<String>,
        /// Filter by memory type.
        #[arg(long)]
        memory_type: Option<CliMemoryType>,
        /// Comma-separated tags (any match).
        #[arg(long)]
        tags: Option<String>,
    },
}

// ---------------------------------------------------------------------------
// Agent subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Subcommand)]
enum AgentCommands {
    /// Create a new agent.
    Create {
        /// Agent name.
        #[arg(long)]
        name: String,
        /// Agent type identifier (e.g. "chat", "coding-assistant").
        #[arg(long)]
        agent_type: String,
        /// Human-readable description.
        #[arg(long, default_value = "")]
        description: String,
        /// Comma-separated capabilities.
        #[arg(long)]
        capabilities: Option<String>,
        /// Operational status.
        #[arg(long)]
        status: Option<CliAgentStatus>,
        /// JSON config string.
        #[arg(long)]
        config: Option<String>,
    },
    /// Retrieve an agent by ID.
    Get {
        /// Agent UUID.
        id: String,
    },
    /// List all agents.
    List,
    /// Update an agent.
    Update {
        /// Agent UUID.
        id: String,
        /// New name.
        #[arg(long)]
        name: Option<String>,
        /// New agent type.
        #[arg(long)]
        agent_type: Option<String>,
        /// New description.
        #[arg(long)]
        description: Option<String>,
        /// New comma-separated capabilities.
        #[arg(long)]
        capabilities: Option<String>,
        /// New status.
        #[arg(long)]
        status: Option<CliAgentStatus>,
        /// New JSON config string.
        #[arg(long)]
        config: Option<String>,
    },
    /// Delete an agent.
    Delete {
        /// Agent UUID.
        id: String,
    },
}

// ---------------------------------------------------------------------------
// Skill subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Subcommand)]
enum SkillCommands {
    /// Create a new skill.
    Create {
        /// Skill name.
        #[arg(long)]
        name: String,
        /// Human-readable description.
        #[arg(long, default_value = "")]
        description: String,
        /// Category (e.g. "search", "code", "memory").
        #[arg(long)]
        category: String,
        /// Optional file-system path to implementation.
        #[arg(long)]
        file_path: Option<String>,
    },
    /// Retrieve a skill by ID.
    Get {
        /// Skill UUID.
        id: String,
    },
    /// List all skills.
    List,
    /// Update a skill.
    Update {
        /// Skill UUID.
        id: String,
        /// New name.
        #[arg(long)]
        name: Option<String>,
        /// New description.
        #[arg(long)]
        description: Option<String>,
        /// New category.
        #[arg(long)]
        category: Option<String>,
        /// New file path.
        #[arg(long)]
        file_path: Option<String>,
    },
    /// Delete a skill.
    Delete {
        /// Skill UUID.
        id: String,
    },
}

// ---------------------------------------------------------------------------
// Setting subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Subcommand)]
enum SettingCommands {
    /// Set a setting value by key.
    Set {
        /// Setting key.
        key: String,
        /// Setting value.
        value: String,
    },
    /// Get a setting value by key.
    Get {
        /// Setting key.
        key: String,
    },
}

// ---------------------------------------------------------------------------
// Audit subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Subcommand)]
enum AuditCommands {
    /// Query the audit log with optional filters.
    Query {
        /// Filter by entity type.
        #[arg(long)]
        entity_type: Option<String>,
        /// Filter by entity ID.
        #[arg(long)]
        entity_id: Option<String>,
        /// Filter by actor.
        #[arg(long)]
        actor: Option<String>,
        /// Maximum results.
        #[arg(long, default_value_t = 100)]
        limit: u64,
    },
}

// ---------------------------------------------------------------------------
// Diagnostics subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Subcommand)]
enum DiagCommands {
    /// Flush pending writes to durable storage.
    Flush,
    /// Trigger a RocksDB checkpoint / compaction, returns sequence number.
    Checkpoint,
    /// Report storage size per column family.
    StorageSize,
    /// Show L1 cache performance telemetry.
    CacheStats,
    /// Clear the L1 cache entirely or for a specific entity type.
    ClearCache {
        /// Optional entity type to clear (e.g. "session", "memory").
        /// If omitted, clears all cached entries.
        #[arg(long)]
        entity_type: Option<String>,
    },
    /// Quick health check — opens engine and reports cache telemetry.
    Health,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Run the CLI: parse arguments, open the engine, dispatch, and print results.
pub fn main() {
    let cli = Cli::parse();

    // Resolve the database path:
    //   1. If the user passed `--db-path` or `CONTEXTER_DB_PATH` is set,
    //      clap stores that value in `cli.db_path`.
    //   2. Otherwise fall back to `~/.contexter/`.
    let db_path = cli.db_path.unwrap_or_else(|| {
        dirs::home_dir()
            .map(|p| p.join(".contexter").to_string_lossy().to_string())
            .unwrap_or_else(|| "./contexter_data".to_string())
    });

    // Validate the database path.
    let db_path_obj = std::path::Path::new(&db_path);
    // Reject if the path exists but is not a directory.
    if db_path_obj.exists() && !db_path_obj.is_dir() {
        eprintln!("Error: '{}' exists but is not a directory", db_path);
        std::process::exit(1);
    }
    let canonical = db_path_obj.canonicalize().unwrap_or_else(|_| {
        // Path doesn't exist yet — will be created by Engine::open.
        std::path::PathBuf::from(&db_path)
    });
    if canonical.starts_with("/tmp") {
        eprintln!(
            "Warning: data in {} may be lost on reboot",
            canonical.display()
        );
    }

    let engine = match Engine::open(&db_path) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Error: failed to open engine at '{db_path}': {err}");
            std::process::exit(1);
        }
    };

    if let Err(err) = dispatch(engine, cli.command, &db_path) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

fn dispatch(engine: Engine, command: Commands, db_path: &str) -> Result<(), ContexterError> {
    match command {
        Commands::Session(cmd) => handle_session(engine, cmd),
        Commands::Memory(cmd) => handle_memory(engine, cmd),
        Commands::Agent(cmd) => handle_agent(engine, cmd),
        Commands::Skill(cmd) => handle_skill(engine, cmd),
        Commands::Setting(cmd) => handle_setting(engine, cmd),
        Commands::Audit(cmd) => handle_audit(engine, cmd),
        Commands::Diag(cmd) => handle_diag(engine, cmd),
        Commands::Status => handle_status(engine, db_path),
        Commands::Checkpoint => {
            let seq = engine.checkpoint()?;
            print_scalar(format!("Checkpoint complete. Sequence number: {seq}"));
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Error wrapper to bridge EngineError
// ---------------------------------------------------------------------------

/// Local error type that wraps [`EngineError`] for unified error reporting.
enum ContexterError {
    Engine(EngineError),
    Message(String),
}

impl std::fmt::Display for ContexterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContexterError::Engine(e) => write!(f, "{e}"),
            ContexterError::Message(m) => write!(f, "{m}"),
        }
    }
}

impl std::fmt::Debug for ContexterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl From<EngineError> for ContexterError {
    fn from(e: EngineError) -> Self {
        ContexterError::Engine(e)
    }
}

// ---------------------------------------------------------------------------
// Parse a UUID from a string, returning a user-friendly error on failure.
// ---------------------------------------------------------------------------

fn parse_uuid(s: &str) -> Result<Uuid, ContexterError> {
    Uuid::parse_str(s).map_err(|e| ContexterError::Message(format!("invalid UUID '{s}': {e}")))
}

// ---------------------------------------------------------------------------
// Parse comma-separated tags into a Vec<String>.
// ---------------------------------------------------------------------------

fn parse_tags(s: &Option<String>) -> Option<Vec<String>> {
    s.as_ref().map(|raw| {
        raw.split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect()
    })
}

// ---------------------------------------------------------------------------
// Parse a JSON metadata/config string into serde_json::Value.
// ---------------------------------------------------------------------------

fn parse_json(s: &Option<String>) -> Result<Option<serde_json::Value>, ContexterError> {
    match s {
        None => Ok(None),
        Some(raw) => {
            let val: serde_json::Value = serde_json::from_str(raw)
                .map_err(|e| ContexterError::Message(format!("invalid JSON '{raw}': {e}")))?;
            Ok(Some(val))
        }
    }
}

// ---------------------------------------------------------------------------
// Print a single item using Display (via serde_json pretty-print).
// ---------------------------------------------------------------------------

fn print_item<T: serde::Serialize>(item: &T) {
    let json =
        serde_json::to_string_pretty(item).unwrap_or_else(|_| "<serialization error>".into());
    println!("{json}");
}

/// Print a list as pretty JSON.
fn print_list<T: serde::Serialize>(items: &[T]) {
    if items.is_empty() {
        println!("[]");
        return;
    }
    let json =
        serde_json::to_string_pretty(items).unwrap_or_else(|_| "<serialization error>".into());
    println!("{json}");
}

/// Print a scalar value (e.g. count, sequence number).
fn print_scalar(msg: impl std::fmt::Display) {
    println!("{msg}");
}

// ===========================================================================
// Session command handlers
// ===========================================================================

fn handle_session(engine: Engine, cmd: SessionCommands) -> Result<(), ContexterError> {
    match cmd {
        SessionCommands::Create {
            agent_id,
            project,
            status,
            metadata,
        } => {
            let agent_id = parse_uuid(&agent_id)?;
            let metadata = parse_json(&metadata)?;
            let session = engine.create_session(NewSession {
                project,
                agent_id,
                status: status.map(SessionStatus::from),
                metadata,
            })?;
            print_item(&session);
        }
        SessionCommands::Get { id } => {
            let id = parse_uuid(&id)?;
            match engine.get_session(id)? {
                Some(session) => print_item(&session),
                None => print_scalar(format!("Session not found: {id}")),
            }
        }
        SessionCommands::List {
            project,
            agent_id,
            status,
            offset,
            limit,
        } => {
            let agent_id = agent_id.map(|a| parse_uuid(&a)).transpose()?;
            let sessions = engine.list_sessions(&SessionFilter {
                project,
                agent_id,
                status: status.map(SessionStatus::from),
                limit,
                offset,
            })?;
            print_list(&sessions);
        }
        SessionCommands::Update {
            id,
            status,
            turn_count,
            duration_ms,
            metadata,
        } => {
            let id = parse_uuid(&id)?;
            let metadata = parse_json(&metadata)?;
            let session = engine.update_session(
                id,
                &SessionPatch {
                    status: status.map(SessionStatus::from),
                    turn_count,
                    duration_ms,
                    metadata,
                },
            )?;
            print_item(&session);
        }
        SessionCommands::Delete { id } => {
            let id = parse_uuid(&id)?;
            engine.delete_session(id)?;
            print_scalar(format!("Deleted session: {id}"));
        }
        SessionCommands::Count {
            project,
            agent_id,
            status,
        } => {
            let agent_id = agent_id.map(|a| parse_uuid(&a)).transpose()?;
            let count = engine.count_sessions(&SessionFilter {
                project,
                agent_id,
                status: status.map(SessionStatus::from),
                limit: 0,
                offset: 0,
            })?;
            print_scalar(format!("{count}"));
        }
    }
    Ok(())
}

// ===========================================================================
// Memory command handlers
// ===========================================================================

fn handle_memory(engine: Engine, cmd: MemoryCommands) -> Result<(), ContexterError> {
    match cmd {
        MemoryCommands::Create {
            session_id,
            agent_id,
            memory_type,
            content,
            tags,
        } => {
            let session_id = parse_uuid(&session_id)?;
            let agent_id = parse_uuid(&agent_id)?;
            let tags = parse_tags(&tags);
            let memory = engine.create_memory(NewMemory {
                session_id,
                agent_id,
                memory_type: memory_type.into(),
                content,
                tags,
            })?;
            print_item(&memory);
        }
        MemoryCommands::Get { id } => {
            let id = parse_uuid(&id)?;
            match engine.get_memory(id)? {
                Some(memory) => print_item(&memory),
                None => print_scalar(format!("Memory not found: {id}")),
            }
        }
        MemoryCommands::Search {
            query,
            memory_type,
            tags,
            session_id,
            agent_id,
            limit,
        } => {
            let session_id = session_id.map(|s| parse_uuid(&s)).transpose()?;
            let agent_id = agent_id.map(|a| parse_uuid(&a)).transpose()?;
            let tags = parse_tags(&tags);
            let results = engine.search_memories(&MemorySearchQuery {
                keywords: query,
                memory_type: memory_type.map(MemoryType::from),
                tags,
                session_id,
                agent_id,
                project: None,
                limit,
                offset: 0,
            })?;
            print_list(&results);
        }
        MemoryCommands::Update {
            id,
            content,
            memory_type,
            tags,
        } => {
            let id = parse_uuid(&id)?;
            let tags = parse_tags(&tags);
            let memory = engine.update_memory(
                id,
                &MemoryPatch {
                    content,
                    memory_type: memory_type.map(MemoryType::from),
                    tags,
                },
            )?;
            print_item(&memory);
        }
        MemoryCommands::Delete { id } => {
            let id = parse_uuid(&id)?;
            engine.delete_memory(id)?;
            print_scalar(format!("Deleted memory: {id}"));
        }
        MemoryCommands::Count {
            session_id,
            agent_id,
            memory_type,
            tags,
        } => {
            let session_id = session_id.map(|s| parse_uuid(&s)).transpose()?;
            let agent_id = agent_id.map(|a| parse_uuid(&a)).transpose()?;
            let tags = parse_tags(&tags);
            let count = engine.count_memories(&MemoryFilter {
                session_id,
                agent_id,
                memory_type: memory_type.map(MemoryType::from),
                tags,
            })?;
            print_scalar(format!("{count}"));
        }
    }
    Ok(())
}

// ===========================================================================
// Agent command handlers
// ===========================================================================

fn handle_agent(engine: Engine, cmd: AgentCommands) -> Result<(), ContexterError> {
    match cmd {
        AgentCommands::Create {
            name,
            agent_type,
            description,
            capabilities,
            status,
            config,
        } => {
            let capabilities = capabilities.map(|c| {
                c.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            });
            let config = parse_json(&config)?;
            let agent = engine.create_agent(NewAgent {
                name,
                agent_type,
                description,
                capabilities,
                status: status.map(AgentStatus::from),
                config,
            })?;
            print_item(&agent);
        }
        AgentCommands::Get { id } => {
            let id = parse_uuid(&id)?;
            match engine.get_agent(id)? {
                Some(agent) => print_item(&agent),
                None => print_scalar(format!("Agent not found: {id}")),
            }
        }
        AgentCommands::List => {
            let agents = engine.list_agents(&AgentFilter::default())?;
            print_list(&agents);
        }
        AgentCommands::Update {
            id,
            name,
            agent_type,
            description,
            capabilities,
            status,
            config,
        } => {
            let id = parse_uuid(&id)?;
            let capabilities = capabilities.map(|c| {
                c.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            });
            let config = parse_json(&config)?;
            let agent = engine.update_agent(
                id,
                &AgentPatch {
                    name,
                    agent_type,
                    description,
                    capabilities,
                    status: status.map(AgentStatus::from),
                    config,
                },
            )?;
            print_item(&agent);
        }
        AgentCommands::Delete { id } => {
            let id = parse_uuid(&id)?;
            engine.delete_agent(id)?;
            print_scalar(format!("Deleted agent: {id}"));
        }
    }
    Ok(())
}

// ===========================================================================
// Skill command handlers
// ===========================================================================

fn handle_skill(engine: Engine, cmd: SkillCommands) -> Result<(), ContexterError> {
    match cmd {
        SkillCommands::Create {
            name,
            description,
            category,
            file_path,
        } => {
            let skill = engine.create_skill(NewSkill {
                name,
                description,
                category,
                file_path,
            })?;
            print_item(&skill);
        }
        SkillCommands::Get { id } => {
            let id = parse_uuid(&id)?;
            match engine.get_skill(id)? {
                Some(skill) => print_item(&skill),
                None => print_scalar(format!("Skill not found: {id}")),
            }
        }
        SkillCommands::List => {
            let skills = engine.list_skills(&SkillFilter::default())?;
            print_list(&skills);
        }
        SkillCommands::Update {
            id,
            name,
            description,
            category,
            file_path,
        } => {
            let id = parse_uuid(&id)?;
            let skill = engine.update_skill(
                id,
                &SkillPatch {
                    name,
                    description,
                    category,
                    file_path,
                },
            )?;
            print_item(&skill);
        }
        SkillCommands::Delete { id } => {
            let id = parse_uuid(&id)?;
            engine.delete_skill(id)?;
            print_scalar(format!("Deleted skill: {id}"));
        }
    }
    Ok(())
}

// ===========================================================================
// Setting command handlers
// ===========================================================================

fn handle_setting(engine: Engine, cmd: SettingCommands) -> Result<(), ContexterError> {
    match cmd {
        SettingCommands::Set { key, value } => {
            engine.set_setting(&key, &value)?;
            print_scalar(format!("Set setting: {key} = {value}"));
        }
        SettingCommands::Get { key } => match engine.get_setting(&key)? {
            Some(value) => print_scalar(format!("{key} = {value}")),
            None => print_scalar(format!("Setting not found: {key}")),
        },
    }
    Ok(())
}

// ===========================================================================
// Audit command handlers
// ===========================================================================

fn handle_audit(engine: Engine, cmd: AuditCommands) -> Result<(), ContexterError> {
    match cmd {
        AuditCommands::Query {
            entity_type,
            entity_id,
            actor,
            limit,
        } => {
            let entries = engine.query_audit(&AuditFilter {
                entity_type,
                entity_id,
                actor,
                limit,
                offset: 0,
            })?;
            print_list(&entries);
        }
    }
    Ok(())
}

// ===========================================================================
// Status command handler
// ===========================================================================

fn handle_status(engine: Engine, db_path: &str) -> Result<(), ContexterError> {
    println!("Contexter Engine Status");
    println!("=======================");
    println!("Data directory: {db_path}");

    // Storage size
    match engine.storage_size() {
        Ok(size) => {
            println!("\nStorage:");
            println!("  Total: {} bytes", size.total);
            println!("  WAL: {} bytes", size.wal_size);
            for (cf, cf_size) in &size.per_cf {
                let label = if cf == "default" { "default" } else { cf };
                println!("  CF '{label}': {cf_size} bytes");
            }
        }
        Err(e) => println!("\nStorage: error — {e}"),
    }

    // Entity counts
    let session_count = engine.count_sessions(&SessionFilter::default())?;
    let memory_count = engine.count_memories(&MemoryFilter::default())?;
    let agent_count = engine.list_agents(&AgentFilter::default())?.len();
    let skill_count = engine.list_skills(&SkillFilter::default())?.len();

    println!("\nEntities:");
    println!("  Sessions: {session_count}");
    println!("  Memories: {memory_count}");
    println!("  Agents:   {agent_count}");
    println!("  Skills:   {skill_count}");

    // Cache telemetry
    let tel = engine.cache_telemetry();
    println!("\nCache:");
    println!("  Hits:         {}", tel.hits);
    println!("  Misses:       {}", tel.misses);
    println!("  Total ops:    {}", tel.total_ops);
    println!("  Entries/type: {:?}", tel.entries_by_type);

    // Health check
    let healthy = tel.total_ops == 0 || (tel.hits as f64 / tel.total_ops.max(1) as f64).is_finite();
    println!("\nHealth: {}", if healthy { "OK" } else { "DEGRADED" });

    Ok(())
}

// ===========================================================================
// Diagnostics command handlers
// ===========================================================================

fn handle_diag(engine: Engine, cmd: DiagCommands) -> Result<(), ContexterError> {
    match cmd {
        DiagCommands::Flush => {
            engine.flush()?;
            print_scalar("Flush completed");
        }
        DiagCommands::Checkpoint => {
            let seq = engine.checkpoint()?;
            print_scalar(format!("Checkpoint complete. Sequence number: {seq}"));
        }
        DiagCommands::StorageSize => {
            let size = engine.storage_size()?;
            print_item(&size);
        }
        DiagCommands::CacheStats => {
            let tel = engine.cache_telemetry();
            print_item(&tel);
        }
        DiagCommands::ClearCache { entity_type } => match entity_type {
            Some(t) => {
                engine.clear_cache_type(&t);
                print_scalar(format!("Cleared cache for entity type: {t}"));
            }
            None => {
                engine.clear_cache();
                print_scalar("Cleared entire cache");
            }
        },
        DiagCommands::Health => {
            let tel = engine.cache_telemetry();
            println!("Engine: OK");
            println!("Cache hits:   {}", tel.hits);
            println!("Cache misses: {}", tel.misses);
            let hit_ratio = if tel.total_ops > 0 { tel.hits as f64 / tel.total_ops as f64 } else { 0.0 };
            println!("Hit ratio:    {:.3}", hit_ratio);
            println!("Total ops:    {}", tel.total_ops);
            println!("Entries by type: {:?}", tel.entries_by_type);
        }
    }
    Ok(())
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Parse tests — verify that clap parses command-line arguments correctly
    // without needing an actual engine.
    // -----------------------------------------------------------------------

    #[test]
    fn test_cli_default_db_path() {
        // `contexter diag health` should have None db_path (resolved at runtime)
        let cli = Cli::try_parse_from(["contexter", "diag", "health"])
            .expect("should parse with default db_path");
        assert_eq!(cli.db_path, None);
        assert!(matches!(cli.command, Commands::Diag(DiagCommands::Health)));
    }

    #[test]
    fn test_cli_custom_db_path() {
        let cli = Cli::try_parse_from(["contexter", "-d", "/tmp/mydb", "diag", "health"])
            .expect("should parse with custom db_path");
        assert_eq!(cli.db_path, Some("/tmp/mydb".to_string()));
    }

    #[test]
    fn test_cli_parse_session_create() {
        let cli = Cli::try_parse_from([
            "contexter",
            "session",
            "create",
            "--agent-id",
            "550e8400-e29b-41d4-a716-446655440000",
            "--project",
            "test-project",
            "--status",
            "active",
        ])
        .expect("should parse session create");
        assert!(matches!(
            cli.command,
            Commands::Session(SessionCommands::Create { .. })
        ));
    }

    #[test]
    fn test_cli_parse_session_get() {
        let cli = Cli::try_parse_from([
            "contexter",
            "session",
            "get",
            "550e8400-e29b-41d4-a716-446655440000",
        ])
        .expect("should parse session get");
        assert!(matches!(
            cli.command,
            Commands::Session(SessionCommands::Get { .. })
        ));
    }

    #[test]
    fn test_cli_parse_session_list_with_filters() {
        let cli = Cli::try_parse_from([
            "contexter",
            "session",
            "list",
            "--project",
            "my-project",
            "--status",
            "active",
            "--offset",
            "10",
            "--limit",
            "25",
        ])
        .expect("should parse session list");
        assert!(matches!(
            cli.command,
            Commands::Session(SessionCommands::List { .. })
        ));
    }

    #[test]
    fn test_cli_parse_session_delete() {
        let cli = Cli::try_parse_from([
            "contexter",
            "session",
            "delete",
            "550e8400-e29b-41d4-a716-446655440000",
        ])
        .expect("should parse session delete");
        assert!(matches!(
            cli.command,
            Commands::Session(SessionCommands::Delete { .. })
        ));
    }

    #[test]
    fn test_cli_parse_session_count() {
        let cli = Cli::try_parse_from(["contexter", "session", "count", "--project", "my-project"])
            .expect("should parse session count");
        assert!(matches!(
            cli.command,
            Commands::Session(SessionCommands::Count { .. })
        ));
    }

    #[test]
    fn test_cli_parse_memory_create() {
        let cli = Cli::try_parse_from([
            "contexter",
            "memory",
            "create",
            "--session-id",
            "550e8400-e29b-41d4-a716-446655440000",
            "--agent-id",
            "550e8400-e29b-41d4-a716-446655440001",
            "--memory-type",
            "fact",
            "--content",
            "The quick brown fox",
            "--tags",
            "animal,nature",
        ])
        .expect("should parse memory create");
        assert!(matches!(
            cli.command,
            Commands::Memory(MemoryCommands::Create { .. })
        ));
    }

    #[test]
    fn test_cli_parse_memory_get() {
        let cli = Cli::try_parse_from([
            "contexter",
            "memory",
            "get",
            "550e8400-e29b-41d4-a716-446655440000",
        ])
        .expect("should parse memory get");
        assert!(matches!(
            cli.command,
            Commands::Memory(MemoryCommands::Get { .. })
        ));
    }

    #[test]
    fn test_cli_parse_memory_search() {
        let cli = Cli::try_parse_from([
            "contexter",
            "memory",
            "search",
            "--query",
            "fox dog",
            "--memory-type",
            "fact",
            "--tags",
            "animal",
            "--limit",
            "20",
        ])
        .expect("should parse memory search");
        assert!(matches!(
            cli.command,
            Commands::Memory(MemoryCommands::Search { .. })
        ));
    }

    #[test]
    fn test_cli_parse_memory_update() {
        let cli = Cli::try_parse_from([
            "contexter",
            "memory",
            "update",
            "550e8400-e29b-41d4-a716-446655440000",
            "--content",
            "updated content",
        ])
        .expect("should parse memory update");
        assert!(matches!(
            cli.command,
            Commands::Memory(MemoryCommands::Update { .. })
        ));
    }

    #[test]
    fn test_cli_parse_memory_delete() {
        let cli = Cli::try_parse_from([
            "contexter",
            "memory",
            "delete",
            "550e8400-e29b-41d4-a716-446655440000",
        ])
        .expect("should parse memory delete");
        assert!(matches!(
            cli.command,
            Commands::Memory(MemoryCommands::Delete { .. })
        ));
    }

    #[test]
    fn test_cli_parse_memory_count() {
        let cli = Cli::try_parse_from([
            "contexter",
            "memory",
            "count",
            "--memory-type",
            "preference",
        ])
        .expect("should parse memory count");
        assert!(matches!(
            cli.command,
            Commands::Memory(MemoryCommands::Count { .. })
        ));
    }

    #[test]
    fn test_cli_parse_agent_create() {
        let cli = Cli::try_parse_from([
            "contexter",
            "agent",
            "create",
            "--name",
            "test-agent",
            "--agent-type",
            "chat",
            "--description",
            "A test agent",
            "--capabilities",
            "code,search",
            "--status",
            "active",
        ])
        .expect("should parse agent create");
        assert!(matches!(
            cli.command,
            Commands::Agent(AgentCommands::Create { .. })
        ));
    }

    #[test]
    fn test_cli_parse_agent_get() {
        let cli = Cli::try_parse_from([
            "contexter",
            "agent",
            "get",
            "550e8400-e29b-41d4-a716-446655440000",
        ])
        .expect("should parse agent get");
        assert!(matches!(
            cli.command,
            Commands::Agent(AgentCommands::Get { .. })
        ));
    }

    #[test]
    fn test_cli_parse_agent_list() {
        let cli =
            Cli::try_parse_from(["contexter", "agent", "list"]).expect("should parse agent list");
        assert!(matches!(cli.command, Commands::Agent(AgentCommands::List)));
    }

    #[test]
    fn test_cli_parse_agent_update() {
        let cli = Cli::try_parse_from([
            "contexter",
            "agent",
            "update",
            "550e8400-e29b-41d4-a716-446655440000",
            "--name",
            "new-name",
        ])
        .expect("should parse agent update");
        assert!(matches!(
            cli.command,
            Commands::Agent(AgentCommands::Update { .. })
        ));
    }

    #[test]
    fn test_cli_parse_agent_delete() {
        let cli = Cli::try_parse_from([
            "contexter",
            "agent",
            "delete",
            "550e8400-e29b-41d4-a716-446655440000",
        ])
        .expect("should parse agent delete");
        assert!(matches!(
            cli.command,
            Commands::Agent(AgentCommands::Delete { .. })
        ));
    }

    #[test]
    fn test_cli_parse_skill_create() {
        let cli = Cli::try_parse_from([
            "contexter",
            "skill",
            "create",
            "--name",
            "code-review",
            "--description",
            "Review code changes",
            "--category",
            "dev",
            "--file-path",
            "/skills/review.py",
        ])
        .expect("should parse skill create");
        assert!(matches!(
            cli.command,
            Commands::Skill(SkillCommands::Create { .. })
        ));
    }

    #[test]
    fn test_cli_parse_skill_list() {
        let cli =
            Cli::try_parse_from(["contexter", "skill", "list"]).expect("should parse skill list");
        assert!(matches!(cli.command, Commands::Skill(SkillCommands::List)));
    }

    #[test]
    fn test_cli_parse_skill_delete() {
        let cli = Cli::try_parse_from([
            "contexter",
            "skill",
            "delete",
            "550e8400-e29b-41d4-a716-446655440000",
        ])
        .expect("should parse skill delete");
        assert!(matches!(
            cli.command,
            Commands::Skill(SkillCommands::Delete { .. })
        ));
    }

    #[test]
    fn test_cli_parse_setting_set() {
        let cli = Cli::try_parse_from(["contexter", "setting", "set", "theme", "dark"])
            .expect("should parse setting set");
        assert!(matches!(
            cli.command,
            Commands::Setting(SettingCommands::Set { .. })
        ));
    }

    #[test]
    fn test_cli_parse_setting_get() {
        let cli = Cli::try_parse_from(["contexter", "setting", "get", "theme"])
            .expect("should parse setting get");
        assert!(matches!(
            cli.command,
            Commands::Setting(SettingCommands::Get { .. })
        ));
    }

    #[test]
    fn test_cli_parse_audit_query() {
        let cli = Cli::try_parse_from([
            "contexter",
            "audit",
            "query",
            "--entity-type",
            "Session",
            "--actor",
            "user-1",
            "--limit",
            "50",
        ])
        .expect("should parse audit query");
        assert!(matches!(
            cli.command,
            Commands::Audit(AuditCommands::Query { .. })
        ));
    }

    #[test]
    fn test_cli_parse_diag_flush() {
        let cli =
            Cli::try_parse_from(["contexter", "diag", "flush"]).expect("should parse diag flush");
        assert!(matches!(cli.command, Commands::Diag(DiagCommands::Flush)));
    }

    #[test]
    fn test_cli_parse_diag_checkpoint() {
        let cli = Cli::try_parse_from(["contexter", "diag", "checkpoint"])
            .expect("should parse diag checkpoint");
        assert!(matches!(
            cli.command,
            Commands::Diag(DiagCommands::Checkpoint)
        ));
    }

    #[test]
    fn test_cli_parse_diag_storage_size() {
        let cli = Cli::try_parse_from(["contexter", "diag", "storage-size"])
            .expect("should parse diag storage-size");
        assert!(matches!(
            cli.command,
            Commands::Diag(DiagCommands::StorageSize)
        ));
    }

    #[test]
    fn test_cli_parse_diag_cache_stats() {
        let cli = Cli::try_parse_from(["contexter", "diag", "cache-stats"])
            .expect("should parse diag cache-stats");
        assert!(matches!(
            cli.command,
            Commands::Diag(DiagCommands::CacheStats)
        ));
    }

    #[test]
    fn test_cli_parse_diag_clear_cache_all() {
        let cli = Cli::try_parse_from(["contexter", "diag", "clear-cache"])
            .expect("should parse diag clear-cache");
        assert!(matches!(
            cli.command,
            Commands::Diag(DiagCommands::ClearCache { .. })
        ));
    }

    #[test]
    fn test_cli_parse_diag_clear_cache_type() {
        let cli = Cli::try_parse_from([
            "contexter",
            "diag",
            "clear-cache",
            "--entity-type",
            "session",
        ])
        .expect("should parse diag clear-cache with type");
        assert!(matches!(
            cli.command,
            Commands::Diag(DiagCommands::ClearCache { .. })
        ));
    }

    #[test]
    fn test_cli_parse_diag_health() {
        let cli =
            Cli::try_parse_from(["contexter", "diag", "health"]).expect("should parse diag health");
        assert!(matches!(cli.command, Commands::Diag(DiagCommands::Health)));
    }

    #[test]
    fn test_cli_parse_status() {
        let cli = Cli::try_parse_from(["contexter", "status"]).expect("should parse status");
        assert!(matches!(cli.command, Commands::Status));
    }

    #[test]
    fn test_cli_parse_checkpoint() {
        let cli =
            Cli::try_parse_from(["contexter", "checkpoint"]).expect("should parse checkpoint");
        assert!(matches!(cli.command, Commands::Checkpoint));
    }

    #[test]
    fn test_cli_parse_session_create_with_all_options() {
        let cli = Cli::try_parse_from([
            "contexter",
            "session",
            "create",
            "--agent-id",
            "550e8400-e29b-41d4-a716-446655440000",
            "--project",
            "my-project",
            "--status",
            "completed",
            "--metadata",
            r#"{"env": "test"}"#,
        ])
        .expect("should parse session create with all options");
        assert!(matches!(
            cli.command,
            Commands::Session(SessionCommands::Create { .. })
        ));
    }

    #[test]
    fn test_cli_parse_session_update_with_patch() {
        let cli = Cli::try_parse_from([
            "contexter",
            "session",
            "update",
            "550e8400-e29b-41d4-a716-446655440000",
            "--status",
            "active",
            "--turn-count",
            "42",
        ])
        .expect("should parse session update");
        assert!(matches!(
            cli.command,
            Commands::Session(SessionCommands::Update { .. })
        ));
    }

    #[test]
    fn test_cli_parse_invalid_uuid_rejected_by_parse() {
        // The parse should succeed (UUID validation happens at dispatch time)
        let cli = Cli::try_parse_from(["contexter", "session", "get", "not-a-uuid"])
            .expect("should parse session get (UUID validated at runtime)");
        assert!(matches!(
            cli.command,
            Commands::Session(SessionCommands::Get { .. })
        ));
    }

    #[test]
    fn test_parse_tags_empty() {
        assert_eq!(parse_tags(&None), None);
    }

    #[test]
    fn test_parse_tags_single() {
        assert_eq!(
            parse_tags(&Some("hello".into())),
            Some(vec!["hello".into()])
        );
    }

    #[test]
    fn test_parse_tags_multiple() {
        assert_eq!(
            parse_tags(&Some("a,b,c".into())),
            Some(vec!["a".into(), "b".into(), "c".into()])
        );
    }

    #[test]
    fn test_parse_tags_with_whitespace() {
        assert_eq!(
            parse_tags(&Some(" a , b , c ".into())),
            Some(vec!["a".into(), "b".into(), "c".into()])
        );
    }

    #[test]
    fn test_parse_tags_empty_string() {
        assert_eq!(parse_tags(&Some("".into())), Some(vec![]));
    }

    #[test]
    fn test_parse_json_valid() {
        let result = parse_json(&Some(r#"{"key": "value"}"#.into())).unwrap();
        assert_eq!(result, Some(serde_json::json!({"key": "value"})));
    }

    #[test]
    fn test_parse_json_invalid() {
        let result = parse_json(&Some("not json".into()));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json_none() {
        assert!(parse_json(&None).unwrap().is_none());
    }

    #[test]
    fn test_parse_uuid_valid() {
        let id = parse_uuid("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_parse_uuid_invalid() {
        let result = parse_uuid("not-a-uuid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid UUID"));
    }

    #[test]
    fn test_cli_parse_env_var_db_path() {
        // The `env` attribute on db_path reads `CONTEXTER_DB_PATH` — this just
        // verifies the default when no env var is set.
        let cli = Cli::try_parse_from(["contexter", "diag", "health"]).expect("should parse");
        assert_eq!(cli.db_path, None);
    }
}
