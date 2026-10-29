package br.com.enthony.controlab;

import android.app.Activity;
import android.os.Build;
import android.util.Log;
import android.view.View;
import android.view.Window;
import android.view.WindowInsetsController;
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

    /**
     * Dark or light icons on the status and navigation bars, from Rust's native thread. The
     * window stays edge-to-edge; only icon contrast changes, and only on the UI thread.
     */
    @SuppressWarnings("deprecation") // View flags are the only API before Android 11.
    public void setSystemBarIcons(Activity activity, boolean darkStatus, boolean darkNavigation) {
        activity.runOnUiThread(() -> {
            Window window = activity.getWindow();
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                WindowInsetsController controller = window.getInsetsController();
                if (controller == null) {
                    return;
                }
                int status = WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS;
                int navigation = WindowInsetsController.APPEARANCE_LIGHT_NAVIGATION_BARS;
                controller.setSystemBarsAppearance(
                        (darkStatus ? status : 0) | (darkNavigation ? navigation : 0),
                        status | navigation);
            } else {
                View decor = window.getDecorView();
                int status = View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR;
                int navigation = Build.VERSION.SDK_INT >= Build.VERSION_CODES.O
                        ? View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR : 0;
                int flags = decor.getSystemUiVisibility() & ~(status | navigation);
                decor.setSystemUiVisibility(flags
                        | (darkStatus ? status : 0) | (darkNavigation ? navigation : 0));
            }
        });
    }
}
