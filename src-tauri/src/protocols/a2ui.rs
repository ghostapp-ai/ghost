//! A2UI Protocol — Agent-to-User Interface for Ghost.
//!
//! Implements Google's A2UI v0.9 specification for declarative, JSON-based
//! generative UI. Agents describe *what* UI to show; the frontend renders
//! it using native React components.
//!
//! A2UI messages are transported via AG-UI CUSTOM events over Tauri IPC.
//!
//! ## Protocol Messages (Server → Client)
//! - `createSurface` — Initialize a new UI surface with a catalog and theme.
//! - `updateComponents` — Provide/update component definitions for a surface.
//! - `updateDataModel` — Insert or replace data that components bind to.
//! - `deleteSurface` — Remove a surface and all its contents.
//!
//! ## Standard Catalog Components
//! Text, Image, Icon, Video, AudioPlayer, Row, Column, List, Card, Tabs,
//! Divider, Modal, Button, CheckBox, TextField, DateTimeInput, ChoicePicker, Slider.
//!
//! Reference: <https://github.com/google/A2UI>
//! Spec version: v0.9

// A2UI types are the public API for agents to build generative UI.
// Not yet called from main app code — usage begins when agents produce surfaces.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::agui::{AgUiEvent, AgUiEventBus};

// ---------------------------------------------------------------------------
// A2UI Envelope Messages (Server → Client)
// ---------------------------------------------------------------------------

/// Top-level A2UI envelope — each message contains exactly one variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct A2uiMessage {
    /// Protocol version (e.g., "v0.9").
    pub version: String,
    /// Create a new surface.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_surface: Option<CreateSurface>,
    /// Update components on a surface.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_components: Option<UpdateComponents>,
    /// Update the data model of a surface.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_data_model: Option<UpdateDataModel>,
    /// Delete a surface.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_surface: Option<DeleteSurface>,
}

impl A2uiMessage {
    /// Create a new `createSurface` message.
    pub fn create_surface(surface_id: &str, catalog_id: &str) -> Self {
        Self {
            version: "v0.9".to_string(),
            create_surface: Some(CreateSurface {
                surface_id: surface_id.to_string(),
                catalog_id: catalog_id.to_string(),
                theme: None,
                send_data_model: None,
            }),
            update_components: None,
            update_data_model: None,
            delete_surface: None,
        }
    }

    /// Create an `updateComponents` message.
    pub fn update_components(surface_id: &str, components: Vec<A2uiComponent>) -> Self {
        Self {
            version: "v0.9".to_string(),
            create_surface: None,
            update_components: Some(UpdateComponents {
                surface_id: surface_id.to_string(),
                components,
            }),
            update_data_model: None,
            delete_surface: None,
        }
    }

    /// Create an `updateDataModel` message.
    pub fn update_data_model(
        surface_id: &str,
        path: Option<&str>,
        value: serde_json::Value,
    ) -> Self {
        Self {
            version: "v0.9".to_string(),
            create_surface: None,
            update_components: None,
            update_data_model: Some(UpdateDataModel {
                surface_id: surface_id.to_string(),
                path: path.map(|s| s.to_string()),
                value: Some(value),
            }),
            delete_surface: None,
        }
    }

