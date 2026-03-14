---
type: pattern
tags: [pattern]
---

# Adding a New Transform

Step-by-step guide for adding a new string transformation to StringKnife.

## 1. Create the Transform Function

Add a pure function in `stringknife-core/src/transforms/<module>.rs`:

```rust
/// Short description of what the transform does.
///
/// # Errors
///
/// Returns `StringKnifeError::InvalidInput` if the input is not valid for this operation.
pub fn transform_name(input: &str) -> Result<String, StringKnifeError> {
    if input.len() > MAX_INPUT_SIZE {
        return Err(StringKnifeError::InputTooLarge {
            max_bytes: MAX_INPUT_SIZE,
            actual_bytes: input.len(),
        });
    }
    // Transform logic here
    Ok(result)
}
```

## 2. Add Tests

In the same file, add a `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_name_happy_path() {
        assert_eq!(transform_name("input").unwrap(), "expected");
    }

    #[test]
    fn test_transform_name_empty_input() {
        assert_eq!(transform_name("").unwrap(), "");
    }

    #[test]
    fn test_transform_name_error_case() {
        assert!(transform_name("invalid").is_err());
    }
}
```

## 3. Export from Module

Add `pub mod <module>;` in `stringknife-core/src/transforms/mod.rs` if it's a
new module file.

## 4. Register as Code Action

In `stringknife-lsp/src/main.rs`, add the transform to the `build_actions()` function.
Use one of the existing patterns:

- **Encode (always shown):** `try_encode("StringKnife: Name", "category", transform_fn(selected));`
- **Decode (smart detection):** `try_decode("StringKnife: Name", "category", transform_fn(selected));`

The category string must match one of: `encoding`, `hashing`, `case`, `json`, `xml`, `csv`, `whitespace`, `escape`, `inspect`, `misc`.

## 5. Update Registry

Add a row to [[Transform Registry]] with status, test count, and ticket number.

## Anti-Patterns

- **No I/O**: Transforms must not read files, make network calls, or access env vars
- **No `unwrap()`**: Always return `Result` — use `?` operator
- **No `unsafe`**: Not allowed in transforms
- **No LSP types**: The transforms crate must not depend on `tower-lsp`
- **No side effects**: No mutation of shared state, no logging in transforms
