package br.com.enthony.lgrstudio;

import android.os.Bundle;
import com.getcapacitor.BridgeActivity;

public class MainActivity extends BridgeActivity {
    @Override
    public void onCreate(Bundle savedInstanceState) {
        registerPlugin(LgrPythonPlugin.class);
        super.onCreate(savedInstanceState);
    }
}