    /// Create a `deleteSurface` message.
    pub fn delete_surface(surface_id: &str) -> Self {
        Self {
            version: "v0.9".to_string(),
            create_surface: None,
            update_components: None,
            update_data_model: None,
            delete_surface: Some(DeleteSurface {
                surface_id: surface_id.to_string(),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Message Payloads
// ---------------------------------------------------------------------------

/// Initialize a new UI surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSurface {
    /// Unique identifier for this surface.
    pub surface_id: String,
    /// Catalog identifier (URI or standard catalog ref).
    pub catalog_id: String,
    /// Optional theme properties (e.g., primaryColor).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<A2uiTheme>,
    /// If true, client sends data model state with every action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_data_model: Option<bool>,
}

/// Update or add components to a surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponents {
    /// Target surface ID.
    pub surface_id: String,
    /// Flat list of component definitions (adjacency list model).
    pub components: Vec<A2uiComponent>,
}

/// Update the data model for a surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDataModel {
    /// Target surface ID.
    pub surface_id: String,
    /// JSON Pointer to the location to update. Defaults to "/" (root).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// The new value. If omitted, the key at `path` is removed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

/// Remove a surface from the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSurface {
    /// Surface ID to delete.
    pub surface_id: String,
}

// ---------------------------------------------------------------------------
// Component Model
// ---------------------------------------------------------------------------

/// A single A2UI component definition.
///
/// Components are identified by `id` and reference children by ID.
/// This creates an adjacency list model — the client reconstructs the tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct A2uiComponent {
    /// Unique component ID within the surface.
    pub id: String,
    /// Component type (e.g., "Text", "Button", "Row", "Column", "Card").
    pub component: String,
    /// Single child reference (for Card, Modal, Button).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child: Option<String>,
    /// List of child component IDs (for Row, Column, List, Tabs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<ChildList>,
    /// Text content (for Text, Button label).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<DynamicString>,
    /// Label (for TextField, CheckBox, Slider).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<DynamicString>,
    /// Variant (e.g., "h1", "h2", "body", "caption" for Text; "primary", "borderless" for Button).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    /// Value binding (for input components).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<DynamicValue>,
    /// Action triggered on interaction (for Button).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<A2uiAction>,
    /// Image/Icon URL or name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<DynamicString>,
    /// Icon name (for Icon component).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Layout alignment (for Row, Column).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<String>,
    /// Layout justification (for Row, Column).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify: Option<String>,
    /// Weight (flex grow factor).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    /// Orientation axis (for Divider: "horizontal" or "vertical").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axis: Option<String>,
    /// Options (for ChoicePicker).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ChoiceOption>>,
    /// Min value (for Slider).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Max value (for Slider).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Step value (for Slider).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    /// Validation checks (for input components).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<Vec<serde_json::Value>>,
    /// Additional properties not covered above.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Child list — either a static array of IDs or a template for data-bound lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChildList {
    /// Static list of child component IDs.
    Array(Vec<String>),
    /// Template for generating children from data.
    Template {
        /// JSON Pointer path to the data array.
        path: String,
        /// Template component ID to instantiate per item.
        #[serde(rename = "componentId")]
        component_id: String,
    },
}

/// Dynamic string value — either a literal or a data binding path.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DynamicString {
    /// Literal string value.
    Literal(String),
    /// Data binding via JSON Pointer path.
    Path { path: String },
    /// Function call for computed values.
    FunctionCall {
        call: String,
        args: serde_json::Value,
    },
}

/// Dynamic value — for input component bindings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DynamicValue {
    /// Literal string.
    LiteralString(String),
    /// Literal number.
    LiteralNumber(f64),
    /// Literal boolean.
    LiteralBool(bool),
    /// Data binding path.
    Path { path: String },
}

/// A choice option for ChoicePicker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceOption {
    pub label: String,
    pub value: String,
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

/// Action definition for interactive components.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct A2uiAction {
    /// Server-side event action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<A2uiEventAction>,
    /// Client-side function call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<serde_json::Value>,
}

/// Event action — sent to the server when triggered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2uiEventAction {
    /// Event name.
    pub name: String,
    /// Context data sent with the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

/// A2UI theme properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct A2uiTheme {
    /// Primary brand color (hex, e.g., "#00BFFF").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_color: Option<String>,
    /// Icon URL for the agent/tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Display name for the agent/tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_display_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Client → Server Messages
// ---------------------------------------------------------------------------

/// Action event from the client (user interaction).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct A2uiClientAction {
    /// Action name.
    pub name: String,
    /// Surface where the action originated.
    pub surface_id: String,
    /// Component that triggered the action.
    pub source_component_id: String,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Context data from the component's action definition.
    pub context: serde_json::Value,
}

