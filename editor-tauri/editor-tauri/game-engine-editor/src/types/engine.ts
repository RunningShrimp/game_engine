// Game Engine Editor Type Definitions

export interface Vector3 {
  x: number;
  y: number;
  z: number;
}

export interface Vector4 {
  x: number;
  y: number;
  z: number;
  w: number;
}

export interface Quaternion {
  x: number;
  y: number;
  z: number;
  w: number;
}

export interface Transform {
  position: Vector3;
  rotation: Quaternion;
  scale: Vector3;
}

export interface Entity {
  id: string;
  name: string;
  transform: Transform;
  components: Component[];
  children: Entity[];
  parentId?: string;
  visible: boolean;
  locked: boolean;
}

export interface Component {
  id: string;
  type: string;
  name: string;
  enabled: boolean;
  properties: Record<string, any>;
}

export interface Material {
  id: string;
  name: string;
  shader: string;
  properties: Record<string, any>;
  textures: Record<string, string>;
}

export interface Mesh {
  id: string;
  name: string;
  vertices: number[];
  indices: number[];
  normals: number[];
  uvs: number[][];
}

export interface Scene {
  id: string;
  name: string;
  entities: Entity[];
  settings: SceneSettings;
}

export interface SceneSettings {
  gravity: Vector3;
  ambientLight: Vector3;
  backgroundColor: Vector4;
  fogEnabled: boolean;
  fogColor: Vector3;
  fogDensity: number;
}

export interface Selection {
  entities: string[];
  components: string[];
}

export enum TransformMode {
  Translate = 'translate',
  Rotate = 'rotate',
  Scale = 'scale',
}

export enum Space {
  World = 'world',
  Local = 'local',
}

export interface EditorState {
  currentScene: Scene;
  selection: Selection;
  transformMode: TransformMode;
  space: Space;
  isPlaying: boolean;
  isPaused: boolean;
  gridSize: number;
  snapEnabled: boolean;
  snapValue: number;
}

export interface Gizmo {
  mode: TransformMode;
  space: Space;
  visible: boolean;
  size: number;
}
