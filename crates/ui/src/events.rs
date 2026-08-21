use ellastic_errors::{Result, EllasticError};
use ellastic_core::{MediaData, MediaType};
use ellastic_image::{ImageProcessor, ImageData};
use ellastic_audio::{AudioProcessor, AudioData};
use ellastic_media::{MediaProcessor};
use ellastic_glitch::{GlitchProcessor, GlitchEffect};
use ellastic_effects::{EffectProcessor, EffectType};
use ellastic_pipeline::{PipelineProcessor, PipelineGraph};
use ellastic_project::{ProjectManager, Project};
use ellastic_export::{ExportManager, ExportRequest};
use ellastic_utils::{create_random_generator};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct EventBus {
    pub events: Arc<RwLock<Vec<Event>>>,
    pub subscribers: Arc<RwLock<HashMap<String, Vec<EventSubscriber>>>,
    pub handlers: Arc<RwLock<HashMap<String, EventHandler>>>,
    pub filters: Arc<RwLock<Vec<EventFilter>>>,
    pub queue: Arc<RwLock<EventQueue>>,
    pub config: EventBusConfig,
}

#[derive(Debug, Clone)]
pub struct EventBusConfig {
    pub max_events: usize,
    pub max_queue_size: usize,
    pub enable_history: bool,
    pub enable_filters: bool,
    pub enable_async: bool,
    pub default_priority: EventPriority,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub source: String,
    pub target: Option<String>,
    pub data: EventData,
    pub metadata: EventMetadata,
    pub timestamp: DateTime<Utc>,
    pub priority: EventPriority,
}

#[derive(Debug, Clone)]
pub enum EventData {
    Empty,
    String(String),
    Number(f64),
    Boolean(bool),
    Object(HashMap<String, serde_json::Value>),
    Array(Vec<serde_json::Value>),
    Binary(Vec<u8>),
    Custom(serde_json::Value),
}

#[derive(Debug, Clone)]
pub struct EventMetadata {
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Lowest = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Highest = 4,
}

