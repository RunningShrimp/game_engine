/**
 * Text Atom Component
 *
 * 原子级文本组件 - 最基础的文本展示单元
 *
 * 功能:
 * - 支持多种文本变体（标题、正文、说明等）
 * - 支持自定义字体大小、粗细、颜色
 * - 支持文本对齐、截断、多行省略
 * - 完全可访问性和响应式设计
 *
 * @example
 * ```tsx
 * <Text variant="h1">主标题</Text>
 * <Text variant="body" size="lg">正文内容</Text>
 * <Text variant="caption" color="gray">说明文字</Text>
 * <Text truncate maxLines={2}>长文本内容...</Text>
 * ```
 */

import React, { useMemo } from 'react';
import type { TextProps, TextVariant } from './Text.types';
import styles from './Text.module.css';

/**
 * Text组件 - 原子级文本展示组件
 */
export const Text: React.FC<TextProps> = ({
  variant = 'body',
  children,
  className,
  size,
  weight,
  color,
  align,
  transform,
  decoration,
  verticalAlign,
  whitespace,
  truncate = false,
  maxLines,
  textOverflow = 'ellipsis',
  italic = false,
  lineHeight,
  letterSpacing,
  style,
  as,
  id,
  onClick,
}) => {
  // 根据variant确定HTML标签
  const tag = useMemo(() => {
    if (as) return as;

    const tagMap: Record<TextVariant, keyof JSX.IntrinsicElements> = {
      h1: 'h1',
      h2: 'h2',
      h3: 'h3',
      h4: 'h4',
      h5: 'h5',
      h6: 'h6',
      body: 'p',
      'body-lg': 'p',
      'body-sm': 'p',
      caption: 'span',
      overline: 'span',
      subtitle1: 'h6',
      subtitle2: 'h6',
      button: 'span',
      code: 'code',
    };

    return tagMap[variant] || 'p';
  }, [as, variant]);

  // 构建CSS类名
  const classNames = useMemo(() => {
    const classes: string[] = [styles.text, styles[variant]];

    if (size) classes.push(styles[`size-${size}`]);
    if (weight) classes.push(styles[`weight-${weight}`]);
    if (align) classes.push(styles[`align-${align}`]);
    if (italic) classes.push(styles.italic);
    if (decoration) classes.push(styles[`decoration-${decoration}`]);
    if (truncate && !maxLines) classes.push(styles.truncate);
    if (maxLines) classes.push(styles.multiline);
    if (className) classes.push(className);

    return classes.join(' ');
  }, [
    variant,
    size,
    weight,
    align,
    italic,
    decoration,
    truncate,
    maxLines,
    className,
  ]);

  // 构建内联样式
  const inlineStyle = useMemo(() => {
    const customStyle: React.CSSProperties = { ...style };

    if (color) customStyle.color = color;
    if (lineHeight) customStyle.lineHeight = typeof lineHeight === 'number' ? String(lineHeight) : lineHeight;
    if (letterSpacing) customStyle.letterSpacing = typeof letterSpacing === 'number' ? `${letterSpacing}px` : letterSpacing;
    if (transform) customStyle.textTransform = transform;
    if (verticalAlign) customStyle.verticalAlign = verticalAlign;
    if (whitespace) customStyle.whiteSpace = whitespace;

    // 处理多行省略
    if (maxLines) {
      customStyle.WebkitLineClamp = maxLines;
      customStyle.WebkitBoxOrient = 'vertical';
    }

    // 处理文本溢出
    if (truncate || maxLines) {
      customStyle.overflow = 'hidden';
      customStyle.textOverflow = textOverflow;
    }

    return customStyle;
  }, [
    color,
    lineHeight,
    letterSpacing,
    transform,
    verticalAlign,
    whitespace,
    maxLines,
    truncate,
    textOverflow,
    style,
  ]);

  // 渲染组件
  const Component = tag as keyof JSX.IntrinsicElements;

  return (
    <Component
      id={id}
      className={classNames}
      style={inlineStyle}
      onClick={onClick}
    >
      {children}
    </Component>
  );
};

/**
 * 显示名称
 */
Text.displayName = 'Text';

/**
 * 默认props
 */
Text.defaultProps = {
  variant: 'body',
  truncate: false,
  italic: false,
  textOverflow: 'ellipsis',
};
