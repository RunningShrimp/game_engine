/**
 * 动画时间轴编辑器类型定义
 */

// ==================== 基础类型 ====================

export interface Vector3 {
  x: number;
  y: number;
  z: number;
}

export interface Quaternion {
  x: number;
  y: number;
  z: number;
  w: number;
}

export type KeyframeValue = number | Vector3 | Quaternion;

// ==================== 枚举类型 ====================

export enum TrackType {
  Transform = 'transform',    // 变换轨道
  Rotation = 'rotation',      // 旋转轨道
  Scale = 'scale',           // 缩放轨道
  Property = 'property',     // 属性轨道
  Event = 'event',           // 事件轨道
}

export enum InterpolationType {
  Constant = 'constant',     // 常量
  Linear = 'linear',         // 线性
  Cubic = 'cubic',          // 三次样条
  Hermite = 'hermite',       // Hermite
}

export enum EasingFunction {
  Linear = 'linear',
  EaseInQuad = 'easeInQuad',
  EaseOutQuad = 'easeOutQuad',
  EaseInOutQuad = 'easeInOutQuad',
  EaseInCubic = 'easeInCubic',
  EaseOutCubic = 'easeOutCubic',
  EaseInOutCubic = 'easeInOutCubic',
  EaseInQuart = 'easeInQuart',
  EaseOutQuart = 'easeOutQuart',
  EaseInOutQuart = 'easeInOutQuart',
  EaseInQuint = 'easeInQuint',
  EaseOutQuint = 'easeOutQuint',
  EaseInOutQuint = 'easeInOutQuint',
  EaseInElastic = 'easeInElastic',
  EaseOutElastic = 'easeOutElastic',
  EaseInOutElastic = 'easeInOutElastic',
  EaseInBounce = 'easeInBounce',
  EaseOutBounce = 'easeOutBounce',
  EaseInOutBounce = 'easeInOutBounce',
}

// ==================== 关键帧 ====================

export interface Keyframe {
  id: string;
  time: number;              // 时间（秒）
  value: KeyframeValue;      // 值
  interpolation: InterpolationType;
  easing: EasingFunction;
  inTangent?: Vector3;       // 入切线（仅用于三次样条）
  outTangent?: Vector3;      // 出切线
}

// ==================== 动画曲线 ====================

export interface AnimationCurve {
  id: string;
  name: string;
  propertyPath: string;      // 例如: "transform.position.x"
  keyframes: Keyframe[];
  color: string;             // 曲线颜色
  valueType: 'number' | 'vector3' | 'quaternion';
}

// ==================== 动画轨道 ====================

export interface AnimationTrack {
  id: string;
  name: string;              // 实体或属性名称
  type: TrackType;
  curves: AnimationCurve[];
  visible: boolean;
  locked: boolean;
  muted: boolean;
  expanded: boolean;         // 是否展开显示子曲线
  color: string;             // 轨道颜色
}

// ==================== 动画剪辑 ====================

export interface AnimationClip {
  id: string;
  name: string;
  duration: number;         // 时长（秒）
  frameRate: number;         // 帧率
  tracks: AnimationTrack[];
  loop: boolean;
  createdAt: number;
  updatedAt: number;
}

// ==================== 时间轴状态 ====================

export interface TimelineSelection {
  selectedTracks: Set<string>;      // 选中的轨道ID
  selectedKeyframes: Set<string>;   // 选中的关键帧ID
  selectedCurves: Set<string>;      // 选中的曲线ID
}

export interface TimelineState {
  currentTime: number;       // 当前时间（秒）
  playbackSpeed: number;     // 播放速度
  isPlaying: boolean;
  isLooping: boolean;
  selection: TimelineSelection;
  zoom: number;              // 时间缩放级别（像素/秒）
  scrollOffset: number;      // 水平滚动偏移
}

// ==================== 动画状态 ====================

export interface AnimationState {
  clipId: string;
  time: number;
  values: Map<string, KeyframeValue>;  // propertyPath -> value
}

// ==================== Dope Sheet视图数据 ====================

export interface DopeSheetRow {
  trackId: string;
  trackName: string;
  keyframes: {
    id: string;
    time: number;
    value: KeyframeValue;
  }[];
}

// ==================== 事件关键帧 ====================