#[derive(Debug, Clone)]
pub struct EventSubscriber {
    pub id: Uuid,
    pub name: String,
    pub event_types: Vec<String>,
    pub sources: Vec<String>,
    pub filter: Option<EventFilter>,
    pub handler: EventHandlerFn,
    pub priority: EventPriority,
    pub async: bool,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

pub type EventHandlerFn = Box<dyn Fn(&Event) -> Result<EventResult> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct EventHandler {
    pub id: String,
    pub name: String,
    pub description: String,
    pub event_type: String,
    pub handler: EventHandlerFn,
    pub priority: EventPriority,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct EventFilter {
    pub id: String,
    pub name: String,
    pub filter_type: FilterType,
    pub conditions: Vec<FilterCondition>,
    pub logic: FilterLogic,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    EventType,
    Source,
    Target,
    Data,
    Metadata,
    Custom,
}

#[derive(Debug, Clone)]
pub struct FilterCondition {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    In,
    NotIn,
    Regex,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterLogic {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone)]
pub struct EventQueue {
    pub events: Vec<QueuedEvent>,
    pub processing: bool,
    pub max_size: usize,
}

#[derive(Debug, Clone)]
pub struct QueuedEvent {
    pub event: Event,
    pub retry_count: u32,
    pub max_retries: u32,
    pub delay_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct EventResult {
    pub success: bool,
    pub data: Option<EventData>,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct EventHistory {
    pub events: Vec<Event>,
    pub max_size: usize,
    pub retention_days: u32,
}

#[derive(Debug, Clone)]
pub struct EventMetrics {
    pub total_events: u64,
    pub events_by_type: HashMap<String, u64>,
    pub events_by_source: HashMap<String, u64>,
    pub events_by_priority: HashMap<EventPriority, u64>,
    pub processing_time: f64,
    pub error_rate: f64,
    pub throughput: f64,
}

#[derive(Debug, Clone)]
pub struct EventBatch {
    pub id: Uuid,
    pub events: Vec<Event>,
    pub batch_type: BatchType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchType {
    Transactional,
    NonTransactional,
    Ordered,
    Unordered,
}

#[derive(Debug, Clone)]
pub struct EventStream {
    pub id: Uuid,
    pub name: String,
    pub event_types: Vec<String>,
    pub buffer: Vec<Event>,
    pub max_buffer_size: usize,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct EventProjection {
    pub id: Uuid,
    pub name: String,
    pub event_types: Vec<String>,
    pub projection_fn: Box<dyn Fn(&Event) -> Result<ProjectionResult> + Send + Sync>,
    pub state: ProjectionState,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum ProjectionState {
    Empty,
    Data(serde_json::Value),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ProjectionResult {
    pub success: bool,
    pub state: ProjectionState,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EventSaga {
    pub id: Uuid,
    pub name: String,
    pub steps: Vec<SagaStep>,
    pub compensations: Vec<SagaStep>,
    pub state: SagaState,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SagaStep {
    pub id: Uuid,
    pub name: String,
    pub action: SagaAction,
    pub compensation: Option<SagaAction>,
    pub retry_policy: RetryPolicy,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum SagaAction {
    EmitEvent(Event),
    CallFunction(String),
    ExecuteCommand(serde_json::Value),
    Custom(serde_json::Value),
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_strategy: BackoffStrategy,
    pub initial_delay: u64,
    pub max_delay: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackoffStrategy {
    Fixed,
    Linear,
    Exponential,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SagaState {
    Pending,
    Running,
    Completed,
    Failed,
    Compensating,
    Compensated,
}

#[derive(Debug, Clone)]
pub struct EventCircuitBreaker {
    pub id: String,
    pub name: String,
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_time: Option<DateTime<Utc>>,
    pub config: CircuitBreakerConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout_ms: u64,
    pub reset_timeout_ms: u64,
}

impl EventBus {
    pub fn new(config: EventBusConfig) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            filters: Arc::new(RwLock::new(Vec::new())),
            queue: Arc::new(RwLock::new(EventQueue::new(config.max_queue_size))),
            config,
        }
    }

    pub fn emit(&mut self, event: Event) -> Result<()> {
        if self.config.enable_history {
            self.add_to_history(event.clone())?;
        }

        if self.config.enable_filters && !self.passes_filters(&event) {
            return Ok(());
        }

        let subscribers = self.get_subscribers_for_event(&event);

        for subscriber in subscribers {
            if subscriber.enabled {
                let result = (subscriber.handler)(&event);

                match result {
                    Ok(event_result) => {
                        if !event_result.success {
                        }
                    }
                    Err(e) => {
                    }
                }
            }
        }

        let mut events = self.events.write();
        if events.len() >= self.config.max_events {
            events.remove(0);
        }
        events.push(event);

        Ok(())
    }

    pub fn emit_async(&mut self, event: Event) -> Result<()> {
        if !self.config.enable_async {
            return self.emit(event);
        }

        let mut queue = self.queue.write();
        if queue.events.len() >= queue.max_size {
            return Err(EllasticError::LimitExceeded("Event queue is full".to_string()));
        }

        let queued_event = QueuedEvent {
            event,
            retry_count: 0,
            max_retries: 3,
            delay_until: None,
            created_at: Utc::now(),
        };

        queue.events.push(queued_event);
        Ok(())
    }

    pub fn subscribe(&mut self, subscriber: EventSubscriber) -> Result<Uuid> {
        let id = subscriber.id;

        let mut subscribers = self.subscribers.write();

        for event_type in &subscriber.event_types {
            if let Some(event_subscribers) = subscribers.get_mut(event_type) {
                event_subscribers.push(subscriber.clone());
            } else {
                subscribers.insert(event_type.clone(), vec![subscriber.clone()]);
            }
        }

        Ok(id)
    }

    pub fn unsubscribe(&mut self, subscriber_id: Uuid) -> Result<()> {
        let mut subscribers = self.subscribers.write();

        for event_subscribers in subscribers.values_mut() {
            event_subscribers.retain(|s| s.id != subscriber_id);
        }

        Ok(())
    }

    pub fn register_handler(&mut self, handler: EventHandler) -> Result<()> {
        let mut handlers = self.handlers.write();
        handlers.insert(handler.id.clone(), handler);
        Ok(())
    }

    pub fn unregister_handler(&mut self, handler_id: &str) -> Result<()> {
        let mut handlers = self.handlers.write();
        handlers.remove(handler_id);
        Ok(())
    }

    pub fn add_filter(&mut self, filter: EventFilter) -> Result<()> {
        let mut filters = self.filters.write();
        filters.push(filter);
        Ok(())
    }

    pub fn remove_filter(&mut self, filter_id: &str) -> Result<()> {
        let mut filters = self.filters.write();
        filters.retain(|f| f.id != filter_id);
        Ok(())
    }

    pub fn process_queue(&mut self) -> Result<()> {
        let mut queue = self.queue.write();

        if queue.processing {
            return Ok(());
        }

        queue.processing = true;

        let now = Utc::now();
        let mut to_remove = Vec::new();

        for (index, queued_event) in queue.events.iter_mut().enumerate() {
            if let Some(delay_until) = queued_event.delay_until {
                if now < delay_until {
                    continue;
                }
            }

            let result = self.emit(queued_event.event.clone());

            match result {
                Ok(_) => {
                    to_remove.push(index);
                }
                Err(e) => {
                    queued_event.retry_count += 1;

                    if queued_event.retry_count >= queued_event.max_retries {
                        to_remove.push(index);
                    } else {
                        queued_event.delay_until = Some(now + chrono::Duration::seconds(1));
                    }
                }
            }
        }

        for &index in to_remove.iter().rev() {
            queue.events.remove(index);
        }

        queue.processing = false;
        Ok(())
    }

    fn get_subscribers_for_event(&self, event: &Event) -> Vec<EventSubscriber> {
        let subscribers = self.subscribers.read();
        let mut result = Vec::new();

        if let Some(event_subscribers) = subscribers.get(&event.event_type) {
            for subscriber in event_subscribers {
                if !subscriber.sources.is_empty() && !subscriber.sources.contains(&event.source) {
                    continue;
                }

                if let Some(filter) = &subscriber.filter {
                    if !self.passes_filter(event, filter) {
                        continue;
                    }
                }

                result.push(subscriber.clone());
            }
        }

        result.sort_by(|a, b| b.priority.cmp(&a.priority));
        result
    }

    fn passes_filters(&self, event: &Event) -> bool {
        let filters = self.filters.read();

        for filter in filters.iter() {
            if filter.enabled && !self.passes_filter(event, filter) {
                return false;
            }
        }

        true
    }

    fn passes_filter(&self, event: &Event, filter: &EventFilter) -> bool {
        let mut results = Vec::new();

        for condition in &filter.conditions {
            let result = self.evaluate_condition(event, condition);
            results.push(result);
        }

        match filter.logic {
            FilterLogic::And => results.iter().all(|&r| r),
            FilterLogic::Or => results.iter().any(|&r| r),
            FilterLogic::Not => !results.iter().any(|&r| r),
        }
    }

    fn evaluate_condition(&self, event: &Event, condition: &FilterCondition) -> bool {
        let field_value = self.get_field_value(event, &condition.field);

        match condition.operator {
            FilterOperator::Equals => self.compare_values(&field_value, &condition.value, |a, b| a == b),
            FilterOperator::NotEquals => self.compare_values(&field_value, &condition.value, |a, b| a != b),
            FilterOperator::Contains => self.string_contains(&field_value, &condition.value, condition.case_sensitive),
            FilterOperator::NotContains => !self.string_contains(&field_value, &condition.value, condition.case_sensitive),
            FilterOperator::StartsWith => self.string_starts_with(&field_value, &condition.value, condition.case_sensitive),
            FilterOperator::EndsWith => self.string_ends_with(&field_value, &condition.value, condition.case_sensitive),
            FilterOperator::GreaterThan => self.compare_numbers(&field_value, &condition.value, |a, b| a > b),
            FilterOperator::LessThan => self.compare_numbers(&field_value, &condition.value, |a, b| a < b),
            FilterOperator::GreaterEqual => self.compare_numbers(&field_value, &condition.value, |a, b| a >= b),
            FilterOperator::LessEqual => self.compare_numbers(&field_value, &condition.value, |a, b| a <= b),
            FilterOperator::In => self.value_in_array(&field_value, &condition.value),
            FilterOperator::NotIn => !self.value_in_array(&field_value, &condition.value),
            FilterOperator::Regex => self.regex_match(&field_value, &condition.value, condition.case_sensitive),
            FilterOperator::Custom => false,
        }
    }

    fn get_field_value(&self, event: &Event, field: &str) -> serde_json::Value {
        match field {
            "event_type" => serde_json::Value::String(event.event_type.clone()),
            "source" => serde_json::Value::String(event.source.clone()),
            "target" => event.target.as_ref().map_or(serde_json::Value::Null, |t| serde_json::Value::String(t.clone())),
            "priority" => serde_json::Value::Number((event.priority as u8).into()),
            "timestamp" => serde_json::Value::String(event.timestamp.to_rfc3339()),
            _ => {
                if field.starts_with("metadata.") {
                    let key = field.strip_prefix("metadata.").unwrap();
                    match key {
                        "user_id" => event.metadata.user_id.as_ref().map_or(serde_json::Value::Null, |u| serde_json::Value::String(u.clone())),
                        "session_id" => event.metadata.session_id.as_ref().map_or(serde_json::Value::Null, |s| serde_json::Value::String(s.clone())),
                        "request_id" => event.metadata.request_id.as_ref().map_or(serde_json::Value::Null, |r| serde_json::Value::String(r.clone())),
                        "correlation_id" => event.metadata.correlation_id.as_ref().map_or(serde_json::Value::Null, |c| serde_json::Value::String(c.clone())),
                        "causation_id" => event.metadata.causation_id.as_ref().map_or(serde_json::Value::Null, |c| serde_json::Value::String(c.clone())),
                        "version" => serde_json::Value::String(event.metadata.version.clone()),
                        _ => serde_json::Value::Null,
                    }
                } else {
                    serde_json::Value::Null
                }
            }
        }
    }

    fn compare_values<F>(&self, a: &serde_json::Value, b: &serde_json::Value, compare_fn: F) -> bool
    where
        F: Fn(&serde_json::Value, &serde_json::Value) -> bool,
    {
        compare_fn(a, b)
    }

    fn string_contains(&self, a: &serde_json::Value, b: &serde_json::Value, case_sensitive: bool) -> bool {
        if let (serde_json::Value::String(a_str), serde_json::Value::String(b_str)) = (a, b) {
            if case_sensitive {
                a_str.contains(b_str)
            } else {
                a_str.to_lowercase().contains(&b_str.to_lowercase())
            }
        } else {
            false
        }
    }

    fn string_starts_with(&self, a: &serde_json::Value, b: &serde_json::Value, case_sensitive: bool) -> bool {
        if let (serde_json::Value::String(a_str), serde_json::Value::String(b_str)) = (a, b) {
            if case_sensitive {
                a_str.starts_with(b_str)
            } else {
                a_str.to_lowercase().starts_with(&b_str.to_lowercase())
            }
        } else {
            false
        }
    }

    fn string_ends_with(&self, a: &serde_json::Value, b: &serde_json::Value, case_sensitive: bool) -> bool {
        if let (serde_json::Value::String(a_str), serde_json::Value::String(b_str)) = (a, b) {
            if case_sensitive {
                a_str.ends_with(b_str)
            } else {
                a_str.to_lowercase().ends_with(&b_str.to_lowercase())
            }
        } else {
            false
        }
    }

    fn compare_numbers<F>(&self, a: &serde_json::Value, b: &serde_json::Value, compare_fn: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        if let (Some(a_num), Some(b_num)) = (a.as_f64(), b.as_f64()) {
            compare_fn(a_num, b_num)
        } else {
            false
        }
    }

    fn value_in_array(&self, a: &serde_json::Value, b: &serde_json::Value) -> bool {
        if let serde_json::Value::Array(array) = b {
            array.contains(a)
        } else {
            false
        }
    }

    fn regex_match(&self, a: &serde_json::Value, b: &serde_json::Value, case_sensitive: bool) -> bool {
        if let (serde_json::Value::String(a_str), serde_json::Value::String(b_str)) = (a, b) {
            self.string_contains(a, b, case_sensitive)
        } else {
            false
        }
    }

    fn add_to_history(&self, event: Event) -> Result<()> {
        Ok(())
    }

    pub fn create_event(&self, event_type: String, source: String, data: EventData) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type,
            source,
            target: None,
            data,
            metadata: EventMetadata::new(),
            timestamp: Utc::now(),
            priority: self.config.default_priority,
        }
    }

    pub fn create_event_with_metadata(&self, event_type: String, source: String, data: EventData, metadata: EventMetadata) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type,
            source,
            target: None,
            data,
            metadata,
            timestamp: Utc::now(),
            priority: self.config.default_priority,
        }
    }

    pub fn create_batch(&self, events: Vec<Event>, batch_type: BatchType) -> EventBatch {
        EventBatch {
            id: Uuid::new_v4(),
            events,
            batch_type,
            created_at: Utc::now(),
        }
    }

    pub fn emit_batch(&mut self, batch: EventBatch) -> Result<()> {
        match batch.batch_type {
            BatchType::Transactional => {
                let mut emitted_events = Vec::new();

                for event in batch.events {
                    match self.emit(event.clone()) {
                        Ok(_) => emitted_events.push(event),
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            }
            BatchType::NonTransactional => {
                for event in batch.events {
                    let _ = self.emit(event);
                }
            }
            BatchType::Ordered => {
                for event in batch.events {
                    self.emit(event)?;
                }
            }
            BatchType::Unordered => {
                for event in batch.events {
                    let _ = self.emit(event);
                }
            }
        }

        Ok(())
    }

    pub fn create_stream(&self, name: String, event_types: Vec<String>, max_buffer_size: usize) -> EventStream {
        EventStream {
            id: Uuid::new_v4(),
            name,
            event_types,
            buffer: Vec::new(),
            max_buffer_size,
            created_at: Utc::now(),
        }
    }

    pub fn get_metrics(&self) -> EventMetrics {
        let events = self.events.read();
        let mut metrics = EventMetrics {
            total_events: events.len() as u64,
            events_by_type: HashMap::new(),
            events_by_source: HashMap::new(),
            events_by_priority: HashMap::new(),
            processing_time: 0.0,
            error_rate: 0.0,
            throughput: 0.0,
        };

        for event in events.iter() {
            *metrics.events_by_type.entry(event.event_type.clone()).or_insert(0) += 1;
            *metrics.events_by_source.entry(event.source.clone()).or_insert(0) += 1;
            *metrics.events_by_priority.entry(event.priority).or_insert(0) += 1;
        }

        metrics
    }

    pub fn clone(&self) -> EventBus {
        EventBus {
            events: self.events.clone(),
            subscribers: self.subscribers.clone(),
            handlers: self.handlers.clone(),
            filters: self.filters.clone(),
            queue: self.queue.clone(),
            config: self.config.clone(),
        }
    }
}

impl EventQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            events: Vec::new(),
            processing: false,
            max_size,
        }
    }

    pub fn clone(&self) -> EventQueue {
        EventQueue {
            events: self.events.clone(),
            processing: self.processing,
            max_size: self.max_size,
        }
    }
}

impl EventMetadata {
    pub fn new() -> Self {
        Self {
            tags: Vec::new(),
            categories: Vec::new(),
            user_id: None,
            session_id: None,
            request_id: None,
            correlation_id: None,
            causation_id: None,
            version: "1.0".to_string(),
        }
    }

    pub fn clone(&self) -> EventMetadata {
        EventMetadata {
            tags: self.tags.clone(),
            categories: self.categories.clone(),
            user_id: self.user_id.clone(),
            session_id: self.session_id.clone(),
            request_id: self.request_id.clone(),
            correlation_id: self.correlation_id.clone(),
            causation_id: self.causation_id.clone(),
            version: self.version.clone(),
        }
    }
}

impl Default fn default() -> Self {
        Self {
            max_events: 10000,
            max_queue_size: 1000,
            enable_history: true,
            enable_filters: true,
            enable_async: true,
            default_priority: EventPriority::Normal,
        }
}

impl Default fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Default".to_string(),
            event_types: Vec::new(),
            sources: Vec::new(),
            filter: None,
            handler: Box::new(|_| Ok(EventResult::success())),
            priority: EventPriority::Normal,
            async: false,
            enabled: true,
            created_at: Utc::now(),
        }
}

impl Default fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Filter".to_string(),
            filter_type: FilterType::EventType,
            conditions: Vec::new(),
            logic: FilterLogic::And,
            enabled: true,
        }
}

impl Default fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_ms: 60000,
            reset_timeout_ms: 300000,
        }
}