// ---------------------------------------------------------------------------
// A2UI ↔ AG-UI Bridge
// ---------------------------------------------------------------------------

/// The A2UI event name used in AG-UI CUSTOM events.
pub const A2UI_EVENT_NAME: &str = "a2ui";

/// Emit an A2UI message through the AG-UI event bus as a CUSTOM event.
///
/// A2UI messages are transported over AG-UI's CUSTOM event type,
/// making them compatible with the existing Tauri IPC pipeline.
pub fn emit_a2ui(run_id: &str, message: &A2uiMessage, event_bus: &AgUiEventBus) {
    let value = serde_json::to_value(message).unwrap_or(serde_json::Value::Null);
    event_bus.emit(AgUiEvent::custom(run_id, A2UI_EVENT_NAME, value));
}

/// Convenience: emit a createSurface + updateComponents + updateDataModel sequence.
pub fn emit_surface(
    run_id: &str,
    surface_id: &str,
    components: Vec<A2uiComponent>,
    data: Option<serde_json::Value>,
    theme: Option<A2uiTheme>,
    event_bus: &AgUiEventBus,
) {
    // 1. Create surface
    let mut create_msg = A2uiMessage::create_surface(surface_id, GHOST_CATALOG_ID);
    if let Some(ref mut cs) = create_msg.create_surface {
        cs.theme = theme;
    }
    emit_a2ui(run_id, &create_msg, event_bus);

    // 2. Update components
    let components_msg = A2uiMessage::update_components(surface_id, components);
    emit_a2ui(run_id, &components_msg, event_bus);

    // 3. Update data model (if provided)
    if let Some(data_value) = data {
        let data_msg = A2uiMessage::update_data_model(surface_id, None, data_value);
        emit_a2ui(run_id, &data_msg, event_bus);
    }
}

// ---------------------------------------------------------------------------
// Ghost Standard Catalog ID
// ---------------------------------------------------------------------------

/// Ghost's A2UI catalog identifier — standard catalog plus Ghost extensions.
pub const GHOST_CATALOG_ID: &str = "https://ghostapp.ai/a2ui/standard/v0.9";

// ---------------------------------------------------------------------------
// Component Builders (convenience constructors)
// ---------------------------------------------------------------------------

/// Builder for commonly used A2UI components.
pub struct Components;

impl Components {
    /// Create a Text component.
    pub fn text(id: &str, text: &str) -> A2uiComponent {
        A2uiComponent {
            id: id.to_string(),
            component: "Text".to_string(),
            text: Some(DynamicString::Literal(text.to_string())),
            ..Default::default()
        }
    }

    /// Create a Text component with a variant (h1, h2, body, caption).
    pub fn text_variant(id: &str, text: &str, variant: &str) -> A2uiComponent {
        let mut c = Self::text(id, text);
        c.variant = Some(variant.to_string());
        c
    }

    /// Create a Text component bound to a data path.
    pub fn text_bound(id: &str, path: &str) -> A2uiComponent {
        A2uiComponent {
            id: id.to_string(),
            component: "Text".to_string(),
            text: Some(DynamicString::Path {
                path: path.to_string(),
            }),
            ..Default::default()
        }
    }

    /// Create a Button component.
    pub fn button(id: &str, label_id: &str, action_name: &str) -> A2uiComponent {
        A2uiComponent {
            id: id.to_string(),
            component: "Button".to_string(),
            child: Some(label_id.to_string()),
            variant: Some("primary".to_string()),
            action: Some(A2uiAction {
                event: Some(A2uiEventAction {
                    name: action_name.to_string(),
                    context: None,
                }),
                function_call: None,
            }),
            ..Default::default()
        }
    }

    /// Create a Row layout container.
    pub fn row(id: &str, children: Vec<&str>) -> A2uiComponent {
        A2uiComponent {
            id: id.to_string(),
            component: "Row".to_string(),
            children: Some(ChildList::Array(
                children.iter().map(|s| s.to_string()).collect(),
            )),
            ..Default::default()
        }
    }

