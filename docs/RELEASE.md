# Release readiness (GDD §7)

## Done

- `capacitor.config.ts` — app id, splash/status-bar colors, `androidScheme: 'https'` (required or Rapier's WASM fails to load silently on Android, GDD §7.1).
- `android/` and `ios/App/` — full native projects generated via `npx cap add android` / `npx cap add ios`. minSdk 24 / targetSdk 36, iOS deployment target 15.0.
- `.github/workflows/ci.yml` — typecheck + full test suite (unit, in-browser physics, interactive UI) + production build on every push/PR. No secrets needed.
- `.github/workflows/android-release.yml` — builds a signed release `.aab` and uploads it to the Play internal track. **Manual trigger only** (`workflow_dispatch`) until the secrets below exist — it will fail without them.

## Still needed before shipping

These require accounts and credentials only you can create — nothing here can be scripted.

1. **Android keystore** (free, ~1 minute, do this once and never lose it):
   ```
   keytool -genkey -v -keystore release.keystore -alias chainworks -keyalg RSA -keysize 2048 -validity 10000
   ```
   Back up `release.keystore` somewhere durable (password manager / secrets vault) — losing it means you can never update the app again under the same `applicationId`. Then add as GitHub repo secrets: `ANDROID_KEYSTORE_BASE64` (`base64 -w0 release.keystore`), `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`.

2. **Google Play Console** account ($25 one-time, identity verification can take about a week). Create the app listing, then create a service-account key (Play Console → Setup → API access) with Release Manager access and add its JSON as the `GOOGLE_PLAY_SERVICE_ACCOUNT_JSON` secret.

3. **Apple Developer Program** ($99/yr, verification 1–2 weeks) — needed for iOS at all. No CI workflow is included for iOS yet (it needs a macOS runner + fastlane + an App Store Connect API key, GDD §7.2); add one once the account exists, or build/sign locally in Xcode in the meantime.

4. **Lock `appId`** in `capacitor.config.ts` (currently `com.chainworks.game`) to your studio's real reverse-DNS domain — it cannot change after the first store publish.

5. **Real store assets** (GDD §7.4): app icon (512×512 Android / 1024×1024 iOS, no transparency/rounded corners), feature graphic, 2–8 screenshots per platform, a privacy policy URL (required even collecting no data), and Android's Data Safety questionnaire. Everything currently shipping uses Capacitor's placeholder icon/splash.

6. Run `npx cap add android` was already done here; after any future dependency or web asset change, run `npx cap sync` before building natively — CI does this automatically in `android-release.yml`.

## Trying it locally

Android Studio or a device isn't available in this environment, so the WASM-loads-correctly-on-a-real-WebView check from GDD §7.1 hasn't been verified on-device — only that the build pipeline produces a correctly-configured `.apk`/`.aab` with the WASM file present in its assets. Do that check first, on a real or emulated device, before trusting anything past `M7a`.