impl EventResult {
    pub fn success() -> Self {
        Self {
            success: true,
            data: None,
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            metadata: HashMap::new(),
        }
    }

    pub fn with_data(data: EventData) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn clone(&self) -> EventResult {
        EventResult {
            success: self.success,
            data: self.data.clone(),
            error: self.error.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

pub fn create_event_bus(config: EventBusConfig) -> EventBus {
    EventBus::new(config)
}

pub fn create_event_bus_config() -> EventBusConfig {
    EventBusConfig::default()
}

pub fn create_event(event_type: String, source: String, data: EventData) -> Event {
    Event {
        id: Uuid::new_v4(),
        event_type,
        source,
        target: None,
        data,
        metadata: EventMetadata::new(),
        timestamp: Utc::now(),
        priority: EventPriority::Normal,
    }
}

pub fn create_event_with_priority(event_type: String, source: String, data: EventData, priority: EventPriority) -> Event {
    Event {
        id: Uuid::new_v4(),
        event_type,
        source,
        target: None,
        data,
        metadata: EventMetadata::new(),
        timestamp: Utc::now(),
        priority,
    }
}

pub fn create_event_data_string(value: String) -> EventData {
    EventData::String(value)
}

pub fn create_event_data_number(value: f64) -> EventData {
    EventData::Number(value)
}

pub fn create_event_data_boolean(value: bool) -> EventData {
    EventData::Boolean(value)
}

pub fn create_event_data_object(value: HashMap<String, serde_json::Value>) -> EventData {
    EventData::Object(value)
}

pub fn create_event_data_array(value: Vec<serde_json::Value>) -> EventData {
    EventData::Array(value)
}

pub fn create_event_data_binary(value: Vec<u8>) -> EventData {
    EventData::Binary(value)
}

pub fn create_event_data_custom(value: serde_json::Value) -> EventData {
    EventData::Custom(value)
}
