use chrono::{
  DateTime,
  Utc,
};
use ellastic_audio::{
  AudioData,
  AudioProcessor,
};
use ellastic_core::{
  MediaData,
  MediaType,
};
use ellastic_effects::{
  EffectProcessor,
  EffectType,
};
use ellastic_errors::{
  EllasticError,
  Result,
};
use ellastic_glitch::{
  GlitchEffect,
  GlitchProcessor,
};
use ellastic_image::{
  ImageData,
  ImageProcessor,
};
use ellastic_media::MediaProcessor;
use ellastic_utils::create_random_generator;
use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::{
  HashMap,
  HashSet,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PipelineGraph {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub nodes: HashMap<Uuid, PipelineNode>,
  pub edges: HashMap<Uuid, PipelineEdge>,
  pub node_connections: HashMap<Uuid, Vec<Uuid>>,
  pub edge_connections: HashMap<Uuid, (Uuid, Uuid)>,
  pub input_nodes: Vec<Uuid>,
  pub output_nodes: Vec<Uuid>,
  pub metadata: HashMap<String, String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PipelineNode {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub node_type: PipelineNodeType,
  pub position: (f32, f32),
  pub parameters: HashMap<String, String>,
  pub input_ports: Vec<Port>,
  pub output_ports: Vec<Port>,
  pub enabled: bool,
  pub metadata: HashMap<String, String>,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum PipelineNodeType {
  Input { media_type: MediaType },
  Output { media_type: MediaType },
  Effect { effect_type: EffectType },
  Filter { filter_type: FilterType },
  Transform { transform_type: TransformType },
  Branch { condition: BranchCondition },
  Merge { merge_type: MergeType },
  TemplateNode { template_node_name: String },
  Custom { custom_type: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
  LowPass,
  HighPass,
  BandPass,
  Notch,
  Gaussian,
  Median,
  Bilateral,
  Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformType {
  Scale,
  Rotate,
  Translate,
  Crop,
  Flip,
  Resize,
  ColorSpace,
  Custom,
}

#[derive(Debug, Clone)]
pub enum BranchCondition {
  MediaType(MediaType),
  Parameter {
    parameter: String,
    operator: ComparisonOperator,
    value: String,
  },
  Custom {
    condition_function: Box<dyn Fn(&MediaProcessor) -> bool + Send + Sync>,
  },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
  Equals,
  NotEquals,
  GreaterThan,
  LessThan,
  Contains,
  NotContains,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeType {
  Concat,
  Blend,
  Overlay,
  Composite,
  Custom,
}

#[derive(Debug, Clone)]
pub struct Port {
  pub id: Uuid,
  pub name: String,
  pub port_type: PortType,
  pub data_type: DataType,
  pub connected_to: Option<Uuid>,
  pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortType {
  Input,
  Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
  Image,
  Audio,
  Video,
  Generic,
  Custom(String),
}

#[derive(Debug, Clone)]
pub struct PipelineEdge {
  pub id: Uuid,
  pub from_node: Uuid,
  pub to_node: Uuid,
  pub from_port: Uuid,
  pub to_port: Uuid,
  pub edge_type: EdgeType,
  pub weight: f32,
  pub enabled: bool,
  pub metadata: HashMap<String, String>,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
  Data,
  Control,
  Feedback,
  Custom,
}

impl PipelineGraph {
  pub fn new() -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      name: "Untitled Pipeline".to_string(),
      description: String::new(),
      nodes: HashMap::new(),
      edges: HashMap::new(),
      node_connections: HashMap::new(),
      edge_connections: HashMap::new(),
      input_nodes: Vec::new(),
      output_nodes: Vec::new(),
      metadata: HashMap::new(),
      created_at: now,
      updated_at: now,
    }
  }

  pub fn with_name(mut self, name: String) -> Self {
    self.name = name;
    self
  }

  pub fn with_description(mut self, description: String) -> Self {
    self.description = description;
    self
  }

  pub fn add_node(&mut self, node: PipelineNode) -> Result<Uuid> {
    if self.nodes.contains_key(&node.id) {
      return Err(EllasticError::AlreadyExists(format!(
        "Node with ID {} already exists",
        node.id
      )));
    }

    let node_id = node.id;

    self.node_connections.insert(node_id, Vec::new());

    match node.node_type {
      PipelineNodeType::Input { .. } => {
        if !self.input_nodes.contains(&node_id) {
          self.input_nodes.push(node_id);
        }
      }
      PipelineNodeType::Output { .. } => {
        if !self.output_nodes.contains(&node_id) {
          self.output_nodes.push(node_id);
        }
      }
      _ => {}
    }

    self.nodes.insert(node_id, node);
    self.update_timestamp();
    Ok(node_id)
  }

  pub fn remove_node(&mut self, node_id: Uuid) -> Result<PipelineNode> {
    let node = self
      .nodes
      .remove(&node_id)
      .ok_or_else(|| EllasticError::InvalidParameter(format!("Node {} not found", node_id)))?;

    let edges_to_remove: Vec<Uuid> = self
      .edges
      .iter()
      .filter(|(_, edge)| edge.from_node == node_id || edge.to_node == node_id)
      .map(|(edge_id, _)| *edge_id)
      .collect();

    for edge_id in edges_to_remove {
      self.remove_edge(edge_id)?;
    }

    self.node_connections.remove(&node_id);

    self.input_nodes.retain(|&id| id != node_id);
    self.output_nodes.retain(|&id| id != node_id);

    self.update_timestamp();
    Ok(node)
  }

  pub fn get_node(&self, node_id: Uuid) -> Option<&PipelineNode> {
    self.nodes.get(&node_id)
  }

  pub fn get_node_mut(&mut self, node_id: Uuid) -> Option<&mut PipelineNode> {
    self.nodes.get_mut(&node_id)
  }

  pub fn add_edge(&mut self, from_node: Uuid, to_node: Uuid, edge_type: EdgeType) -> Result<Uuid> {
    if !self.nodes.contains_key(&from_node) {
      return Err(EllasticError::InvalidParameter(format!(
        "From node {} not found",
        from_node
      )));
    }
    if !self.nodes.contains_key(&to_node) {
      return Err(EllasticError::InvalidParameter(format!(
        "To node {} not found",
        to_node
      )));
    }

    let from_port = self.nodes[&from_node]
      .output_ports
      .iter()
      .find(|port| port.connected_to.is_none())
      .ok_or_else(|| EllasticError::InvalidParameter("No available output port".to_string()))?;

    let to_port = self.nodes[&to_node]
      .input_ports
      .iter()
      .find(|port| port.connected_to.is_none())
      .ok_or_else(|| EllasticError::InvalidParameter("No available input port".to_string()))?;

    if self.would_create_cycle(from_node, to_node) {
      return Err(EllasticError::InvalidParameter(
        "Adding this edge would create a cycle".to_string(),
      ));
    }

    let edge_id = Uuid::new_v4();
    let edge = PipelineEdge {
      id: edge_id,
      from_node,
      to_node,
      from_port: from_port.id,
      to_port: to_port.id,
      edge_type,
      weight: 1.0,
      enabled: true,
      metadata: HashMap::new(),
      created_at: Utc::now(),
    };

    self.edge_connections.insert(edge_id, (from_node, to_node));
    self
      .node_connections
      .get_mut(&from_node)
      .unwrap()
      .push(to_node);

    if let Some(from_node_mut) = self.nodes.get_mut(&from_node) {
      for port in &mut from_node_mut.output_ports {
        if port.id == from_port.id {
          port.connected_to = Some(to_port.id);
          break;
        }
      }
    }

    if let Some(to_node_mut) = self.nodes.get_mut(&to_node) {
      for port in &mut to_node_mut.input_ports {
        if port.id == to_port.id {
          port.connected_to = Some(from_port.id);
          break;
        }
      }
    }

    self.edges.insert(edge_id, edge);
    self.update_timestamp();
    Ok(edge_id)
  }

  pub fn remove_edge(&mut self, edge_id: Uuid) -> Result<PipelineEdge> {
    let edge = self
      .edges
      .remove(&edge_id)
      .ok_or_else(|| EllasticError::InvalidParameter(format!("Edge {} not found", edge_id)))?;

    if let Some(connections) = self.node_connections.get_mut(&edge.from_node) {
      connections.retain(|&id| id != edge.to_node);
    }

    self.edge_connections.remove(&edge_id);

    if let Some(from_node_mut) = self.nodes.get_mut(&edge.from_node) {
      for port in &mut from_node_mut.output_ports {
        if port.id == edge.from_port {
          port.connected_to = None;
          break;
        }
      }
    }

    if let Some(to_node_mut) = self.nodes.get_mut(&edge.to_node) {
      for port in &mut to_node_mut.input_ports {
        if port.id == edge.to_port {
          port.connected_to = None;
          break;
        }
      }
    }

    self.update_timestamp();
    Ok(edge)
  }

  pub fn get_edge(&self, edge_id: Uuid) -> Option<&PipelineEdge> {
    self.edges.get(&edge_id)
  }

  pub fn get_edge_mut(&mut self, edge_id: Uuid) -> Option<&mut PipelineEdge> {
    self.edges.get_mut(&edge_id)
  }

  pub fn list_nodes(&self) -> Vec<&PipelineNode> {
    self.nodes.values().collect()
  }

  pub fn list_edges(&self) -> Vec<&PipelineEdge> {
    self.edges.values().collect()
  }

  pub fn get_input_nodes(&self) -> Vec<&PipelineNode> {
    self
      .input_nodes
      .iter()
      .filter_map(|id| self.nodes.get(id))
      .collect()
  }

  pub fn get_output_nodes(&self) -> Vec<&PipelineNode> {
    self
      .output_nodes
      .iter()
      .filter_map(|id| self.nodes.get(id))
      .collect()
  }

  pub fn get_connected_nodes(&self, node_id: Uuid) -> Vec<&PipelineNode> {
    self
      .node_connections
      .get(&node_id)
      .map(|connections| {
        connections
          .iter()
          .filter_map(|id| self.nodes.get(id))
          .collect()
      })
      .unwrap_or_default()
  }

  pub fn validate(&self) -> Result<()> {
    if self.input_nodes.is_empty() {
      return Err(EllasticError::InvalidParameter(
        "Pipeline must have at least one input node".to_string(),
      ));
    }

    if self.output_nodes.is_empty() {
      return Err(EllasticError::InvalidParameter(
        "Pipeline must have at least one output node".to_string(),
      ));
    }

    if self.has_cycles() {
      return Err(EllasticError::InvalidParameter(
        "Pipeline contains cycles".to_string(),
      ));
    }

    let visited = self.get_reachable_nodes();
    for node_id in self.nodes.keys() {
      if !visited.contains(node_id) && !self.input_nodes.contains(node_id) {
        return Err(EllasticError::InvalidParameter(format!(
          "Node {} is disconnected from the pipeline",
          node_id
        )));
      }
    }

    for edge in self.edges.values() {
      if edge.from_port == edge.to_port {
        return Err(EllasticError::InvalidParameter(
          "Edge cannot connect port to itself".to_string(),
        ));
      }

      let from_node = self
        .nodes
        .get(&edge.from_node)
        .ok_or_else(|| EllasticError::InvalidParameter("From node not found".to_string()))?;

      let to_node = self
        .nodes
        .get(&edge.to_node)
        .ok_or_else(|| EllasticError::InvalidParameter("To node not found".to_string()))?;

      if !from_node
        .output_ports
        .iter()
        .any(|p| p.id == edge.from_port)
      {
        return Err(EllasticError::InvalidParameter(
          "From port not found".to_string(),
        ));
      }

      if !to_node.input_ports.iter().any(|p| p.id == edge.to_port) {
        return Err(EllasticError::InvalidParameter(
          "To port not found".to_string(),
        ));
      }
    }

    Ok(())
  }

  fn would_create_cycle(&self, from_node: Uuid, to_node: Uuid) -> bool {
    let mut visited = HashSet::new();
    self.has_cycle_dfs(to_node, from_node, &mut visited)
  }

  fn has_cycle_dfs(&self, current: Uuid, target: Uuid, visited: &mut HashSet<Uuid>) -> bool {
    if current == target {
      return true;
    }

    if visited.contains(&current) {
      return false;
    }

    visited.insert(current);

    if let Some(connections) = self.node_connections.get(&current) {
      for &neighbor in connections {
        if self.has_cycle_dfs(neighbor, target, visited) {
          return true;
        }
      }
    }

    false
  }

  fn has_cycles(&self) -> bool {
    let mut visited = HashSet::new();
    let mut recursion_stack = HashSet::new();

    for node_id in self.nodes.keys() {
      if !visited.contains(node_id) {
        if self.has_cycle_dfs_recursive(*node_id, &mut visited, &mut recursion_stack) {
          return true;
        }
      }
    }

    false
  }

  fn has_cycle_dfs_recursive(
    &self,
    current: Uuid,
    visited: &mut HashSet<Uuid>,
    recursion_stack: &mut HashSet<Uuid>,
  ) -> bool {
    visited.insert(current);
    recursion_stack.insert(current);

    if let Some(connections) = self.node_connections.get(&current) {
      for &neighbor in connections {
        if !visited.contains(&neighbor) {
          if self.has_cycle_dfs_recursive(neighbor, visited, recursion_stack) {
            return true;
          }
        } else if recursion_stack.contains(&neighbor) {
          return true;
        }
      }
    }

    recursion_stack.remove(&current);
    false
  }

  fn get_reachable_nodes(&self) -> HashSet<Uuid> {
    let mut reachable = HashSet::new();

    for &input_node in &self.input_nodes {
      self.dfs_reachable(input_node, &mut reachable);
    }

    reachable
  }

  fn dfs_reachable(&self, current: Uuid, reachable: &mut HashSet<Uuid>) {
    reachable.insert(current);

    if let Some(connections) = self.node_connections.get(&current) {
      for &neighbor in connections {
        if !reachable.contains(&neighbor) {
          self.dfs_reachable(neighbor, reachable);
        }
      }
    }
  }

  pub fn clone(&self) -> PipelineGraph {
    PipelineGraph {
      id: self.id,
      name: self.name.clone(),
      description: self.description.clone(),
      nodes: self.nodes.clone(),
      edges: self.edges.clone(),
      node_connections: self.node_connections.clone(),
      edge_connections: self.edge_connections.clone(),
      input_nodes: self.input_nodes.clone(),
      output_nodes: self.output_nodes.clone(),
      metadata: self.metadata.clone(),
      created_at: self.created_at,
      updated_at: self.updated_at,
    }
  }

  fn update_timestamp(&mut self) {
    self.updated_at = Utc::now();
  }
}

impl PipelineNode {
  pub fn new(name: String, description: String, node_type: PipelineNodeType) -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      name,
      description,
      node_type,
      position: (0.0, 0.0),
      parameters: HashMap::new(),
      input_ports: Vec::new(),
      output_ports: Vec::new(),
      enabled: true,
      metadata: HashMap::new(),
      created_at: now,
    }
  }

  pub fn with_position(mut self, x: f32, y: f32) -> Self {
    self.position = (x, y);
    self
  }

  pub fn with_parameters(mut self, parameters: HashMap<String, String>) -> Self {
    self.parameters = parameters;
    self
  }

  pub fn with_input_ports(mut self, ports: Vec<Port>) -> Self {
    self.input_ports = ports;
    self
  }

  pub fn with_output_ports(mut self, ports: Vec<Port>) -> Self {
    self.output_ports = ports;
    self
  }

  pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
    self.metadata = metadata;
    self
  }

  pub fn enabled(mut self, enabled: bool) -> Self {
    self.enabled = enabled;
    self
  }

  pub fn parameters(&self) -> &HashMap<String, String> {
    &self.parameters
  }

  pub fn parameters_mut(&mut self) -> &mut HashMap<String, String> {
    &mut self.parameters
  }

  pub fn input_ports(&self) -> &Vec<Port> {
    &self.input_ports
  }

  pub fn input_ports_mut(&mut self) -> &mut Vec<Port> {
    &mut self.input_ports
  }

  pub fn output_ports(&self) -> &Vec<Port> {
    &self.output_ports
  }

  pub fn output_ports_mut(&mut self) -> &mut Vec<Port> {
    &mut self.output_ports
  }

  pub fn add_input_port(&mut self, port: Port) -> Result<Uuid> {
    if self.input_ports.iter().any(|p| p.name == port.name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Input port '{}' already exists",
        port.name
      )));
    }

    let port_id = port.id;
    self.input_ports.push(port);
    Ok(port_id)
  }

  pub fn add_output_port(&mut self, port: Port) -> Result<Uuid> {
    if self.output_ports.iter().any(|p| p.name == port.name) {
      return Err(EllasticError::AlreadyExists(format!(
        "Output port '{}' already exists",
        port.name
      )));
    }

    let port_id = port.id;
    self.output_ports.push(port);
    Ok(port_id)
  }

  pub fn remove_input_port(&mut self, port_id: Uuid) -> Option<Port> {
    let index = self.input_ports.iter().position(|p| p.id == port_id)?;
    Some(self.input_ports.remove(index))
  }

  pub fn remove_output_port(&mut self, port_id: Uuid) -> Option<Port> {
    let index = self.output_ports.iter().position(|p| p.id == port_id)?;
    Some(self.output_ports.remove(index))
  }

  pub fn get_input_port(&self, port_id: Uuid) -> Option<&Port> {
    self.input_ports.iter().find(|p| p.id == port_id)
  }

  pub fn get_output_port(&self, port_id: Uuid) -> Option<&Port> {
    self.output_ports.iter().find(|p| p.id == port_id)
  }

  pub fn clone(&self) -> PipelineNode {
    PipelineNode {
      id: self.id,
      name: self.name.clone(),
      description: self.description.clone(),
      node_type: self.node_type.clone(),
      position: self.position,
      parameters: self.parameters.clone(),
      input_ports: self.input_ports.clone(),
      output_ports: self.output_ports.clone(),
      enabled: self.enabled,
      metadata: self.metadata.clone(),
      created_at: self.created_at,
    }
  }
}

