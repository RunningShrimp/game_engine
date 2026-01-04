# Molecule Components

Molecule components are simple groups of UI elements that function together as a unit. They are composed of Atom components and provide more complex functionality for form inputs, buttons, and other interactive elements.

## Overview

Molecules are the second level of the Atomic Design methodology. They combine multiple atoms to form relatively simple, yet distinct, UI components. All molecule components follow consistent design patterns and include comprehensive support for:

- Form validation (value/onChange)
- Error states with messages
- Disabled states
- Required field indicators
- Help text
- Complete focus states
- Accessibility features

## Available Components

### Button
A versatile button component composed of Icon + Text atoms.

**Features:**
- Multiple variants (primary, secondary, outline, ghost, danger)
- Size options (xs, sm, md, lg, xl)
- Icon support with configurable position
- Loading state with spinner
- Full width option
- Error state styling

```tsx
import { Button } from '@/components/molecules';

<Button variant="primary" icon="Send" onClick={handleSubmit}>
  Submit
</Button>

<Button variant="outline" icon="Download" iconPosition="right">
  Download
</Button>

<Button variant="danger" isLoading>
  Deleting...
</Button>
```

### Input
A text input component composed of Icon + Text input atoms.

**Features:**
- Icon support (left/right positions)
- Label and required indicator
- Error state with message
- Help text
- Controlled and uncontrolled modes
- Size variants (sm, md, lg)

```tsx
import { Input } from '@/components/molecules';

<Input
  label="Email"
  icon="Mail"
  placeholder="Enter your email"
  value={email}
  onChange={setEmail}
  required
/>

<Input
  label="Password"
  icon="Lock"
  type="password"
  error={hasError}
  errorMessage="Password is required"
/>
```

### TextArea
A multi-line text input component.

**Features:**
- Auto-resize support
- Character count with maxLength
- Label and help text
- Error state handling
- Size variants

```tsx
import { TextArea } from '@/components/molecules';

<TextArea
  label="Description"
  placeholder="Enter a description"
  value={description}
  onChange={setDescription}
  rows={4}
  maxLength={500}
  showCount
/>
```

### Select
A dropdown select component composed of Icon + Select elements.

**Features:**
- Icon support
- Option array configuration
- Placeholder text
- Label and help text
- Error state handling

```tsx
import { Select } from '@/components/molecules';

const options = [
  { value: 'apple', label: 'Apple' },
  { value: 'banana', label: 'Banana' },
  { value: 'orange', label: 'Orange' },
];

<Select
  label="Fruit"
  icon="Apple"
  options={options}
  placeholder="Select a fruit"
  value={fruit}
  onChange={setFruit}
  required
/>
```

### Checkbox
A checkbox input component with label support.

**Features:**
- Indeterminate state support
- Label and help text
- Error state handling
- Size variants
- Controlled and uncontrolled modes

```tsx
import { Checkbox } from '@/components/molecules';

<Checkbox
  label="Accept terms and conditions"
  checked={accepted}
  onChange={setAccepted}
  required
/>

<Checkbox
  label="Select all"
  indeterminate={someSelected}
  checked={allSelected}
/>
```

### Radio
A radio group component with support for multiple options.

**Features:**
- Multiple options configuration
- Vertical/horizontal orientation
- Label and help text
- Error state handling
- Individual option help text

```tsx
import { Radio } from '@/components/molecules';

const options = [
  { value: 'apple', label: 'Apple' },
  { value: 'banana', label: 'Banana' },
  { value: 'orange', label: 'Orange' },
];

<Radio
  label="Select a fruit"
  options={options}
  value={fruit}
  onChange={setFruit}
  orientation="horizontal"
  required
/>
```

### Switch
A toggle switch component with label support.

**Features:**
- Smooth animations
- Label and help text
- Error state handling
- Size variants
- Visual check indicator

