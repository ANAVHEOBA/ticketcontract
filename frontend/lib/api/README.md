# API Layer

Use this flow for fastest delivery:

1. Implement/adjust API function in `lib/api/*`.
2. Build component/page using that function.
3. Add loading/error state in component.

This keeps backend contracts centralized and prevents fetch logic duplication.
