/**
 * Material Node System Type Definitions
 * Defines the core types for the node-based material editor
 */

/**
 * Node types available in the material editor
 */
export enum NodeType {
  // Input nodes
  TextureInput = 'texture_input',
  ColorInput = 'color_input',
  FloatInput = 'float_input',
  Vector2Input = 'vector2_input',
  Vector3Input = 'vector3_input',
  Vector4Input = 'vector4_input',

  // Math/Operation nodes
  Multiply = 'multiply',
  Add = 'add',
  Subtract = 'subtract',
  Divide = 'divide',
  Mix = 'mix',
  Lerp = 'lerp',
  Normalize = 'normalize',
  Power = 'power',
  Sqrt = 'sqrt',
  Abs = 'abs',
  Min = 'min',
  Max = 'max',
  Clamp = 'clamp',

  // PBR nodes
  PBRMaster = 'pbr_master',
  Metallic = 'metallic',
  Roughness = 'roughness',
  NormalMap = 'normal_map',
  AmbientOcclusion = 'ambient_occlusion',
  Emission = 'emission',

  // UV nodes
  UVCoordinate = 'uv_coordinate',
  TextureMapping = 'texture_mapping',
  UVScroll = 'uv_scroll',

  // Texture nodes
  TextureSample = 'texture_sample',
  TextureParameter = 'texture_parameter',

  // Output node
  MaterialOutput = 'material_output',
}

/**
 * Data types for node ports
 */
export enum PortDataType {
  Float = 'float',
  Vector2 = 'vector2',
  Vector3 = 'vector3',
  Vector4 = 'vector4',
  Color = 'color',
  Texture2D = 'texture2d',
}

/**
 * Node port definition
 */
export interface NodePort {
  id: string;
  name: string;
  dataType: PortDataType;
  connectedTo: string | null; // ID of the connected port
  nodeId: string; // ID of the node this port belongs to
  isInput: boolean; // true for input, false for output
}

/**
 * Node parameter definition
 */
export interface NodeParameter {
  id: string;
  name: string;
  dataType: PortDataType;
  value: any;
  min?: number;
  max?: number;
  step?: number;
}

/**
 * Material node definition
 */
export interface MaterialNode {
  id: string;
  type: NodeType;
  position: { x: number; y: number };
  inputs: NodePort[];
  outputs: NodePort[];
  parameters: NodeParameter[];
  label?: string; // Custom label for the node
  selected?: boolean;
}

/**
 * Connection between two nodes
 */
export interface NodeConnection {
  id: string;
  fromNodeId: string;
  fromPortId: string;
  toNodeId: string;
  toPortId: string;
}

/**
 * Complete material definition
 */
export interface Material {
  id: string;
  name: string;
  nodes: MaterialNode[];
  connections: NodeConnection[];
  previewMesh?: 'sphere' | 'cube' | 'plane';
}

/**
 * Material preset for quick start
 */
export interface MaterialPreset {
  name: string;
  description: string;
  material: Material;
}

/**
 * Category for organizing nodes in the palette
 */
export interface NodeCategory {
  name: string;
  nodeTypes: NodeType[];
}

/**
 * Texture asset reference
 */
export interface TextureAsset {
  id: string;
  name: string;
  path: string;
  type: 'diffuse' | 'normal' | 'metallic' | 'roughness' | 'ao' | 'emission';
}

/**
 * Material export format (JSON)
 */
export interface MaterialExport {
  version: string;
  material: Material;
  metadata: {
    exportedAt: string;
    editorVersion: string;
  };
}
