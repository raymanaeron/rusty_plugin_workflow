# Plugin Migration Guide: Bootstrap to daisyUI

This document outlines the approach used to migrate plugins from Bootstrap to daisyUI styling, using `plugin_welcome` as a reference implementation.

## General Approach

1. **Keep HTML structure intact**: Preserve the overall DOM structure to maintain functionality
2. **Replace class names**: Swap Bootstrap classes with daisyUI/Tailwind equivalents
3. **Preserve all functionality**: Ensure all JavaScript interactions continue to work
4. **Match visual appearance**: Create a similar look and feel with the new styling system

## Common Class Replacements

### Layout
- `container` → `container mx-auto`
- `row` → `flex flex-wrap`
- `col-*` → Tailwind width utilities (`w-full`, `md:w-1/2`, etc.)

### Components
- `card` → `card bg-base-100`
- `card-body` → `card-body`
- `card-title` → `card-title`
- `btn` → `btn`
- `btn-primary` → `btn-primary`
- `alert-*` → `alert alert-*`

### Utilities
- `text-center` → `text-center`
- `mb-4` → `mb-4` (Tailwind uses similar spacing scale)
- `d-flex` → `flex`
- `justify-content-between` → `justify-between`

## Implementation Example

### Before (Bootstrap):
```html
<div class="card">
  <div class="card-body">
    <h5 class="card-title">Welcome</h5>
    <p class="card-text">This is the welcome message.</p>
    <div class="alert alert-info">
      Important information here.
    </div>
    <button class="btn btn-primary">Get Started</button>
  </div>
</div>
```

### After (daisyUI):
```html
<div class="card bg-base-100 shadow-lg">
  <div class="card-body">
    <h5 class="card-title">Welcome</h5>
    <p>This is the welcome message.</p>
    <div class="alert alert-info">
      <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-6 h-6">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
      </svg>
      <span>Important information here.</span>
    </div>
    <button class="btn btn-primary">Get Started</button>
  </div>
</div>
```

## Testing Checklist

After migrating a plugin, verify:

1. ✓ Visual appearance is consistent with the application styling
2. ✓ All interactive elements work as before
3. ✓ Theme switching is applied correctly
4. ✓ Responsive layouts function as expected
5. ✓ No JavaScript errors in console
6. ✓ All plugin functionality works as before

## Notes for Future Plugin Migrations

- Use the compatibility classes in styles.css when possible
- For complex components, refer to the daisyUI documentation
- Theme colors (`primary`, `secondary`, etc.) are automatically applied
- Use semantic color classes for consistent theming
