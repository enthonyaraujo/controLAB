package br.com.enthony.controlab;

import android.util.Log;
import com.chaquo.python.PyObject;
import com.chaquo.python.Python;
import com.chaquo.python.android.PyApplication;
import org.json.JSONException;
import org.json.JSONObject;

/** Process-scoped Python engine called directly from Rust worker threads via JNI. */
public final class NativeApplication extends PyApplication {
    private PyObject bridge;

    // Serialize engine access: the canonical engine and Matplotlib have mutable state.
    public synchronized String dispatchJson(String payload) {
        try {
            if (bridge == null) {
                bridge = Python.getInstance().getModule("native_presentation");
            }
            return bridge.callAttr("dispatch_android", payload).toString();
        } catch (Exception error) {
            Log.e("ControLAB", "Python dispatch failed", error);
            JSONObject response = new JSONObject();
            try {
                response.put("success", false);
                response.put("error", error.getClass().getSimpleName() + ": " + error.getMessage());
            } catch (JSONException impossible) {
                return "{\"success\":false,\"error\":\"Falha na ponte Python Android\"}";
            }
            return response.toString();
        }
    }
}
