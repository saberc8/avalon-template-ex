# Dynamic Login Captcha Design

## Goal

Replace the login form's hardcoded captcha values with data returned by the backend captcha image API.

## Approach

The Next.js login form will call `GET /captcha/image` through the shared API client when the form mounts. The returned `uuid` will be stored in component state and submitted with the login request. The returned `img` data URL will be displayed next to the captcha input, and clicking it will refresh the captcha.

## Scope

- Add a typed `getImageCaptcha()` wrapper in the auth API module.
- Add a captcha response type with `uuid`, `img`, `expireTime`, and `isEnabled`.
- Update the login form so the captcha input starts empty instead of `"local"`.
- Submit the backend captcha `uuid` instead of the literal `"local"`.
- Refresh the captcha after failed login attempts and when the captcha image is clicked.
- Add focused tests for API wrapper behavior and login form behavior.

## Non-Goals

- Do not change the Rust captcha implementation in this task.
- Do not enforce captcha validation in the frontend when the backend returns `isEnabled=false`.
- Do not change the existing login payload shape beyond replacing the hardcoded `uuid`.

## Testing

Use Vitest and Testing Library to verify the auth API endpoint and login form behavior. Run the Next.js test suite, typecheck, and a focused search to confirm no login captcha `"local"` fallback remains.
