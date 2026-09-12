import type { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  // reverse-DNS — cannot change after the first store publish. Placeholder:
  // replace with the studio's real domain before shipping (GDD §7.1).
  appId: 'com.chainworks.game',
  appName: 'ChainWorks',
  webDir: 'dist',
  android: {
    allowMixedContent: false,
    webContentsDebuggingEnabled: false,
  },
  ios: {
    contentInset: 'never',
    scrollEnabled: false,
  },
  plugins: {
    SplashScreen: { launchAutoHide: false, backgroundColor: '#2a2420' },
    StatusBar: { style: 'DARK', overlaysWebView: false },
  },
  // Serving over https (not the default capacitor:// scheme) is required —
  // without it Rapier's WASM fails to load silently on Android (GDD §7.1).
  server: { androidScheme: 'https' },
};

export default config;
