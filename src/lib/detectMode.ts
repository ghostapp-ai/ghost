/**
 * Smart mode detection for the Ghost Omnibox.
 *
 * Determines whether a query should trigger file search or chat.
 * Returns "search" for file/keyword queries, "chat" for conversational input.
 */

/** File-like patterns that always indicate search mode. */
const FILE_PATTERNS =
  /\.(pdf|docx?|xlsx?|txt|md|tsx?|jsx?|json|ya?ml|toml|csv|py|rs|go|html?|css|svg|png|jpe?g|gif)\b/i;

/** Path-like patterns (contains slashes or backslashes). */
const PATH_PATTERN = /[/\\][\w.-]+/;

/** Conversational starters (Spanish + English). */
const CHAT_STARTERS =
  /^(qué|que |cómo|como |por qué|porque |dónde|donde |cuándo|cuando |cuál|cual |quién|quien |explica|describe|resume|traduce|ayuda|genera|escribe|analiza|compara|define|lista|enumera|what |how |why |where |when |which |who |explain |tell |show |can you |could you |please |help |write |create |generate |analyze |compare |define |list |summarize |translate |hola|hi |hello|hey )/i;

/** Explicit prefix triggers. */
const SEARCH_PREFIX = /^[/!>]/;
const CHAT_PREFIX = /^[?@]/;

export type InputMode = "search" | "chat";

/**
 * Detect whether input should trigger search or chat.
 *
 * @param query - The current input text
 * @param hasActiveChat - Whether there are existing chat messages (sticky mode)
 */
export function detectMode(query: string, hasActiveChat: boolean): InputMode {
  const trimmed = query.trim();

  // Empty → keep context (chat if has messages, search otherwise)
  if (!trimmed) return hasActiveChat ? "chat" : "search";

  // Explicit prefixes override everything
  if (SEARCH_PREFIX.test(trimmed)) return "search";
  if (CHAT_PREFIX.test(trimmed)) return "chat";

  // File patterns always mean search
  if (FILE_PATTERNS.test(trimmed)) return "search";
  if (PATH_PATTERN.test(trimmed)) return "search";

  // Conversational starters → chat
  if (CHAT_STARTERS.test(trimmed)) return "chat";

  // Questions (ends with ?) → chat
  if (trimmed.endsWith("?")) return "chat";

  // Long natural language (> 5 words) → likely chat
  if (trimmed.split(/\s+/).length > 5) return "chat";

  // If there's an active chat and the query is medium-length, stay in chat (sticky)
  if (hasActiveChat && trimmed.split(/\s+/).length > 2) return "chat";

  // Default: search for short queries
  return "search";
}