export interface EventKeyframe extends Omit<Keyframe, 'value'> {
  value: string;             // 事件名称
  parameters?: Record<string, any>;  // 事件参数
}

// ==================== 曲线编辑器数据 ====================

export interface CurveEditorState {
  selectedCurves: Set<string>;
  showTangents: boolean;
  showGrid: boolean;
  minTime: number;
  maxTime: number;
  minValue: number;
  maxValue: number;
}

// ==================== Tauri API 数据传输对象 ====================

export interface KeyframeData {
  id: string;
  time: number;
  value: KeyframeValue;
  interpolation: InterpolationType;
  easing: EasingFunction;
  inTangent?: Vector3;
  outTangent?: Vector3;
}

export interface AnimationCurveData {
  id: string;
  name: string;
  propertyPath: string;
  keyframes: KeyframeData[];
  color: string;
  valueType: 'number' | 'vector3' | 'quaternion';
}

export interface AnimationTrackData {
  id: string;
  name: string;
  type: TrackType;
  curves: AnimationCurveData[];
  visible: boolean;
  locked: boolean;
  muted: boolean;
  expanded: boolean;
  color: string;
}

export interface AnimationClipData {
  id: string;
  name: string;
  duration: number;
  frameRate: number;
  tracks: AnimationTrackData[];
  loop: boolean;
  createdAt: number;
  updatedAt: number;
}

// ==================== Easing函数实现 ====================

export const EasingFunctions: Record<EasingFunction, (t: number) => number> = {
  [EasingFunction.Linear]: (t: number) => t,

  [EasingFunction.EaseInQuad]: (t: number) => t * t,
  [EasingFunction.EaseOutQuad]: (t: number) => t * (2 - t),
  [EasingFunction.EaseInOutQuad]: (t: number) =>
    t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t,

  [EasingFunction.EaseInCubic]: (t: number) => t * t * t,
  [EasingFunction.EaseOutCubic]: (t: number) => --t * t * t + 1,
  [EasingFunction.EaseInOutCubic]: (t: number) =>
    t < 0.5 ? 4 * t * t * t : (t - 1) * (2 * t - 2) * (2 * t - 2) + 1,

  [EasingFunction.EaseInQuart]: (t: number) => t * t * t * t,
  [EasingFunction.EaseOutQuart]: (t: number) => 1 - --t * t * t * t,
  [EasingFunction.EaseInOutQuart]: (t: number) =>
    t < 0.5 ? 8 * t * t * t * t : 1 - 8 * --t * t * t * t,

  [EasingFunction.EaseInQuint]: (t: number) => t * t * t * t * t,
  [EasingFunction.EaseOutQuint]: (t: number) => 1 + --t * t * t * t * t,
  [EasingFunction.EaseInOutQuint]: (t: number) =>
    t < 0.5 ? 16 * t * t * t * t * t : 1 + 16 * --t * t * t * t * t,

  [EasingFunction.EaseInElastic]: (t: number) => {
    const c4 = (2 * Math.PI) / 3;
    return t === 0 ? 0 : t === 1 ? 1 : -Math.pow(2, 10 * t - 10) * Math.sin((t * 10 - 10.75) * c4);
  },
  [EasingFunction.EaseOutElastic]: (t: number) => {
    const c4 = (2 * Math.PI) / 3;
    return t === 0 ? 0 : t === 1 ? 1 : Math.pow(2, -10 * t) * Math.sin((t * 10 - 0.75) * c4) + 1;
  },
  [EasingFunction.EaseInOutElastic]: (t: number) => {
    const c5 = (2 * Math.PI) / 4.5;
    return t === 0
      ? 0
      : t === 1
      ? 1
      : t < 0.5
      ? -(Math.pow(2, 20 * t - 10) * Math.sin((20 * t - 11.125) * c5)) / 2
      : (Math.pow(2, -20 * t + 10) * Math.sin((20 * t - 11.125) * c5)) / 2 + 1;
  },

  [EasingFunction.EaseInBounce]: (t: number) => 1 - EasingFunctions[EasingFunction.EaseOutBounce](1 - t),
  [EasingFunction.EaseOutBounce]: (t: number) => {
    const n1 = 7.5625;
    const d1 = 2.75;
    if (t < 1 / d1) {
      return n1 * t * t;
    } else if (t < 2 / d1) {
      return n1 * (t -= 1.5 / d1) * t + 0.75;
    } else if (t < 2.5 / d1) {
      return n1 * (t -= 2.25 / d1) * t + 0.9375;
    } else {
      return n1 * (t -= 2.625 / d1) * t + 0.984375;
    }
  },
  [EasingFunction.EaseInOutBounce]: (t: number) =>
    t < 0.5
      ? (1 - EasingFunctions[EasingFunction.EaseOutBounce](1 - 2 * t)) / 2
      : (1 + EasingFunctions[EasingFunction.EaseOutBounce](2 * t - 1)) / 2,
};