    /// Create a Column layout container.
    pub fn column(id: &str, children: Vec<&str>) -> A2uiComponent {
        A2uiComponent {
            id: id.to_string(),
            component: "Column".to_string(),
            children: Some(ChildList::Array(
                children.iter().map(|s| s.to_string()).collect(),
            )),
            ..Default::default()
        }
    }

    /// Create a Card container.
    pub fn card(id: &str, child_id: &str) -> A2uiComponent {
        A2uiComponent {
            id: id.to_string(),
            component: "Card".to_string(),
            child: Some(child_id.to_string()),
            ..Default::default()
        }
    }

    /// Create a TextField input.
    pub fn text_field(id: &str, label: &str, value_path: &str) -> A2uiComponent {
        A2uiComponent {
            id: id.to_string(),
            component: "TextField".to_string(),
            label: Some(DynamicString::Literal(label.to_string())),
            value: Some(DynamicValue::Path {
                path: value_path.to_string(),
            }),
            variant: Some("shortText".to_string()),
            ..Default::default()
        }
    }

    /// Create a CheckBox input.
    pub fn checkbox(id: &str, label: &str, value_path: &str) -> A2uiComponent {
        A2uiComponent {
            id: id.to_string(),
            component: "CheckBox".to_string(),
            label: Some(DynamicString::Literal(label.to_string())),
            value: Some(DynamicValue::Path {
                path: value_path.to_string(),
            }),
            ..Default::default()
        }
    }

    /// Create a Divider.
    pub fn divider(id: &str) -> A2uiComponent {
        A2uiComponent {
            id: id.to_string(),
            component: "Divider".to_string(),
            axis: Some("horizontal".to_string()),
            ..Default::default()
        }
    }

    /// Create an Icon component.
    pub fn icon(id: &str, name: &str) -> A2uiComponent {
        A2uiComponent {
            id: id.to_string(),
            component: "Icon".to_string(),
            name: Some(name.to_string()),
            ..Default::default()
        }
    }

    /// Create a ChoicePicker (single/multi select).
    pub fn choice_picker(
        id: &str,
        options: Vec<(&str, &str)>,
        value_path: &str,
        variant: &str,
    ) -> A2uiComponent {
        A2uiComponent {
            id: id.to_string(),
            component: "ChoicePicker".to_string(),
            options: Some(
                options
                    .into_iter()
                    .map(|(l, v)| ChoiceOption {
                        label: l.to_string(),
                        value: v.to_string(),
                    })
                    .collect(),
            ),
            value: Some(DynamicValue::Path {
                path: value_path.to_string(),
            }),
            variant: Some(variant.to_string()),
            ..Default::default()
        }
    }

    /// Create a Slider input.
    pub fn slider(id: &str, min: f64, max: f64, step: f64, value_path: &str) -> A2uiComponent {
        A2uiComponent {
            id: id.to_string(),
            component: "Slider".to_string(),
            min: Some(min),
            max: Some(max),
            step: Some(step),
            value: Some(DynamicValue::Path {
                path: value_path.to_string(),
            }),
            ..Default::default()
        }
    }
}

/// Default implementation for A2uiComponent (all optional fields are None).
impl Default for A2uiComponent {
    fn default() -> Self {
        Self {
            id: String::new(),
            component: String::new(),
            child: None,
            children: None,
            text: None,
            label: None,
            variant: None,
            value: None,
            action: None,
            url: None,
            name: None,
            align: None,
            justify: None,
            weight: None,
            axis: None,
            options: None,
            min: None,
            max: None,
            step: None,
            checks: None,
            extra: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_surface_message() {
        let msg = A2uiMessage::create_surface("test_surface", GHOST_CATALOG_ID);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"version\":\"v0.9\""));
        assert!(json.contains("\"surfaceId\":\"test_surface\""));
        assert!(json.contains("\"catalogId\""));
        // No other message types should be present
        assert!(!json.contains("\"updateComponents\""));
        assert!(!json.contains("\"updateDataModel\""));
        assert!(!json.contains("\"deleteSurface\""));
    }

