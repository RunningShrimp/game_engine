/**
 * Text Atom Component Examples
 *
 * 展示Text组件的各种使用场景
 */

import React from 'react';
import { Text } from './Text';

/**
 * 基础示例 - 各种文本变体
 */
export const BasicVariants = () => {
  return (
    <div style={{ padding: '2rem', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <Text variant="h1">Heading 1 - 主标题</Text>
      <Text variant="h2">Heading 2 - 二级标题</Text>
      <Text variant="h3">Heading 3 - 三级标题</Text>
      <Text variant="body">Body text - 正文内容</Text>
      <Text variant="caption">Caption text - 说明文字</Text>
      <Text variant="code">const x = 1;</Text>
    </div>
  );
};

/**
 * 字体大小示例
 */
export const SizeExamples = () => {
  return (
    <div style={{ padding: '2rem', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <Text size="xs">Extra Small Text</Text>
      <Text size="sm">Small Text</Text>
      <Text size="base">Base Text</Text>
      <Text size="lg">Large Text</Text>
      <Text size="xl">Extra Large Text</Text>
      <Text size="2xl">2X Large Text</Text>
    </div>
  );
};

/**
 * 字体粗细示例
 */
export const WeightExamples = () => {
  return (
    <div style={{ padding: '2rem', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <Text weight="thin">Thin (100)</Text>
      <Text weight="light">Light (300)</Text>
      <Text weight="normal">Normal (400)</Text>
      <Text weight="medium">Medium (500)</Text>
      <Text weight="semibold">Semibold (600)</Text>
      <Text weight="bold">Bold (700)</Text>
      <Text weight="extrabold">Extra Bold (800)</Text>
    </div>
  );
};

/**
 * 文本对齐示例
 */
export const AlignmentExamples = () => {
  return (
    <div style={{ padding: '2rem', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <Text align="left">Left aligned text</Text>
      <Text align="center">Center aligned text</Text>
      <Text align="right">Right aligned text</Text>
      <Text align="justify">
        Justified text - This paragraph is justified, meaning that the text is
        spaced so that the left and right edges are straight. This creates a
        clean and formal look, often used in newspapers and books.
      </Text>
    </div>
  );
};

/**
 * 文本颜色示例
 */
export const ColorExamples = () => {
  return (
    <div style={{ padding: '2rem', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <Text color="#333333">Dark gray text</Text>
      <Text color="#666666">Medium gray text</Text>
      <Text color="red">Red text</Text>
      <Text color="blue">Blue text</Text>
      <Text color="green">Green text</Text>
      <Text color="currentColor">Inherits parent color</Text>
    </div>
  );
};

/**
 * 文本截断示例
 */
export const TruncationExamples = () => {
  const longText =
    'This is a very long text that should be truncated with ellipsis to show that there is more content that is not visible.';

  return (
    <div style={{ padding: '2rem', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <div style={{ maxWidth: '300px' }}>
        <Text truncate>{longText}</Text>
      </div>
      <div style={{ maxWidth: '300px' }}>
        <Text maxLines={2}>
          {longText} {longText}
        </Text>
      </div>
      <div style={{ maxWidth: '300px' }}>
        <Text maxLines={3}>
          {longText} {longText} {longText}
        </Text>
      </div>
    </div>
  );
};

/**
 * 文本样式示例
 */
export const StyleExamples = () => {
  return (
    <div style={{ padding: '2rem', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <Text italic>Italic text</Text>
      <Text decoration="underline">Underlined text</Text>
      <Text decoration="line-through">Strikethrough text</Text>
      <Text transform="uppercase">Uppercase text</Text>
      <Text transform="capitalize">capitalized text</Text>
      <Text transform="lowercase">LOWERCASE TEXT</Text>
    </div>
  );
};

/**
 * 行高和字间距示例
 */
export const SpacingExamples = () => {
  return (
    <div style={{ padding: '2rem', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <Text lineHeight={1}>Line height 1 - Tight</Text>
      <Text lineHeight={1.5}>Line height 1.5 - Normal</Text>
      <Text lineHeight={2}>Line height 2 - Relaxed</Text>
      <Text letterSpacing="normal">Normal letter spacing</Text>
      <Text letterSpacing="0.1em">Wide letter spacing</Text>
      <Text letterSpacing="0.2em">Very wide letter spacing</Text>
    </div>
  );
};

/**
 * 自定义标签示例
 */
export const CustomTagExamples = () => {
  return (
    <div style={{ padding: '2rem', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <Text as="span">This is a span</Text>
      <Text as="label">This is a label</Text>
      <Text as="strong">This is strong</Text>
      <Text as="em">This is emphasis</Text>
    </div>
  );
};

/**
 * 组合示例 - 卡片标题
 */
export const CardTitle = () => {
  return (
    <div style={{ padding: '2rem', border: '1px solid #ccc', borderRadius: '8px' }}>
      <Text variant="h5" weight="semibold" color="#333">
        Card Title
      </Text>
      <Text variant="body" size="sm" color="#666" style={{ marginTop: '0.5rem' }}>
        Card description goes here with some details about the content.
      </Text>
    </div>
  );
};

/**
 * 组合示例 - 按钮文本
 */
export const ButtonText = () => {
  return (
    <div style={{ padding: '2rem', display: 'flex', gap: '1rem' }}>
      <button
        style={{
          padding: '0.5rem 1rem',
          backgroundColor: '#007bff',
          color: 'white',
          border: 'none',
          borderRadius: '4px',
          cursor: 'pointer',
        }}
      >
        <Text variant="button" style={{ color: 'white' }}>
          Click Me
        </Text>
      </button>
    </div>
  );
};

/**
 * 响应式示例
 */
export const ResponsiveExample = () => {
  return (
    <div style={{ padding: '2rem' }}>
      <Text variant="h1" size="3xl">
        Responsive heading
      </Text>
      <Text variant="body">
        Resize the browser window to see the heading size adjust for smaller screens.
      </Text>
    </div>
  );
};

/**
 * 可点击文本示例
 */
export const ClickableText = () => {
  const handleClick = () => {
    alert('Text clicked!');
  };

  return (
    <div style={{ padding: '2rem' }}>
      <Text onClick={handleClick} color="#007bff" style={{ cursor: 'pointer' }}>
        Clickable link text
      </Text>
    </div>
  );
};

/**
 * 所有示例的集合
 */
export const AllTextExamples = () => {
  return (
    <div>
      <BasicVariants />
      <hr style={{ margin: '2rem 0' }} />
      <SizeExamples />
      <hr style={{ margin: '2rem 0' }} />
      <WeightExamples />
      <hr style={{ margin: '2rem 0' }} />
      <AlignmentExamples />
      <hr style={{ margin: '2rem 0' }} />
      <ColorExamples />
      <hr style={{ margin: '2rem 0' }} />
      <TruncationExamples />
      <hr style={{ margin: '2rem 0' }} />
      <StyleExamples />
      <hr style={{ margin: '2rem 0' }} />
      <SpacingExamples />
      <hr style={{ margin: '2rem 0' }} />
      <CustomTagExamples />
      <hr style={{ margin: '2rem 0' }} />
      <CardTitle />
      <hr style={{ margin: '2rem 0' }} />
      <ButtonText />
    </div>
  );
};
