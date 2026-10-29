package br.com.enthony.controlab;

import android.app.Activity;
import android.app.Application;
import android.os.Build;
import android.view.View;
import android.view.Window;
import android.view.WindowInsetsController;

/** Process-scoped Android application; scientific calculations run in Rust. */
public final class NativeApplication extends Application {
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
