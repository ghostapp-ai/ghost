//! Conversation memory — persistent storage for agent conversations.
//!
//! Stores conversations and messages in SQLite for context recall.
//! Supports:
//! - Multiple conversations with metadata
//! - Full message history with roles
//! - FTS5 search across past conversations
//! - Automatic conversation summarization (future)

use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::error::Result;

/// A conversation with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: i64,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: i64,
    /// Summary of the conversation (populated after several messages).
    pub summary: Option<String>,
}

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub conversation_id: i64,
    pub role: String,
    pub content: String,
    pub created_at: String,
    /// Tool calls made in this message (JSON array).
    pub tool_calls: Option<String>,
    /// Tool result (for "tool" role messages).
    pub tool_result: Option<String>,
    /// Model used to generate this message.
    pub model: Option<String>,
}

/// Initialize conversation tables in the database.
pub fn initialize_memory_schema(db: &Database) -> Result<()> {
    db.with_conn(|conn| {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL DEFAULT 'New Conversation',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                summary TEXT
            );

            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL
                    REFERENCES conversations(id) ON DELETE CASCADE,
                role TEXT NOT NULL,
                content TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                tool_calls TEXT,
                tool_result TEXT,
                model TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_messages_conversation
                ON messages(conversation_id, created_at);

            -- FTS5 for searching across conversations
            CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
                content,
                content=messages,
                content_rowid=id,
                tokenize='porter unicode61'
            );

            -- Sync triggers for FTS5
            CREATE TRIGGER IF NOT EXISTS messages_ai AFTER INSERT ON messages BEGIN
                INSERT INTO messages_fts(rowid, content) VALUES (new.id, new.content);
            END;
            CREATE TRIGGER IF NOT EXISTS messages_ad AFTER DELETE ON messages BEGIN
                INSERT INTO messages_fts(messages_fts, rowid, content)
                    VALUES('delete', old.id, old.content);
            END;
            CREATE TRIGGER IF NOT EXISTS messages_au AFTER UPDATE ON messages BEGIN
                INSERT INTO messages_fts(messages_fts, rowid, content)
                    VALUES('delete', old.id, old.content);
                INSERT INTO messages_fts(rowid, content) VALUES (new.id, new.content);
            END;
            ",
        )?;
        Ok(())
    })
}

/// Create a new conversation. Returns the conversation ID.
pub fn create_conversation(db: &Database, title: &str) -> Result<i64> {
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO conversations (title) VALUES (?1)",
            rusqlite::params![title],
        )?;
        Ok(conn.last_insert_rowid())
    })
}

/// Add a message to a conversation. Returns the message ID.
pub fn add_message(
    db: &Database,
    conversation_id: i64,
    role: &str,
    content: &str,
    tool_calls: Option<&str>,
    tool_result: Option<&str>,
    model: Option<&str>,
) -> Result<i64> {
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO messages (conversation_id, role, content, tool_calls, tool_result, model)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                conversation_id,
                role,
                content,
                tool_calls,
                tool_result,
                model
            ],
        )?;
        let msg_id = conn.last_insert_rowid();

        // Update conversation timestamp
        conn.execute(
            "UPDATE conversations SET updated_at = datetime('now') WHERE id = ?1",
            rusqlite::params![conversation_id],
        )?;

        Ok(msg_id)
    })
}

/// Get messages for a conversation, ordered chronologically.
pub fn get_messages(
    db: &Database,
    conversation_id: i64,
    limit: Option<usize>,
) -> Result<Vec<Message>> {
    db.with_conn(|conn| {
        let limit_clause = match limit {
            Some(n) => format!("LIMIT {}", n),
            None => String::new(),
        };
        let sql = format!(
            "SELECT id, conversation_id, role, content, created_at, tool_calls, tool_result, model
             FROM messages WHERE conversation_id = ?1
             ORDER BY created_at ASC {}",
            limit_clause
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params![conversation_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                tool_calls: row.get(5)?,
                tool_result: row.get(6)?,
                model: row.get(7)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    })
}

/// List all conversations, most recent first.
pub fn list_conversations(db: &Database, limit: usize) -> Result<Vec<Conversation>> {
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT c.id, c.title, c.created_at, c.updated_at, c.summary,
                    (SELECT COUNT(*) FROM messages m WHERE m.conversation_id = c.id)
             FROM conversations c
             ORDER BY c.updated_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit as i64], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                summary: row.get(4)?,
                message_count: row.get(5)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    })
}

