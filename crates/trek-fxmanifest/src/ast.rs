use std::fmt;

/// A parsed declaration in the manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    /// `key 'value'`, `key('value')` or `key = 'value'`
    Scalar { key: Key, value: String },
    /// `key { 'a', 'b' }`, `key = { ... }` or `key('a', 'b')`
    Table { key: Key, values: Vec<String> },
}

impl Statement {
    pub fn key(&self) -> &Key {
        match self {
            Statement::Scalar { key, .. } | Statement::Table { key, .. } => key,
        }
    }
}

/// Well-known manifest keys; unknown keys fall back to [`Key::Other`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    FxVersion,
    Game,
    Author,
    Description,
    Version,
    ClientScripts,
    ServerScripts,
    SharedScripts,
    Files,
    Dependency,
    Dependencies,
    Lua54,
    Other(String),
}

impl Key {
    pub fn parse(s: &str) -> Self {
        match s {
            "fx_version" => Key::FxVersion,
            "game" => Key::Game,
            "author" => Key::Author,
            "description" => Key::Description,
            "version" => Key::Version,
            "client_scripts" | "client_script" => Key::ClientScripts,
            "server_scripts" | "server_script" => Key::ServerScripts,
            "shared_scripts" | "shared_script" => Key::SharedScripts,
            "files" => Key::Files,
            "dependency" => Key::Dependency,
            "dependencies" => Key::Dependencies,
            "lua54" => Key::Lua54,
            other => Key::Other(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Key::FxVersion => "fx_version",
            Key::Game => "game",
            Key::Author => "author",
            Key::Description => "description",
            Key::Version => "version",
            Key::ClientScripts => "client_scripts",
            Key::ServerScripts => "server_scripts",
            Key::SharedScripts => "shared_scripts",
            Key::Files => "files",
            Key::Dependency => "dependency",
            Key::Dependencies => "dependencies",
            Key::Lua54 => "lua54",
            Key::Other(s) => s,
        }
    }
}

impl From<&str> for Key {
    fn from(value: &str) -> Self {
        Key::parse(value)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Supported FiveM game targets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Game {
    Gta5,
    Rdr3,
    Other(String),
}

impl Game {
    fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "gta5" => Game::Gta5,
            "rdr3" => Game::Rdr3,
            _ => Game::Other(s.trim().to_string()),
        }
    }
}

/// Stage 4 of the pipeline: the strongly-typed manifest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Manifest {
    pub fx_version: Option<String>,
    pub game: Option<Game>,
    pub statements: Vec<Statement>,
}

impl Manifest {
    pub(crate) fn from_statements(statements: Vec<Statement>) -> Self {
        let mut manifest = Self {
            statements,
            ..Self::default()
        };

        for stmt in &manifest.statements {
            if let Statement::Scalar { key, value } = stmt {
                match key {
                    Key::FxVersion => manifest.fx_version = Some(value.clone()),
                    Key::Game => manifest.game = Some(Game::parse(value)),
                    _ => {}
                }
            }
        }

        manifest
    }

    /// Returns the first scalar value declared under `key`.
    pub fn get(&self, key: &Key) -> Option<&str> {
        self.statements.iter().find_map(|stmt| match stmt {
            Statement::Scalar { value, .. } if stmt.key() == key => Some(value.as_str()),
            _ => None,
        })
    }

    /// Returns every value declared under `key`, flattening tables.
    pub fn values(&self, key: &Key) -> Vec<&str> {
        self.statements
            .iter()
            .filter(|stmt| stmt.key() == key)
            .flat_map(|stmt| match stmt {
                Statement::Scalar { value, .. } => vec![value.as_str()],
                Statement::Table { values, .. } => values.iter().map(String::as_str).collect(),
            })
            .collect()
    }

    pub fn version(&self) -> Option<&str> {
        self.get(&Key::Version)
    }

    pub fn description(&self) -> Option<&str> {
        self.get(&Key::Description)
    }

    pub fn client_scripts(&self) -> Vec<&str> {
        self.values(&Key::ClientScripts)
    }

    pub fn server_scripts(&self) -> Vec<&str> {
        self.values(&Key::ServerScripts)
    }

    pub fn shared_scripts(&self) -> Vec<&str> {
        self.values(&Key::SharedScripts)
    }
}
