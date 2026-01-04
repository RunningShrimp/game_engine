/**
 * Atom Components
 *
 * Basic building blocks that can be combined to create more complex UI components.
 * These components are designed to be simple, reusable, and composable.
 *
 * @example
 * ```tsx
 * import { Icon, Text, Badge, Avatar } from '@/components/atoms';
 *
 * function MyComponent() {
 *   return (
 *     <div>
 *       <Avatar initials="JD" size="md" />
 *       <Text variant="h3">John Doe</Text>
 *       <Badge variant="success">Active</Badge>
 *     </div>
 *   );
 * }
 * ```
 */

// Icon Component
export { Icon } from './Icon';
export type { IconProps } from './Icon';

// Text Component
export { Text } from './Text';
export type { TextProps, TextVariant } from './Text';

// Badge Component
export { Badge } from './Badge';
export type { BadgeProps, BadgeVariant, BadgeSize } from './Badge';

// Avatar Component
export { Avatar } from './Avatar';
export type { AvatarProps, AvatarSize } from './Avatar';

// Divider Component
export { Divider } from './Divider';
export type { DividerProps, DividerOrientation } from './Divider';

// Spacer Component
export { Spacer } from './Spacer';
export type { SpacerProps, SpacerAxis, SpacerSize } from './Spacer';

// Tooltip Component
export { Tooltip } from './Tooltip';
export type { TooltipProps, TooltipPosition } from './Tooltip';

// ProgressBar Component
export { ProgressBar } from './ProgressBar';
export type { ProgressBarProps, ProgressBarSize, ProgressBarVariant } from './ProgressBar';

// Spinner Component
export { Spinner } from './Spinner';
export type { SpinnerProps, SpinnerSize, SpinnerColor } from './Spinner';

// Skeleton Component
export { Skeleton, CardSkeleton, TableSkeleton } from './Skeleton';
export type { SkeletonProps, SkeletonVariant } from './Skeleton';