impl Port {
  pub fn new(name: String, port_type: PortType, data_type: DataType) -> Self {
    Self {
      id: Uuid::new_v4(),
      name,
      port_type,
      data_type,
      connected_to: None,
      required: false,
    }
  }

  pub fn required(mut self, required: bool) -> Self {
    self.required = required;
    self
  }

  pub fn clone(&self) -> Port {
    Port {
      id: self.id,
      name: self.name.clone(),
      port_type: self.port_type,
      data_type: self.data_type,
      connected_to: self.connected_to,
      required: self.required,
    }
  }
}

impl PipelineEdge {
  pub fn new(
    from_node: Uuid,
    to_node: Uuid,
    from_port: Uuid,
    to_port: Uuid,
    edge_type: EdgeType,
  ) -> Self {
    Self {
      id: Uuid::new_v4(),
      from_node,
      to_node,
      from_port,
      to_port,
      edge_type,
      weight: 1.0,
      enabled: true,
      metadata: HashMap::new(),
      created_at: Utc::now(),
    }
  }

  pub fn with_weight(mut self, weight: f32) -> Self {
    self.weight = weight;
    self
  }

  pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
    self.metadata = metadata;
    self
  }

  pub fn enabled(mut self, enabled: bool) -> Self {
    self.enabled = enabled;
    self
  }

  pub fn clone(&self) -> PipelineEdge {
    PipelineEdge {
      id: self.id,
      from_node: self.from_node,
      to_node: self.to_node,
      from_port: self.from_port,
      to_port: self.to_port,
      edge_type: self.edge_type,
      weight: self.weight,
      enabled: self.enabled,
      metadata: self.metadata.clone(),
      created_at: self.created_at,
    }
  }
}

pub fn create_pipeline_graph() -> PipelineGraph {
  PipelineGraph::new()
}

pub fn create_pipeline_node(
  name: String,
  description: String,
  node_type: PipelineNodeType,
) -> PipelineNode {
  PipelineNode::new(name, description, node_type)
}

pub fn create_port(name: String, port_type: PortType, data_type: DataType) -> Port {
  Port::new(name, port_type, data_type)
}

pub fn create_pipeline_edge(
  from_node: Uuid,
  to_node: Uuid,
  from_port: Uuid,
  to_port: Uuid,
  edge_type: EdgeType,
) -> PipelineEdge {
  PipelineEdge::new(from_node, to_node, from_port, to_port, edge_type)
}