```tsx
import { Switch } from '@/components/molecules';

<Switch
  label="Enable notifications"
  checked={enabled}
  onChange={setEnabled}
  helpText="Receive push notifications"
/>

<Switch
  label="Dark mode"
  defaultChecked
/>
```

### Slider
A range slider component with value display.

**Features:**
- Min/max constraints
- Step control
- Value formatting
- Marks at intervals
- Label and help text
- Error state handling
- Change commit handler

```tsx
import { Slider } from '@/components/molecules';

<Slider
  label="Volume"
  value={volume}
  onChange={setVolume}
  min={0}
  max={100}
  showValue
/>

<Slider
  label="Opacity"
  value={opacity}
  onChange={setOpacity}
  min={0}
  max={1}
  step={0.1}
  valueFormat={(v) => `${Math.round(v * 100)}%`}
  marks={[0, 0.25, 0.5, 0.75, 1]}
/>
```

### SearchInput
A search input component composed of Input + Icon atoms.

**Features:**
- Search icon
- Clear button
- Debounced search
- Enter key support
- Configurable debounce delay

```tsx
import { SearchInput } from '@/components/molecules';

<SearchInput
  label="Search users"
  placeholder="Enter name or email"
  value={searchQuery}
  onChange={setSearchQuery}
  onSearch={handleSearch}
  debounceDelay={500}
/>
```

### ColorPicker
A color picker input component with preview swatch.

**Features:**
- Color preview swatch
- Preset colors
- Multiple color formats (hex, rgb, hsl)
- Label and help text
- Error state handling

```tsx
import { ColorPicker } from '@/components/molecules';

<ColorPicker
  label="Primary color"
  value={color}
  onChange={setColor}
  format="hex"
  presetColors={['#000000', '#FFFFFF', '#FF0000']}
/>
```

### NumberInput
A number input component with increment/decrement controls.

**Features:**
- Increment/decrement buttons
- Min/max constraints
- Precision control
- Custom value formatting
- Label and help text

```tsx
import { NumberInput } from '@/components/molecules';

<NumberInput
  label="Quantity"
  value={quantity}
  onChange={setQuantity}
  min={0}
  max={100}
  step={1}
/>

<NumberInput
  label="Price"
  value={price}
  onChange={setPrice}
  min={0}
  step={0.01}
  precision={2}
  formatValue={(v) => `$${v.toFixed(2)}`}
/>
```

### InputGroup
Groups multiple input components together with separators.

**Features:**
- Multiple input grouping
- Configurable separators (-, ., :, /)
- Horizontal/vertical layout
- Common label and error handling

```tsx
import { InputGroup, Input, NumberInput } from '@/components/molecules';

<InputGroup label="Date of Birth" separator="/">
  <Input type="text" placeholder="MM" maxLength={2} />
  <Input type="text" placeholder="DD" maxLength={2} />
  <Input type="text" placeholder="YYYY" maxLength={4} />
</InputGroup>

<InputGroup label="IP Address" separator=".">
  <Input type="text" placeholder="000" maxLength={3} />
  <Input type="text" placeholder="000" maxLength={3} />
  <Input type="text" placeholder="000" maxLength={3} />
  <Input type="text" placeholder="000" maxLength={3} />
</InputGroup>
```

### Label
A form label component with required indicator and help text.

**Features:**
- Required indicator
- Help text display
- Error state styling
- Association with form elements

```tsx
import { Label } from '@/components/molecules';

<Label htmlFor="email" required>
  Email
</Label>

<Label
  htmlFor="password"
  error
  helpText="Must be at least 8 characters"
>
  Password
</Label>
```

## Common Props

All molecule components share these common props:

### Form Props
- `value` - Controlled value
- `defaultValue` - Uncontrolled default value
- `onChange` - Change handler
- `disabled` - Disable the component
- `required` - Mark field as required
- `error` - Error state
- `errorMessage` - Error message to display
- `helpText` - Help/description text
- `label` - Field label
- `fullWidth` - Take full width of parent