// ==================== 工具函数 ====================

/**
 * 应用easing函数
 */
export function applyEasing(t: number, easing: EasingFunction): number {
  const fn = EasingFunctions[easing];
  return fn(Math.max(0, Math.min(1, t)));
}

/**
 * 在两个关键帧之间插值
 */
export function interpolateKeyframes(
  time: number,
  prevKeyframe: Keyframe,
  nextKeyframe: Keyframe
): KeyframeValue {
  const { time: t1, value: v1, interpolation, easing } = prevKeyframe;
  const { time: t2, value: v2 } = nextKeyframe;

  if (interpolation === InterpolationType.Constant) {
    return v1;
  }

  const t = (time - t1) / (t2 - t1);
  const easedT = applyEasing(t, easing);

  if (typeof v1 === 'number' && typeof v2 === 'number') {
    if (interpolation === InterpolationType.Linear) {
      return v1 + (v2 - v1) * easedT;
    } else {
      // 对于三次样条和Hermite插值，这里简化处理
      return v1 + (v2 - v1) * easedT;
    }
  } else if (
    typeof v1 === 'object' &&
    typeof v2 === 'object' &&
    'x' in v1 &&
    'x' in v2
  ) {
    // Vector3插值
    return {
      x: v1.x + (v2.x - v1.x) * easedT,
      y: v1.y + (v2.y - v1.y) * easedT,
      z: v1.z + (v2.z - v1.z) * easedT,
    };
  } else if (
    typeof v1 === 'object' &&
    typeof v2 === 'object' &&
    'w' in v1 &&
    'w' in v2
  ) {
    // Quaternion插值（简化版）
    // 实际应该使用slerp
    return {
      x: v1.x + (v2.x - v1.x) * easedT,
      y: v1.y + (v2.y - v1.y) * easedT,
      z: v1.z + (v2.z - v1.z) * easedT,
      w: v1.w + (v2.w - v1.w) * easedT,
    };
  }

  return v1;
}

/**
 * 获取轨道默认颜色
 */
export function getTrackColor(type: TrackType): string {
  switch (type) {
    case TrackType.Transform:
      return '#ef4444'; // red
    case TrackType.Rotation:
      return '#22c55e'; // green
    case TrackType.Scale:
      return '#3b82f6'; // blue
    case TrackType.Property:
      return '#a855f7'; // purple
    case TrackType.Event:
      return '#f59e0b'; // amber
    default:
      return '#64748b'; // slate
  }
}

/**
 * 创建空动画剪辑
 */
export function createEmptyAnimationClip(name: string): AnimationClip {
  return {
    id: `clip_${Date.now()}`,
    name,
    duration: 5.0,
    frameRate: 60,
    tracks: [],
    loop: false,
    createdAt: Date.now(),
    updatedAt: Date.now(),
  };
}

/**
 * 创建空动画轨道
 */
export function createEmptyTrack(
  name: string,
  type: TrackType,
  entityId?: string
): AnimationTrack {
  return {
    id: `track_${Date.now()}`,
    name,
    type,
    curves: [],
    visible: true,
    locked: false,
    muted: false,
    expanded: true,
    color: getTrackColor(type),
  };
}

/**
 * 创建关键帧
 */
export function createKeyframe(
  time: number,
  value: KeyframeValue,
  interpolation: InterpolationType = InterpolationType.Linear,
  easing: EasingFunction = EasingFunction.Linear
): Keyframe {
  return {
    id: `keyframe_${Date.now()}_${Math.random()}`,
    time,
    value,
    interpolation,
    easing,
  };
}
