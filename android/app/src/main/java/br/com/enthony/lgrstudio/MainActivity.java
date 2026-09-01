package br.com.enthony.lgrstudio;

import android.os.Bundle;
import androidx.core.view.WindowCompat;
import com.getcapacitor.BridgeActivity;

public class MainActivity extends BridgeActivity {
    @Override
    public void onCreate(Bundle savedInstanceState) {
        registerPlugin(LgrPythonPlugin.class);
        super.onCreate(savedInstanceState);

        // Garante que o WebView e o app respeitem a area util (sem ficar por baixo da barra de status)
        WindowCompat.setDecorFitsSystemWindows(getWindow(), true);
    }
}