    #[test]
    fn test_update_components_message() {
        let components = vec![
            Components::column("root", vec!["title", "body"]),
            Components::text_variant("title", "Hello Ghost", "h2"),
            Components::text("body", "Welcome to A2UI."),
        ];
        let msg = A2uiMessage::update_components("test_surface", components);
        let json = serde_json::to_string_pretty(&msg).unwrap();
        // pretty-print has ": " (space after colon)
        assert!(
            json.contains("\"surfaceId\": \"test_surface\""),
            "surfaceId not found in: {json}"
        );
        assert!(json.contains("\"component\": \"Column\""));
        assert!(json.contains("\"component\": \"Text\""));
        assert!(json.contains("\"Hello Ghost\""));
    }

    #[test]
    fn test_update_data_model_message() {
        let msg = A2uiMessage::update_data_model(
            "test_surface",
            Some("/user/name"),
            serde_json::json!("Alice"),
        );
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"path\":\"/user/name\""));
        assert!(json.contains("\"Alice\""));
    }

    #[test]
    fn test_delete_surface_message() {
        let msg = A2uiMessage::delete_surface("test_surface");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"deleteSurface\""));
        assert!(json.contains("\"surfaceId\":\"test_surface\""));
    }

    #[test]
    fn test_component_builders() {
        let text = Components::text("t1", "Hello");
        assert_eq!(text.component, "Text");
        assert!(matches!(text.text, Some(DynamicString::Literal(ref s)) if s == "Hello"));

        let row = Components::row("r1", vec!["a", "b"]);
        assert_eq!(row.component, "Row");
        assert!(matches!(row.children, Some(ChildList::Array(ref v)) if v.len() == 2));

        let btn = Components::button("b1", "label1", "submit");
        assert_eq!(btn.component, "Button");
        assert!(btn.action.is_some());
    }

    #[test]
    fn test_dynamic_string_serialization() {
        let literal = DynamicString::Literal("hello".to_string());
        assert_eq!(serde_json::to_string(&literal).unwrap(), "\"hello\"");

        let path = DynamicString::Path {
            path: "/user/name".to_string(),
        };
        let json = serde_json::to_string(&path).unwrap();
        assert!(json.contains("\"path\":\"/user/name\""));
    }

    #[test]
    fn test_child_list_variants() {
        let array = ChildList::Array(vec!["a".to_string(), "b".to_string()]);
        let json = serde_json::to_string(&array).unwrap();
        assert_eq!(json, "[\"a\",\"b\"]");

        let template = ChildList::Template {
            path: "/items".to_string(),
            component_id: "item_template".to_string(),
        };
        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("\"path\":\"/items\""));
        assert!(json.contains("\"componentId\":\"item_template\""));
    }

    #[test]
    fn test_full_surface_roundtrip() {
        // Simulate a full surface creation sequence
        let create = A2uiMessage::create_surface("profile", GHOST_CATALOG_ID);
        let components = A2uiMessage::update_components(
            "profile",
            vec![
                Components::card("root", "content"),
                Components::column(
                    "content",
                    vec!["name", "email_field", "submit_btn", "submit_label"],
                ),
                Components::text_bound("name", "/user/name"),
                Components::text_field("email_field", "Email", "/user/email"),
                Components::button("submit_btn", "submit_label", "save_profile"),
                Components::text("submit_label", "Save"),
            ],
        );
        let data = A2uiMessage::update_data_model(
            "profile",
            None,
            serde_json::json!({
                "user": {
                    "name": "Ghost User",
                    "email": "ghost@local"
                }
            }),
        );

        // Verify all serialize correctly
        for msg in [&create, &components, &data] {
            let json = serde_json::to_string(msg).unwrap();
            assert!(json.contains("\"v0.9\""));
            let _roundtrip: A2uiMessage = serde_json::from_str(&json).unwrap();
        }
    }
}