### Size Props
- `size` - Component size ('sm' | 'md' | 'lg')

### Accessibility Props
- All components support standard ARIA attributes
- Proper label associations
- Keyboard navigation support
- Screen reader friendly

## Styling

All molecule components use Tailwind CSS for styling and support:

- Consistent spacing and sizing
- Focus states with rings
- Error states with colors
- Disabled state styling
- Smooth transitions
- Responsive design

## Best Practices

### 1. Form Validation
Always use controlled components for form validation:

```tsx
const [email, setEmail] = useState('');
const [error, setError] = useState('');

const validateEmail = (value: string) => {
  if (!value) {
    setError('Email is required');
    return false;
  }
  if (!/^\S+@\S+\.\S+$/.test(value)) {
    setError('Invalid email format');
    return false;
  }
  setError('');
  return true;
};

<Input
  label="Email"
  value={email}
  onChange={(value) => {
    setEmail(value);
    validateEmail(value);
  }}
  error={!!error}
  errorMessage={error}
  required
/>
```

### 2. Loading States
Use loading states for async operations:

```tsx
const [isLoading, setIsLoading] = useState(false);

const handleSubmit = async () => {
  setIsLoading(true);
  try {
    await submitForm();
  } finally {
    setIsLoading(false);
  }
};

<Button isLoading={isLoading} onClick={handleSubmit}>
  Submit
</Button>
```

### 3. Error Handling
Provide clear error messages:

```tsx
<Input
  label="Username"
  value={username}
  onChange={setUsername}
  error={usernameError}
  errorMessage={usernameError || 'Username must be at least 3 characters'}
  required
/>
```

### 4. Accessibility
Always include proper labels and ARIA attributes:

```tsx
<Input
  label="Email Address"
  placeholder="you@example.com"
  aria-label="Email address input"
  aria-describedby="email-help"
  required
/>
```

## TypeScript Support

All components are fully typed with TypeScript. Import types as needed:

```tsx
import type {
  ButtonProps,
  InputProps,
  SelectOption,
  CheckboxProps,
} from '@/components/molecules';
```

## Composition Examples

### Login Form
```tsx
<form onSubmit={handleSubmit}>
  <Input
    label="Email"
    type="email"
    icon="Mail"
    value={email}
    onChange={setEmail}
    required
  />

  <Input
    label="Password"
    type="password"
    icon="Lock"
    value={password}
    onChange={setPassword}
    required
  />

  <Checkbox
    label="Remember me"
    checked={remember}
    onChange={setRemember}
  />

  <Button type="submit" fullWidth>
    Sign In
  </Button>
</form>
```

### Settings Panel
```tsx
<div>
  <Switch
    label="Dark mode"
    checked={darkMode}
    onChange={setDarkMode}
    helpText="Enable dark theme"
  />

  <Slider
    label="Font size"
    value={fontSize}
    onChange={setFontSize}
    min={12}
    max={24}
    step={1}
    showValue
  />

  <ColorPicker
    label="Accent color"
    value={accentColor}
    onChange={setAccentColor}
    format="hex"
  />

  <Select
    label="Language"
    options={languageOptions}
    value={language}
    onChange={setLanguage}
  />
</div>
```

## Migration Guide

If you're migrating from existing form components:

1. Replace standard inputs with `<Input>` components
2. Add labels using the `label` prop
3. Implement error states with `error` and `errorMessage` props
4. Use controlled components with `value` and `onChange`
5. Add icons using the `icon` prop
6. Include help text with the `helpText` prop

## Contributing

When adding new molecule components:

1. Compose them from existing atom components
2. Follow the established prop patterns
3. Include TypeScript types
4. Add comprehensive examples
5. Support error, disabled, and required states
6. Include accessibility features
7. Add to the index.ts exports
8. Update this README

## Related Components

- **Atoms** - Basic building blocks (Icon, Text, etc.)
- **Organisms** - Complex components composed of molecules
- **Templates** - Page-level components

## License

MIT
