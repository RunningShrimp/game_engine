/**
 * Text Atom Component Types
 *
 * 原子级文本组件的类型定义
 */

import type { CSSProperties, ReactNode } from 'react';

/**
 * 文本变体 - 决定使用哪个HTML标签和默认样式
 */
export type TextVariant =
  | 'h1'        // 主标题
  | 'h2'        // 二级标题
  | 'h3'        // 三级标题
  | 'h4'        // 四级标题
  | 'h5'        // 五级标题
  | 'h6'        // 六级标题
  | 'body'      // 正文
  | 'body-lg'   // 大号正文
  | 'body-sm'   // 小号正文
  | 'caption'   // 说明文字
  | 'overline'  // 上标标签
  | 'subtitle1' // 副标题1
  | 'subtitle2' // 副标题2
  | 'button'    // 按钮文字
  | 'code';     // 代码

/**
 * 字体大小
 */
export type TextSize =
  | 'xs'    // 0.75rem (12px)
  | 'sm'    // 0.875rem (14px)
  | 'base'  // 1rem (16px)
  | 'lg'    // 1.125rem (18px)
  | 'xl'    // 1.25rem (20px)
  | '2xl'   // 1.5rem (24px)
  | '3xl'   // 1.875rem (30px)
  | '4xl'   // 2.25rem (36px)
  | '5xl'   // 3rem (48px)
  | '6xl';  // 3.75rem (60px)

/**
 * 字体粗细
 */
export type TextWeight =
  | 'thin'       // 100
  | 'extralight' // 200
  | 'light'      // 300
  | 'normal'     // 400
  | 'medium'     // 500
  | 'semibold'   // 600
  | 'bold'       // 700
  | 'extrabold'  // 800
  | 'black';     // 900

/**
 * 文本对齐
 */
export type TextAlign = 'left' | 'center' | 'right' | 'justify';

/**
 * 文本转换
 */
export type TextTransform = 'uppercase' | 'lowercase' | 'capitalize' | 'none';

/**
 * 文本装饰
 */
export type TextDecoration = 'none' | 'underline' | 'line-through';

/**
 * 垂直对齐
 */
export type TextVerticalAlign = 'baseline' | 'sub' | 'super' | 'text-top' | 'text-bottom' | 'middle';

/**
 * 空白处理
 */
export type TextWhitespace = 'normal' | 'nowrap' | 'pre' | 'pre-line' | 'pre-wrap';

/**
 * 文本溢出处理
 */
export type TextOverflow = 'clip' | 'ellipsis';

/**
 * Text组件Props接口
 */
export interface TextProps {
  /**
   * 文本变体，决定HTML标签和默认样式
   * @default 'body'
   */
  variant?: TextVariant;

  /**
   * 文本内容
   */
  children: ReactNode;

  /**
   * 自定义类名
   */
  className?: string;

  /**
   * 字体大小
   */
  size?: TextSize;

  /**
   * 字体粗细
   */
  weight?: TextWeight;

  /**
   * 文本颜色 (CSS color值)
   * @example 'currentColor'
   * @example '#333333'
   * @example 'rgb(51, 51, 51)'
   */
  color?: string;

  /**
   * 文本对齐
   */
  align?: TextAlign;

  /**
   * 文本转换
   */
  transform?: TextTransform;

  /**
   * 文本装饰
   */
  decoration?: TextDecoration;

  /**
   * 垂直对齐
   */
  verticalAlign?: TextVerticalAlign;

  /**
   * 空白处理
   */
  whitespace?: TextWhitespace;

  /**
   * 是否截断文本（单行省略）
   * @default false
   */
  truncate?: boolean;

  /**
   * 最大行数（多行省略）
   * @example 2
   * @example 3
   */
  maxLines?: number;

  /**
   * 文本溢出时的省略号样式
   * @default 'ellipsis'
   */
  textOverflow?: TextOverflow;

  /**
   * 是否斜体
   * @default false
   */
  italic?: boolean;

  /**
   * 行高
   * @example 1.5
   * @example '1.5'
   * @example '24px'
   */
  lineHeight?: string | number;

  /**
   * 字母间距
   * @example '0.1em'
   * @example '1px'
   */
  letterSpacing?: string | number;

  /**
   * 自定义样式
   */
  style?: CSSProperties;

  /**
   * HTML标签（覆盖variant默认标签）
   * @example 'span'
   * @example 'p'
   */
  as?: keyof JSX.IntrinsicElements;

  /**
   * ID属性
   */
  id?: string;

  /**
   * 点击事件
   */
  onClick?: () => void;
}

/**
 * Text组件CSS类名接口
 */
export interface TextClasses {
  root: string;           // 根容器类名
  variant: string;        // 变体类名
  size?: string;          // 尺寸类名
  weight?: string;        // 粗细类名
  align?: string;         // 对齐类名
  truncate?: string;      // 截断类名
  multiline?: string;     // 多行省略类名
  custom?: string;        // 自定义类名
}
