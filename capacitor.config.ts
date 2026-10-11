import type { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'br.com.enthony.lgrstudio',
  appName: 'ControLAB',
  webDir: 'src/renderer',
  android: {
    allowMixedContent: false,
    backgroundColor: '#0f172a'
  },
  plugins: {
    SystemBars: {
      insetsHandling: 'css'
    }
  }
};

export default config;