/// Delete a conversation and all its messages.
pub fn delete_conversation(db: &Database, conversation_id: i64) -> Result<()> {
    db.with_conn(|conn| {
        conn.execute(
            "DELETE FROM conversations WHERE id = ?1",
            rusqlite::params![conversation_id],
        )?;
        Ok(())
    })
}

/// Update conversation title.
pub fn update_conversation_title(db: &Database, conversation_id: i64, title: &str) -> Result<()> {
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE conversations SET title = ?1 WHERE id = ?2",
            rusqlite::params![title, conversation_id],
        )?;
        Ok(())
    })
}

/// Search across all conversations using FTS5.
pub fn search_conversations(db: &Database, query: &str, limit: usize) -> Result<Vec<Message>> {
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT m.id, m.conversation_id, m.role, m.content, m.created_at,
                    m.tool_calls, m.tool_result, m.model
             FROM messages_fts fts
             JOIN messages m ON m.id = fts.rowid
             WHERE messages_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![query, limit as i64], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                tool_calls: row.get(5)?,
                tool_result: row.get(6)?,
                model: row.get(7)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    })
}

/// Get the most recent messages across all conversations for context.
/// Useful for building conversation summaries.
#[allow(dead_code)]
pub fn get_recent_context(db: &Database, limit: usize) -> Result<Vec<Message>> {
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, role, content, created_at, tool_calls, tool_result, model
             FROM messages
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit as i64], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                tool_calls: row.get(5)?,
                tool_result: row.get(6)?,
                model: row.get(7)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Database {
        let db = Database::open_in_memory().unwrap();
        initialize_memory_schema(&db).unwrap();
        db
    }

    #[test]
    fn test_create_conversation() {
        let db = setup_test_db();
        let id = create_conversation(&db, "Test Conversation").unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_add_and_get_messages() {
        let db = setup_test_db();
        let conv_id = create_conversation(&db, "Test").unwrap();

        add_message(&db, conv_id, "user", "Hello!", None, None, None).unwrap();
        add_message(
            &db,
            conv_id,
            "assistant",
            "Hi there!",
            None,
            None,
            Some("qwen3:8b"),
        )
        .unwrap();

        let messages = get_messages(&db, conv_id, None).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Hello!");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[1].model, Some("qwen3:8b".into()));
    }

    #[test]
    fn test_list_conversations() {
        let db = setup_test_db();
        create_conversation(&db, "First").unwrap();
        create_conversation(&db, "Second").unwrap();

        let convs = list_conversations(&db, 10).unwrap();
        assert_eq!(convs.len(), 2);
    }

    #[test]
    fn test_delete_conversation() {
        let db = setup_test_db();
        let id = create_conversation(&db, "To Delete").unwrap();
        add_message(&db, id, "user", "test", None, None, None).unwrap();

        delete_conversation(&db, id).unwrap();
        let convs = list_conversations(&db, 10).unwrap();
        assert_eq!(convs.len(), 0);
    }

    #[test]
    fn test_search_conversations() {
        let db = setup_test_db();
        let id = create_conversation(&db, "Search Test").unwrap();
        add_message(
            &db,
            id,
            "user",
            "Where is the quantum physics paper?",
            None,
            None,
            None,
        )
        .unwrap();
        add_message(
            &db,
            id,
            "assistant",
            "I found it in your documents folder.",
            None,
            None,
            None,
        )
        .unwrap();

        let results = search_conversations(&db, "quantum physics", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].content.contains("quantum"));
    }

    #[test]
    fn test_tool_call_message() {
        let db = setup_test_db();
        let id = create_conversation(&db, "Tool Test").unwrap();

        let tool_calls = r#"[{"function":{"name":"ghost_search","arguments":{"query":"test"}}}]"#;
        add_message(
            &db,
            id,
            "assistant",
            "",
            Some(tool_calls),
            None,
            Some("qwen3:8b"),
        )
        .unwrap();
        add_message(
            &db,
            id,
            "tool",
            "Found 3 results",
            None,
            Some("ghost_search result"),
            None,
        )
        .unwrap();

        let messages = get_messages(&db, id, None).unwrap();
        assert_eq!(messages.len(), 2);
        assert!(messages[0].tool_calls.is_some());
        assert!(messages[1].tool_result.is_some());
    }
}
